use std::sync::Arc;

use anyhow::Result;
use axum::async_trait;
use chrono::{DateTime, Duration, Utc};
use tracing::debug;
use uuid::Uuid;

use domain_model::{Action, Candle, CreateSimulation, Currency, CurrencyPair, Exchange, InstrumentId, MarketType, NewDeployment, Order, OrderActionType, OrderMarketType, OrderStatus, OrderType, Position, Side, Simulation, SimulationDeployment, SimulationPosition, Size, Tick};
use engine_core_api::api::EngineApi;
use interactor_core_api::InteractorApi;
use simulator_core_api::{SimulationReport, SimulatorApi};
use simulator_persistence_api::SimulationReportRepository;
use storage_core_api::StorageApi;

use crate::file_logger::Logger;

pub struct Simulator<E: EngineApi, S: StorageApi, I: InteractorApi, SR: SimulationReportRepository> {
    engine_client: Arc<E>,
    storage_client: Arc<S>,
    interactor_client: Arc<I>,
    simulation_report_repository: Arc<SR>,
}

#[async_trait]
impl<E: EngineApi, S: StorageApi, I: InteractorApi, SR: SimulationReportRepository> SimulatorApi
for Simulator<E, S, I, SR>
{
    async fn run_simulation(&self, simulation: CreateSimulation) -> Result<SimulationReport> {
        let simulation: Simulation = simulation.into();
        let mut logger = Logger::new(simulation.id);
        let report = self
            .run_simulation_with_logger(simulation, &mut logger)
            .await;
        logger.save();
        Ok(report)
    }

    async fn get_simulation_report(&self, id: Uuid) -> Result<SimulationReport> {
        self.simulation_report_repository
            .get(Some(id))
            .await
            .first()
            .cloned()
            .ok_or(anyhow::Error::msg("Simulation report not found"))
    }

    async fn get_simulation_reports(&self) -> Result<Vec<SimulationReport>> {
        let reports = self.simulation_report_repository.get(None).await;
        Ok(reports)
    }
}

impl<E: EngineApi, S: StorageApi, I: InteractorApi, SR: SimulationReportRepository>
Simulator<E, S, I, SR>
{
    pub fn new(
        engine_client: Arc<E>,
        storage_client: Arc<S>,
        interactor_client: Arc<I>,
        simulation_report_repository: Arc<SR>,
    ) -> Self {
        Self {
            engine_client,
            storage_client,
            interactor_client,
            simulation_report_repository,
        }
    }

    async fn run_simulation_with_logger(
        &self,
        mut simulation: Simulation,
        logger: &mut Logger,
    ) -> SimulationReport {
        logger.log(format!("Start simulation: '{:?}'", simulation));
        self.create_positions(&simulation).await;
        self.create_deployments(&mut simulation).await;
        let mut simulation_stats = SimulationStats::default();

        let mut batch_start = simulation.start;
        let mut batch_end = simulation.start;

        while batch_end != simulation.end {
            let new_batch_end = batch_end + Duration::days(7);
            batch_end = if new_batch_end < simulation.end {
                new_batch_end
            } else {
                simulation.end
            };

            self.run_simulation_batch(logger, &mut simulation, &mut simulation_stats, batch_start, batch_end)
                .await;

            batch_start += Duration::days(7);
        }
        self.delete_deployments(&simulation.deployments).await;

        let report = self.build_report(simulation, simulation_stats).await;
        self.simulation_report_repository
            .save(report.clone())
            .await
            .unwrap();
        logger.log(format!("{report:?}"));
        report
    }

    async fn run_simulation_batch(
        &self,
        logger: &mut Logger,
        simulation: &mut Simulation,
        simulation_stats: &mut SimulationStats,
        batch_start: DateTime<Utc>,
        batch_end: DateTime<Utc>,
    ) {
        debug!("Batch processing from start: {batch_start} to end: {batch_end}");
        let ticks = self
            .get_ticks(logger, simulation, batch_start, batch_end)
            .await;
        let positions = &mut simulation.positions;
        let active_orders = &mut simulation.active_orders;
        debug!("Ticks len: {}", ticks.len());
        simulation.ticks_len += ticks.len() as u32;
        for tick in &ticks {
            logger.log(format!(
                "| Tick: {} '{}' {}-{}='{}'",
                tick.instrument_id.exchange,
                tick.timestamp,
                tick.instrument_id.pair.target,
                tick.instrument_id.pair.source,
                tick.price
            ));
            self.check_active_orders(active_orders, tick, positions, simulation_stats, logger)
                .await;
            let actions = self.engine_client.get_actions(tick).await;
            for action in &actions {
                logger.log(format!(
                    "|* Action: {:?} \n   for tick: {} '{}' {}-{}='{}'",
                    action,
                    tick.instrument_id.exchange,
                    tick.timestamp,
                    tick.instrument_id.pair.target,
                    tick.instrument_id.pair.source,
                    tick.price
                ));
                simulation.actions_count += 1;
                self.execute_action(tick.timestamp, action, active_orders, logger)
                    .await;
            }
            self.check_active_orders(active_orders, tick, positions, simulation_stats, logger)
                .await;
        }
    }

    async fn build_report(&self, simulation: Simulation, simulation_stats: SimulationStats) -> SimulationReport {
        let mut positions = simulation.positions;
        positions
            .iter_mut()
            .for_each(|position| position.diff = position.end - position.start);

        let profit = self.calculate_profit(&positions, simulation.end).await;
        let profit_clear = self.calculate_profit(&positions, simulation.start).await;
        let fees = self.calculate_fees(&positions, simulation.end).await;

        SimulationReport {
            simulation_id: simulation.id,
            start: simulation.start,
            end: simulation.end,
            deployments: simulation.deployments,
            ticks: simulation.ticks_len,
            actions: simulation.actions_count,
            profit,
            profit_clear,
            assets: positions,
            fees,
            active_orders: simulation.active_orders,
            sl_count: simulation_stats.sl_count,
            tp_count: simulation_stats.tp_count,
            sl_percent: simulation_stats.sl_percent(),
            tp_percent: simulation_stats.tp_percent(),
            max_sl_streak: simulation_stats.max_sl_streak,
            max_tp_streak: simulation_stats.max_tp_streak,
        }
    }

    async fn calculate_profit(
        &self,
        positions_diff: &[SimulationPosition],
        timestamp: DateTime<Utc>,
    ) -> f64 {
        let mut result = 0.0;
        for position_diff in positions_diff {
            let instrument_id = InstrumentId {
                exchange: position_diff.exchange,
                market_type: MarketType::Spot,
                pair: CurrencyPair {
                    target: position_diff.currency,
                    source: Currency::USDT,
                },
            };
            result += self
                .convert_currency(
                    &instrument_id,
                    timestamp,
                    position_diff.diff,
                    CurrencyConversion::ToSource,
                )
                .await;
        }
        result
    }

    async fn calculate_fees(
        &self,
        positions_diff: &[SimulationPosition],
        timestamp: DateTime<Utc>,
    ) -> f64 {
        let mut result = 0.0;
        for position_diff in positions_diff {
            let instrument_id = InstrumentId {
                exchange: position_diff.exchange,
                market_type: MarketType::Spot,
                pair: CurrencyPair {
                    target: position_diff.currency,
                    source: Currency::USDT,
                },
            };
            result += self
                .convert_currency(
                    &instrument_id,
                    timestamp,
                    position_diff.fees,
                    CurrencyConversion::ToSource,
                )
                .await;
        }
        result
    }

    async fn convert_currency(
        &self,
        instrument_id: &InstrumentId,
        timestamp: DateTime<Utc>,
        value: f64,
        conversion_type: CurrencyConversion,
    ) -> f64 {
        if instrument_id.pair.source == instrument_id.pair.target {
            return value;
        }
        let price = self
            .interactor_client
            .get_price(instrument_id, Some(timestamp))
            .await
            .expect("No find price for currency conversion");
        match conversion_type {
            CurrencyConversion::ToTarget => value / price,
            CurrencyConversion::ToSource => value * price,
        }
    }

    async fn execute_action(
        &self,
        timestamp: DateTime<Utc>,
        action: &Action,
        active_orders: &mut Vec<Order>,
        logger: &mut Logger,
    ) {
        match action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => {
                    let order = Order {
                        id: create_order.id.clone(),
                        timestamp,
                        simulation_id: order_action.simulation_id,
                        exchange: order_action.exchange,
                        status: OrderStatus::Created,
                        market_type: create_order.market_type,
                        order_type: create_order.order_type,
                        pair: create_order.pair,
                        side: create_order.side,
                        size: create_order.size.clone(),
                        fee: 0.,
                        avg_fill_price: 0.,
                        stop_loss: create_order.stop_loss.clone(),
                        avg_sl_price: 0.,
                        take_profit: create_order.take_profit.clone(),
                        avg_tp_price: 0.,
                    };
                    self.storage_client.save_order(order.clone()).await.unwrap();
                    logger.log(format!("|-> Place Order: {} {:?} {:?} '{}-{}' {} '{:?}', stop-loss: {:?}, take-profit: {:?}, id: '{}'",
                                       order.exchange, order.market_type, order.order_type, order.pair.target, order.pair.source, order.side, order.size, order.stop_loss, order.take_profit, order.id));
                    active_orders.push(order);
                }
                OrderActionType::PatchOrder => unimplemented!(),
                OrderActionType::CancelOrder(_cancel_order) => unimplemented!(),
            },
        }
    }

    async fn create_positions(&self, simulation: &Simulation) {
        for position in simulation.positions.clone().iter() {
            let position = Position::from(position.clone());
            self.storage_client.save_position(position).await.unwrap();
        }
    }

    async fn create_deployments(&self, simulation: &mut Simulation) {
        let create_deployments: Vec<_> = simulation
            .deployments
            .iter()
            .map(|strategy| convert_to_create_deployment_dto(strategy.clone(), simulation.id))
            .collect();
        let created_deployments = self
            .engine_client
            .deploy(&create_deployments)
            .await
            .unwrap();
        for deployment in created_deployments {
            for simulation_deployment in simulation.deployments.iter_mut() {
                if deployment.plugin_id == simulation_deployment.plugin_id
                    && deployment.params == simulation_deployment.params
                {
                    simulation_deployment.deployment_id = Some(deployment.id);
                    simulation_deployment.subscriptions = deployment.subscriptions.clone();
                    simulation_deployment.indicators = deployment.indicators.clone();
                }
            }
        }
    }

    async fn delete_deployments(&self, deployments: &[SimulationDeployment]) {
        for deployment in deployments {
            if let Some(deployment_id) = deployment.deployment_id {
                self.engine_client
                    .delete_deployment(deployment_id)
                    .await
                    .unwrap();
            }
        }
    }

    async fn check_active_orders(
        &self,
        active_orders: &mut Vec<Order>,
        tick: &Tick,
        positions: &mut Vec<SimulationPosition>,
        simulation_stats: &mut SimulationStats,
        logger: &mut Logger,
    ) {
        let mut completed_orders = Vec::new();
        for order in &mut *active_orders {
            match order.order_type {
                OrderType::Limit(price) => {
                    if self.check_limit_order(order, price, tick, positions, simulation_stats, logger).await {
                        completed_orders.push(order.id.clone());
                    }
                }
                OrderType::Market => {
                    if self.check_market_order(order, tick, positions, simulation_stats, logger).await {
                        completed_orders.push(order.id.clone());
                    }
                }
            }
        }
        for order in active_orders.iter_mut() {
            if completed_orders.contains(&order.id) {
                logger.log(format!("|---> Order fully processed: '{}'", order.id));

                order.status = OrderStatus::Completed;
                self.storage_client.save_order(order.clone()).await.unwrap();
            }
        }
        active_orders.retain(|order|
            order.status == OrderStatus::Created ||
                order.status == OrderStatus::InProgress);
    }

    async fn check_limit_order(
        &self,
        order: &mut Order,
        price: f64,
        tick: &Tick,
        positions: &mut Vec<SimulationPosition>,
        simulation_stats: &mut SimulationStats,
        logger: &mut Logger,
    ) -> bool {
        if order.status == OrderStatus::Created {
            match order.side {
                Side::Buy if tick.price <= price => {
                    logger.log(format!(
                        "|--> Execute limit order: {}, price: '{}'",
                        order.id, price
                    ));
                    self.execute_order(order, price, positions, logger).await;
                    order.side = change_side(order.side);
                }
                Side::Sell if tick.price >= price => {
                    logger.log(format!(
                        "|--> Execute limit order: {}, price: '{}'",
                        order.id, price
                    ));
                    self.execute_order(order, price, positions, logger).await;
                }
                _ => {}
            }
            false
        } else {
            self.check_sl_and_tp(order, tick, positions, simulation_stats, logger).await
        }
    }

    async fn check_market_order(
        &self,
        order: &mut Order,
        tick: &Tick,
        positions: &mut Vec<SimulationPosition>,
        simulation_stats: &mut SimulationStats,
        logger: &mut Logger,
    ) -> bool {
        if order.status == OrderStatus::Created {
            logger.log(format!(
                "|--> Execute market order: {}, price: '{}'",
                order.id, tick.price
            ));
            self.execute_order(order, tick.price, positions, logger)
                .await;
            false
        } else {
            self.check_sl_and_tp(order, tick, positions, simulation_stats, logger).await
        }
    }

    async fn check_sl_and_tp(
        &self,
        order: &mut Order,
        tick: &Tick,
        positions: &mut Vec<SimulationPosition>,
        simulation_stats: &mut SimulationStats,
        logger: &mut Logger,
    ) -> bool {
        let mut fully_completed = true;
        if let Some(stop_loss) = &order.stop_loss {
            let price = match stop_loss.order_px {
                OrderType::Limit(limit) => limit,
                OrderType::Market => stop_loss.trigger_px
            };
            if self.check_sl(order, price, tick, positions, logger).await {
                simulation_stats.add_sl();
                let size = match order.size {
                    Size::Target(size) => size,
                    Size::Source(size) => size,
                };
                let loss = (size / 100.) * ((order.avg_fill_price - order.avg_sl_price).abs() / (order.avg_sl_price / 100.));
                logger.log(format!(
                    "|X-> Execute SL '{}' for {} order: {}. Result: -{loss}",
                    price, order.side, order.id
                ));
                return true;
            } else {
                fully_completed = false;
            }
        }
        if let Some(take_profit) = &order.take_profit {
            let price = match take_profit.order_px {
                OrderType::Limit(limit) => limit,
                OrderType::Market => take_profit.trigger_px
            };
            if self.check_tp(order, price, tick, positions, logger).await {
                simulation_stats.add_tp();
                let size = match order.size {
                    Size::Target(size) => size,
                    Size::Source(size) => size,
                };
                let profit = (size / 100.) * ((order.avg_fill_price - order.avg_tp_price).abs() / (order.avg_tp_price / 100.));
                logger.log(format!(
                    "|X-> Execute TP '{}' for {} order: {}. Result: +{profit}",
                    price, order.side, order.id
                ));
                return true;
            } else {
                fully_completed = false;
            }
        }
        fully_completed
    }

    async fn check_sl(
        &self,
        order: &mut Order,
        price: f64,
        tick: &Tick,
        positions: &mut Vec<SimulationPosition>,
        logger: &mut Logger,
    ) -> bool {
        match order.side {
            Side::Buy if tick.price <= price => {
                self.execute_order(order, price, positions, logger).await;
                order.avg_sl_price = price;
                true
            }
            Side::Sell if tick.price >= price => {
                self.execute_order(order, price, positions, logger).await;
                order.avg_sl_price = price;
                true
            }
            _ => false,
        }
    }

    async fn check_tp(
        &self,
        order: &mut Order,
        price: f64,
        tick: &Tick,
        positions: &mut Vec<SimulationPosition>,
        logger: &mut Logger,
    ) -> bool {
        match order.side {
            Side::Buy if tick.price >= price => {
                self.execute_order(order, price, positions, logger).await;
                order.avg_tp_price = price;
                true
            }
            Side::Sell if tick.price <= price => {
                self.execute_order(order, price, positions, logger).await;
                order.avg_tp_price = price;
                true
            }
            _ => false,
        }
    }

    async fn execute_order(
        &self,
        order: &mut Order,
        quote: f64,
        positions: &mut Vec<SimulationPosition>,
        logger: &mut Logger,
    ) {
        let target_position_index = positions
            .iter()
            .position(|position| position.currency == order.pair.target);
        let source_position_index = positions
            .iter()
            .position(|position| position.currency == order.pair.source);

        if target_position_index.is_none() {
            positions.push(SimulationPosition {
                simulation_id: order.simulation_id.unwrap(),
                exchange: order.exchange,
                currency: order.pair.target,
                start: 0.0,
                end: 0.0,
                diff: 0.0,
                fees: 0.0,
            });
        }
        if source_position_index.is_none() {
            positions.push(SimulationPosition {
                simulation_id: order.simulation_id.unwrap(),
                exchange: order.exchange,
                currency: order.pair.source,
                start: 0.0,
                end: 0.0,
                diff: 0.0,
                fees: 0.0,
            });
        }

        let mut target_position = None;
        let mut source_position = None;

        positions.iter_mut().for_each(|position| {
            if position.currency == order.pair.target {
                target_position = Some(position);
            } else if position.currency == order.pair.source {
                source_position = Some(position);
            }
        });

        let is_sl_tp_execution = order.avg_fill_price != 0.;
        let mut side = order.side;
        if is_sl_tp_execution {
            side = change_side(side);
        } else {
            order.avg_fill_price = quote;
        }

        let fee_percent = get_fee_percent(order.exchange, order.market_type, side);
        let (target_size, source_size) = match order.size {
            Size::Target(size) => (size, size * quote),
            Size::Source(size) => (size / quote, size),
        };
        order.fee += calculate_fee_size(source_size, fee_percent);

        match side {
            Side::Buy => {
                self.update_positions(
                    target_size,
                    source_size,
                    fee_percent,
                    target_position,
                    source_position,
                    logger,
                )
                    .await;
            }
            Side::Sell => {
                self.update_positions(
                    source_size,
                    target_size,
                    fee_percent,
                    source_position,
                    target_position,
                    logger,
                )
                    .await;
            }
        }
        order.status = OrderStatus::InProgress;
        self.storage_client.save_order(order.clone()).await.unwrap();
    }

    async fn get_ticks(
        &self,
        logger: &mut Logger,
        simulation: &Simulation,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<Tick> {
        let mut ticks = Vec::new();
        let simulation_id = Some(simulation.id);
        for deployments in &simulation.deployments {
            let timeframe = deployments.timeframe;
            for instrument_id in &deployments.subscriptions {
                let sync_report = self
                    .storage_client
                    .sync(instrument_id, &[timeframe], from, Some(to))
                    .await
                    .unwrap();
                logger.log(format!(
                    "|> Sync candles for {}-{} from: {from}, to: {to}, report: {sync_report:?}",
                    instrument_id.pair.target, instrument_id.pair.source
                ));
                let candles = self
                    .storage_client
                    .get_candles(instrument_id, Some(timeframe), Some(from), Some(to), None)
                    .await
                    .unwrap();

                ticks = candles.iter()
                    .rev()
                    .flat_map(candle_to_ticks)
                    .map(|mut tick| {
                        tick.simulation_id = simulation_id;
                        tick
                    })
                    .collect()
            }
        }
        ticks = remove_redundancy(ticks);
        ticks
    }

    async fn update_positions(
        &self,
        target_size: f64,
        source_size: f64,
        fee_percent: f64,
        target_position: Option<&mut SimulationPosition>,
        source_position: Option<&mut SimulationPosition>,
        logger: &mut Logger,
    ) {
        let source_position = source_position.expect("No source asset to execute order");
        source_position.end -= source_size;
        self.storage_client
            .save_position(Position::from(source_position.clone()))
            .await
            .unwrap();
        logger.log(format!(
            "|--> Update position: {} {} '{} | -{}'",
            source_position.exchange, source_position.currency, source_position.end, source_size
        ));

        let target_position = target_position.expect("No target asset to execute order");
        let fee = calculate_fee_size(target_size, fee_percent);
        target_position.end += target_size - fee;
        target_position.fees += fee;
        self.storage_client
            .save_position(Position::from(target_position.clone()))
            .await
            .unwrap();
        logger.log(format!(
            "|--> Update position: {} {} '{} | +{} | -{}'",
            target_position.exchange,
            target_position.currency,
            target_position.end,
            target_size,
            fee
        ));
    }
}

fn candle_to_ticks(candle: &Candle) -> Vec<Tick> {
    let open_tick = Tick::new(
        None,
        candle.timestamp,
        candle.instrument_id.clone(),
        candle.open_price,
    );
    let lowest_tick = Tick::new(
        None,
        candle.timestamp,
        candle.instrument_id.clone(),
        candle.lowest_price,
    );
    let highest_tick = Tick::new(
        None,
        candle.timestamp,
        candle.instrument_id.clone(),
        candle.highest_price,
    );
    let close_tick = Tick::new(
        None,
        candle.timestamp,
        candle.instrument_id.clone(),
        candle.close_price,
    );
    vec![open_tick, lowest_tick, highest_tick, close_tick]
}

fn remove_redundancy(ticks: Vec<Tick>) -> Vec<Tick> {
    let mut optimized_ticks = Vec::new();
    let mut tail_iter = ticks.iter();
    for next_tick in ticks.iter().skip(1) {
        let previous_tick = tail_iter.next().unwrap();
        if next_tick.price != previous_tick.price {
            optimized_ticks.push(previous_tick.clone());
        }
    }
    optimized_ticks.push(tail_iter.next().unwrap().clone());
    debug!("Remove redundancy ticks, all: '{}', optimized: '{}'", ticks.len(), optimized_ticks.len());
    optimized_ticks
}

fn change_side(side: Side) -> Side {
    match side {
        Side::Buy => Side::Sell,
        Side::Sell => Side::Buy,
    }
}

#[derive(PartialEq, Debug)]
struct Price(f64);

impl Eq for Price {}

fn get_fee_percent(exchange: Exchange, market_type: OrderMarketType, side: Side) -> f64 {
    match exchange {
        Exchange::OKX => match market_type {
            OrderMarketType::Spot => match side {
                Side::Buy => 0.08,
                Side::Sell => 0.1,
            },
            OrderMarketType::Margin(_) => match side {
                Side::Buy => 0.1,
                Side::Sell => 0.1,
            },
        },
        Exchange::BYBIT => match market_type {
            OrderMarketType::Spot => match side {
                Side::Buy => 0.1,
                Side::Sell => 0.1,
            },
            OrderMarketType::Margin(_) => match side {
                Side::Buy => 0.1,
                Side::Sell => 0.1,
            },
        }
    }
}

fn calculate_fee_size(size: f64, fee_percent: f64) -> f64 {
    size * fee_percent / 100.0
}

fn convert_to_create_deployment_dto(
    value: SimulationDeployment,
    simulation_id: Uuid,
) -> NewDeployment {
    NewDeployment {
        simulation_id: Some(simulation_id),
        state_id: None,
        plugin_id: value.plugin_id,
        params: value.params,
    }
}

#[allow(unused)]
enum CurrencyConversion {
    ToTarget,
    ToSource,
}

#[derive(Default)]
struct SimulationStats {
    sl_count: u64,
    tp_count: u64,
    current_sl_streak: u64,
    current_tp_streak: u64,
    max_sl_streak: u64,
    max_tp_streak: u64,
}


impl SimulationStats {
    pub fn add_sl(&mut self) {
        self.sl_count += 1;
        self.current_sl_streak += 1;
        if self.current_sl_streak > self.max_sl_streak {
            self.max_sl_streak = self.current_sl_streak;
        }
        self.current_tp_streak = 0;
    }

    pub fn add_tp(&mut self) {
        self.tp_count += 1;
        self.current_tp_streak += 1;
        if self.current_tp_streak > self.max_tp_streak {
            self.max_tp_streak = self.current_tp_streak;
        }
        self.current_sl_streak = 0;
    }

    pub fn sl_percent(&self) -> f64 {
        let sl_tp_count = self.sl_count + self.tp_count;
        if sl_tp_count > 0 {
            let percent = sl_tp_count as f64 / 100.;
            self.sl_count as f64 / percent
        } else { 0. }
    }

    pub fn tp_percent(&self) -> f64 {
        let sl_tp_count = self.sl_count + self.tp_count;
        if sl_tp_count > 0 {
            let percent = sl_tp_count as f64 / 100.;
            self.tp_count as f64 / percent
        } else { 0. }
    }
}
