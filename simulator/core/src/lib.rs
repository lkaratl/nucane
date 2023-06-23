use std::fs;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use domain_model::{Action, Currency, CurrencyPair, Exchange, InstrumentId, MarketType, Order, OrderActionType, OrderMarketType, OrderStatus, OrderType, Position, Side, Simulation, SimulationDeployment, SimulationPosition, Tick, Timeframe};
use engine_rest_api::dto::{CreateDeploymentDto};
use engine_rest_client::EngineClient;
use interactor_rest_client::InteractorClient;
use storage_rest_client::StorageClient;
use synapse::SynapseSend;

pub struct SimulationService {
    engine_client: EngineClient,
    storage_client: StorageClient,
    interactor_client: InteractorClient,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimulationReport {
    pub simulation_id: Uuid,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub deployments: Vec<SimulationDeployment>,
    pub ticks: usize,
    pub actions: u16,
    pub profit: f64,
    pub profit_clear: f64,
    pub fees: f64,
    pub assets: Vec<SimulationPosition>,
    pub active_orders: Vec<Order>,
}

impl SimulationService {
    pub fn new(strategy_engine_client: EngineClient,
               storage_client: StorageClient,
                interactor_client: InteractorClient
    ) -> Self {
        SimulationService {
            engine_client: strategy_engine_client,
            storage_client,
            interactor_client,
        }
    }

    pub async fn run(&self, simulation: Simulation) -> SimulationReport {
        let mut logger = Logger::new(simulation.id);
        let report = self.run_simulation_with_logger(simulation, &mut logger).await;
        logger.save();
        report
    }

    async fn run_simulation_with_logger(&self, mut simulation: Simulation, logger: &mut Logger) -> SimulationReport {
        logger.log(format!("Start simulation: '{:?}'", simulation));
        self.create_positions(&simulation).await;
        self.create_deployments(&mut simulation).await;

        let mut batch_start = simulation.start;
        let mut batch_end = simulation.start;

        while batch_end != simulation.end {
            let new_batch_end = batch_end + Duration::days(7);
            batch_end = if new_batch_end < simulation.end {
                new_batch_end
            } else { simulation.end };

            self.run_simulation_batch(logger, &mut simulation, batch_start, batch_end).await;

            batch_start += Duration::days(7);
        }
        self.delete_deployments(&simulation.deployments).await;

        let report = self.build_report(simulation).await;
        logger.log(format!("{report:?}"));
        report
    }

    async fn run_simulation_batch(&self, logger: &mut Logger, mut simulation: &mut Simulation, batch_start: DateTime<Utc>, batch_end: DateTime<Utc>) {
        debug!("Batch processing from start: {batch_start} to end: {batch_end}");
        let ticks = self.get_ticks(logger, simulation, batch_start, batch_end).await;
        let positions = &mut simulation.positions;
        let active_orders = &mut simulation.active_orders;
        debug!("Ticks len: {}", ticks.len());
        simulation.ticks_len += ticks.len();
        for tick in &ticks {
            info!("| Tick: {} '{}' {}-{}= '{}'", tick.instrument_id.exchange, tick.timestamp,
            tick.instrument_id.pair.target,
            tick.instrument_id.pair.source, tick.price);
            let actions = self.engine_client.create_actions(tick).await.unwrap();
            for action in &actions {
                logger.log(format!("|* Action: {:?} \n   for tick: {} '{}' {}-{}= '{}'", action, tick.instrument_id.exchange, tick.timestamp, tick.instrument_id.pair.target, tick.instrument_id.pair.source, tick.price));
                simulation.actions_count += 1;
                self.execute_action(action, active_orders, logger).await;
            }
            self.check_active_orders(active_orders, tick, positions, logger).await;
        }
    }

    async fn build_report(&self, simulation: Simulation) -> SimulationReport {
        let mut positions = simulation.positions;
        positions.iter_mut()
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
        }
    }

    async fn calculate_profit(&self, positions_diff: &[SimulationPosition], timestamp: DateTime<Utc>) -> f64 {
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
            result += self.convert_currency(&instrument_id, timestamp, position_diff.diff, CurrencyConversion::ToSource).await;
        }
        result
    }

    async fn calculate_fees(&self, positions_diff: &[SimulationPosition], timestamp: DateTime<Utc>) -> f64 {
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
            result += self.convert_currency(&instrument_id, timestamp, position_diff.fees, CurrencyConversion::ToSource).await;
        }
        result
    }

    async fn convert_currency(&self, instrument_id: &InstrumentId, timestamp: DateTime<Utc>, value: f64, conversion_type: CurrencyConversion) -> f64 {
        if instrument_id.pair.source == instrument_id.pair.target { return value; }
        let price = self.interactor_client.get_price(instrument_id, Some(timestamp))
            .await
            .expect("No find price for currency conversion");
        match conversion_type {
            CurrencyConversion::ToTarget => value / price,
            CurrencyConversion::ToSource => value * price
        }
    }

    async fn execute_action(&self, action: &Action, active_orders: &mut Vec<Order>, logger: &mut Logger) {
        match action {
            Action::OrderAction(order_action) => {
                match &order_action.order {
                    OrderActionType::CreateOrder(create_order) => {
                        let order = Order {
                            id: create_order.id.clone(),
                            timestamp: Utc::now(),
                            simulation_id: order_action.simulation_id,
                            exchange: order_action.exchange,
                            status: OrderStatus::InProgress,
                            market_type: create_order.market_type,
                            order_type: create_order.order_type,
                            pair: create_order.pair,
                            side: create_order.side,
                            size: create_order.size,
                            avg_price: 0.0,
                        };
                        synapse::writer().send(&order);
                        logger.log(format!("|-> Place Order: {} {:?} {:?} '{}-{}' {} '{}' {}", order.exchange, order.market_type, order.order_type, order.pair.target, order.pair.source, order.side, order.size, order.id));
                        active_orders.push(order);
                    }
                    OrderActionType::PatchOrder => unimplemented!(),
                    OrderActionType::CancelOrder => unimplemented!(),
                }
            }
        }
    }

    async fn create_positions(&self, simulation: &Simulation) {
        simulation.positions.clone()
            .iter()
            .for_each(|position| {
                synapse::writer().send(&Position::from(position.clone()));
            });
    }

    async fn create_deployments(&self, simulation: &mut Simulation) {
        let create_deployments = simulation.deployments.iter()
            .map(|strategy| convert_to_create_deployment_dto(strategy.clone(), simulation.id))
            .collect();
        let created_deployments = self.engine_client.create_deployment(create_deployments)
            .await
            .unwrap();
        for deployment in created_deployments {
            for simulation_deployment in simulation.deployments.iter_mut() {
                if deployment.strategy_name == simulation_deployment.strategy_name &&
                    deployment.strategy_version == simulation_deployment.strategy_version &&
                    deployment.params == simulation_deployment.params {
                    simulation_deployment.deployment_id = Some(deployment.id);
                    simulation_deployment.subscriptions = deployment.subscriptions.clone();
                }
            }
        }
    }

    async fn delete_deployments(&self, deployments: &[SimulationDeployment]) {
        for deployment in deployments {
            if let Some(deployment_id) = deployment.deployment_id {
                self.engine_client.remove_deployment(deployment_id)
                    .await
                    .unwrap();
            }
        }
    }

    async fn check_active_orders(&self, active_orders: &mut Vec<Order>, tick: &Tick, positions: &mut Vec<SimulationPosition>, logger: &mut Logger) {
        let mut executed_orders = Vec::new();
        for order in &mut *active_orders {
            match order.order_type {
                OrderType::Limit(price) => {
                    match order.side {
                        Side::Buy if tick.price <= price => {
                            self.execute_order(order, price, positions, logger).await;
                            executed_orders.push(order.id.clone());
                        }
                        Side::Sell if tick.price >= price => {
                            self.execute_order(order, price, positions, logger).await;
                            executed_orders.push(order.id.clone());
                        }
                        _ => {}
                    }
                }
                OrderType::Market => {
                    self.execute_order(order, tick.price, positions, logger).await;
                    executed_orders.push(order.id.clone());
                }
            }
        }
        active_orders.retain(|order| if executed_orders.contains(&order.id) {
            logger.log(format!("|--> Order successfully processed: '{}'", order.id));
            false
        } else {
            true
        });
    }

    // todo refactor
    async fn execute_order(&self, order: &mut Order, quote: f64, positions: &mut Vec<SimulationPosition>, logger: &mut Logger) {
        logger.log(format!("|--> Execute Order: {}, quote: '{}'", order.id, quote));
        let mut target_position = None;
        let mut source_position = None;
        positions.iter_mut()
            .for_each(|position| if position.currency == order.pair.target {
                target_position = Some(position);
            } else if position.currency == order.pair.source {
                source_position = Some(position);
            });
        match order.side {
            Side::Buy => {
                let fee = get_fee(order.exchange, order.market_type, order.side, order.size);
                let source_size = order.size;
                let source_position = source_position.expect("No asset to execute order");
                source_position.end -= source_size + fee;
                source_position.fees += fee;
                synapse::writer().send(&Position::from(source_position.clone()));
                logger.log(format!("|--> Update position: {} {} '{} | -{} | -{}'", source_position.exchange, source_position.currency, source_position.end, source_size, fee));

                let target_size = source_size / quote;
                if let Some(target_position) = target_position {
                    target_position.end += target_size;
                    synapse::writer().send(&Position::from(target_position.clone()));
                    logger.log(format!("|--> Update position: {} {} '{} | +{}'", target_position.exchange, target_position.currency, target_position.end, target_size));
                } else {
                    let new_position = SimulationPosition {
                        simulation_id: order.simulation_id.unwrap(),
                        exchange: order.exchange,
                        currency: order.pair.target,
                        start: 0.0,
                        end: target_size,
                        diff: target_size,
                        fees: 0.0,
                    };
                    synapse::writer().send(&Position::from(new_position.clone()));
                    logger.log(format!("|--> New position: {} {} '{}'", new_position.exchange, new_position.currency, new_position.end));
                    positions.push(new_position);
                }
            }
            Side::Sell => {
                let target_size = order.size;
                let fee = get_fee(order.exchange, order.market_type, order.side, target_size);
                let target_position = target_position.expect("No asset to execute order");
                target_position.end -= target_size + fee;
                target_position.fees += fee;
                synapse::writer().send(&Position::from(target_position.clone()));
                logger.log(format!("|--> Update position: {} {} '{} | -{} | -{}'", target_position.exchange, target_position.currency, target_position.end, target_size, fee));

                let source_size = target_size * quote;
                if let Some(source_position) = source_position {
                    source_position.end += source_size;
                    synapse::writer().send(&Position::from(source_position.clone()));
                    logger.log(format!("|--> Update position: {} {} '{} | +{}'", source_position.exchange, source_position.currency, source_position.end, source_size));
                } else {
                    let new_position = SimulationPosition {
                        simulation_id: order.simulation_id.unwrap(),
                        exchange: order.exchange,
                        currency: order.pair.target,
                        start: 0.0,
                        end: source_size,
                        diff: source_size,
                        fees: 0.0,
                    };
                    synapse::writer().send(&Position::from(new_position.clone()));
                    logger.log(format!("|--> New position: {} {} '{}'", new_position.exchange, new_position.currency, new_position.end));
                    positions.push(new_position);
                }
            }
        }
        order.status = OrderStatus::Completed;
        synapse::writer().send(order);
    }

    async fn get_ticks(&self, logger: &mut Logger, simulation: &Simulation, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<Tick> {
        let mut ticks = Vec::new();
        let simulation_id = Some(simulation.id);
        for deployments in &simulation.deployments {
            let timeframe = deployments.timeframe;
            for instrument_id in &deployments.subscriptions {
                let sync_report = self.storage_client.sync_candles(instrument_id, &[timeframe], from, Some(to)).await.unwrap();
                logger.log(format!("|> Sync candles for {}-{} from: {from}, to: {to}, report: {sync_report:?}", instrument_id.pair.target, instrument_id.pair.source));
                let candles = self.storage_client.get_candles(instrument_id,
                                                              Some(timeframe),
                                                              Some(from),
                                                              Some(to),
                                                              None)
                    .await
                    .unwrap();
                let mut first_iter = candles.iter();
                for next_candle in candles.iter().skip(1) {
                    let previous_candle = first_iter.next().unwrap();
                    if Price(previous_candle.open_price) != Price(next_candle.open_price) {
                        ticks.push(Tick {
                            id: Uuid::new_v4(),
                            simulation_id,
                            timestamp: next_candle.timestamp,
                            instrument_id: instrument_id.clone(),
                            price: next_candle.open_price,
                        });
                    }
                }
            }
        }
        ticks.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        ticks
    }
}

#[derive(PartialEq, Debug)]
struct Price(f64);

impl Eq for Price {}

struct Logger {
    simulation_id: Uuid,
    file_content: Vec<String>,
}

impl Logger {
    fn new(simulation_id: Uuid) -> Self {
        Self {
            simulation_id,
            file_content: Vec::new(),
        }
    }
    fn log(&mut self, message: String) {
        info!(message);
        self.file_content.push(message);
    }

    fn save(&self) {
        fs::write(format!("./simulation-{}.log", self.simulation_id), self.file_content.join("\n"))
            .expect("Error during saving simulation log")
    }
}

fn get_fee(exchange: Exchange, market_type: OrderMarketType, side: Side, size: f64) -> f64 {
    match exchange {
        Exchange::OKX => {
            match market_type {
                OrderMarketType::Spot => {
                    match side {
                        Side::Buy => size * 0.08 / 100.0,
                        Side::Sell => size * 0.1 / 100.0
                    }
                }
                OrderMarketType::Margin(_) => {
                    match side {
                        Side::Buy => size * 0.02 / 100.0,
                        Side::Sell => size * 0.05 / 100.0
                    }
                }
            }
        }
    }
}

fn convert_to_create_deployment_dto(value: SimulationDeployment, simulation_id: Uuid) -> CreateDeploymentDto {
    CreateDeploymentDto {
        simulation_id: Some(simulation_id),
        strategy_name: value.strategy_name,
        strategy_version: value.strategy_version,
        params: value.params,
    }
}

enum CurrencyConversion {
    ToTarget,
    ToSource,
}
