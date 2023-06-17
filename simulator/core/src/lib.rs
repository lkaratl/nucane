use std::collections::HashMap;
use std::fs;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use domain_model::{Action, Currency, CurrencyPair, Exchange, InstrumentId, MarketType, Order, OrderActionType, OrderMarketType, OrderStatus, OrderType, Position, Side, Simulation, SimulationPosition, Tick, Timeframe};
use engine_rest_client::EngineClient;
use storage_rest_client::StorageClient;
use synapse::SynapseSend;

pub struct SimulationService {
    engine_client: EngineClient,
    storage_client: StorageClient,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimulationReport {
    simulation_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    strategy_id: String,
    strategy_version: String,
    strategy_params: HashMap<String, String>,
    ticks: usize,
    actions: u16,
    profit: f64,
    profit_clear: f64,
    fees: f64,
    assets: Vec<SimulationPosition>,
    active_orders: Vec<Order>,
}

impl SimulationService {
    pub fn new(strategy_engine_client: EngineClient,
               storage_client: StorageClient,
    ) -> Self {
        SimulationService {
            engine_client: strategy_engine_client,
            storage_client,
        }
    }

    pub async fn run(&self, simulation: Simulation) -> SimulationReport {
        let mut logger = Logger::new(simulation.id);
        logger.log(format!("Start simulation: '{}'", simulation.id));
        synapse::writer().send(&simulation);
        let mut positions = simulation.positions.clone();
        positions.iter()
            .for_each(|position| {
                logger.log(format!("| Initial position: {}-{}= '{}'", position.exchange, position.currency, position.end));
                synapse::writer().send(&Position::from(position.clone()));
            });
        let deployment = self.engine_client.create_deployment(
            Some(simulation.id),
            &simulation.strategy_id,
            &simulation.strategy_version,
            simulation.params.clone())
            .await
            .unwrap();

        let mut ticks_len = 0;
        let mut active_orders = Vec::new();
        let mut actions_count = 0;

        let mut batch_start = simulation.start;
        let mut batch_end = simulation.start;

        while batch_end != simulation.end {
            let new_batch_end = batch_end + Duration::days(7);
            batch_end = if new_batch_end < simulation.end {
                new_batch_end
            } else { simulation.end };
            debug!("Batch processing from start: {batch_start} to end: {batch_end}");

            let ticks = self.get_ticks(simulation.id, batch_start, batch_end, &deployment.subscriptions).await;
            debug!("Ticks len: {}", ticks.len());
            ticks_len += ticks.len();
            for tick in &ticks {
                // info!("| Tick: {} '{}' {}-{}= '{}'", tick.instrument_id.exchange, tick.timestamp, tick.instrument_id.pair.target, tick.instrument_id.pair.source, tick.price);
                let actions = self.engine_client.create_actions(tick).await.unwrap();
                for action in &actions {
                    logger.log(format!("|* Action: {:?} \n   for tick: {} '{}' {}-{}= '{}'", action, tick.instrument_id.exchange, tick.timestamp, tick.instrument_id.pair.target, tick.instrument_id.pair.source, tick.price));
                    actions_count += 1;
                    self.execute_action(action, &mut active_orders, &mut logger).await;
                }
                self.check_active_orders(&mut active_orders, tick, &mut positions, &mut logger).await;
            }

            batch_start += Duration::days(7);
        }

        positions.iter_mut()
            .for_each(|position| position.diff = position.end - position.start);

        let profit = self.calculate_profit(&positions, simulation.end).await;
        let profit_clear = self.calculate_profit(&positions, simulation.start).await;
        let fees = self.calculate_fees(&positions, simulation.end).await;

        self.engine_client.remove_deployment(deployment.id)
            .await
            .unwrap();
        let report = SimulationReport {
            simulation_id: simulation.id,
            start: simulation.start,
            end: simulation.end,
            strategy_id: simulation.strategy_id,
            strategy_version: simulation.strategy_version,
            strategy_params: simulation.params,
            ticks: ticks_len,
            actions: actions_count,
            profit,
            profit_clear,
            assets: positions,
            fees,
            active_orders,
        };
        logger.log(format!("{report:?}"));
        logger.save();
        report
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
        let from_timestamp = timestamp - Duration::minutes(1);
        let to_timestamp = timestamp - Duration::minutes(1);
        let candles = self.storage_client.get_candles(instrument_id, Some(Timeframe::OneS), Some(from_timestamp), Some(to_timestamp), Some(1))
            .await
            .expect("No find candle for currency conversion");
        let candle = candles
            .first()
            .expect("No find candle for currency conversion");
        match conversion_type {
            CurrencyConversion::ToTarget => value / candle.close_price,
            CurrencyConversion::ToSource => value * candle.close_price
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

    async fn get_ticks(&self, simulation_id: Uuid, start: DateTime<Utc>, end: DateTime<Utc>, instrument_ids: &Vec<InstrumentId>) -> Vec<Tick> {
        let mut ticks = Vec::new();
        let simulation_id = Some(simulation_id);
        for instrument_id in instrument_ids {
            let candles = self.storage_client.get_candles(instrument_id,
                                                          Some(Timeframe::OneS),
                                                          Some(start),
                                                          Some(end),
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

enum CurrencyConversion {
    ToTarget,
    ToSource,
}
