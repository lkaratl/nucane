use std::collections::HashMap;
use std::future::Future;
use std::str::FromStr;
use chrono::Utc;
use serde_json::from_value;
use tracing::{error, info, trace};
use uuid::Uuid;
use domain_model::{Candle, CandleStatus, Currency, CurrencyPair, Exchange, InstrumentId, MarginMode, MarketType, Order, OrderMarketType, OrderStatus, OrderType, Position, Side, Size, Tick, Timeframe, Trigger};
use eac::{enums};
use eac::enums::{OrdState, OrdType, TdMode};
use eac::rest::{Account, CandleResponse, MarkPriceResponse, OrderDetailsResponse};
use eac::websocket::{Channel, Message};

const TICK_PRICE_DEVIATION_MULTIPLIER: f64 = 1000.0;
const TICK_PRICE_THRESHOLD: f64 = 2.0;

pub fn on_tick<C: Fn(Tick) + Send + 'static>(callback: C, currency_pair: CurrencyPair, market_type: MarketType) -> impl FnMut(Message) {
    let mut deviation_percent = 1f64;
    move |message| {
        match message {
            Message::Data { arg: _, mut data, .. } => {
                trace!("Retrieved massage with raw payload: {:?}", &data);
                let data = data.pop().unwrap();
                let mark_price: MarkPriceResponse = from_value(data).unwrap();

                let price = mark_price.mark_px;
                let deviation = price / deviation_percent - TICK_PRICE_DEVIATION_MULTIPLIER;
                if !(TICK_PRICE_THRESHOLD * -1.0..=TICK_PRICE_THRESHOLD).contains(&deviation) {
                    deviation_percent = price / TICK_PRICE_DEVIATION_MULTIPLIER;
                    let tick = Tick {
                        id: Uuid::new_v4(),
                        simulation_id: None,
                        timestamp: mark_price.ts,
                        instrument_id: InstrumentId {
                            exchange: Exchange::OKX,
                            market_type,
                            pair: currency_pair,
                        },
                        price,
                    };
                    callback(tick);
                }
            }
            Message::Error { code, msg, .. } => error!("Error {}: {}", code, msg),
            _ => {}
        }
    }
}

pub fn on_position<C: Fn(Position) + Send + 'static>(callback: C) -> impl FnMut(Message) {
    let mut positions = HashMap::new();
    move |message| {
        match message {
            Message::Data { arg: _, data, .. } => {
                trace!("Retrieved massage with raw payload: {:?}", &data);
                for item in data {
                    let account: Account = from_value(item).unwrap();
                    for asset in account.details {
                        let previous_ccy_amount = positions.entry(asset.ccy.clone()).or_insert(0.0);
                        if *previous_ccy_amount != asset.avail_bal {
                            let currency = Currency::from_str(&asset.ccy).unwrap();
                            let size = asset.avail_bal;
                            let side = if size < 0.0 { Side::Sell } else { Side::Buy };
                            let position = Position::new(None, Exchange::OKX, currency, side, size);
                            callback(position);
                            positions.insert(asset.ccy, asset.avail_bal);
                        }
                    }
                }
            }
            Message::Error { code, msg, .. } => error!("Error {}: {}", code, msg),
            _ => {}
        }
    }
}

pub fn on_order<C: Fn(Order) + Send + 'static>(callback: C) -> impl FnMut(Message) {
    move |message: Message| {
        match message {
            Message::Data { arg: _, data, .. } => {
                info!("Retrieved massage with raw payload: {:?}", &data);
                for item in data {
                    let order_details: OrderDetailsResponse = from_value(item).unwrap();
                    if !order_details.cl_ord_id.is_empty() {
                        let status = match order_details.state {
                            OrdState::Canceled => OrderStatus::Canceled,
                            OrdState::Live => OrderStatus::InProgress,
                            OrdState::PartiallyFilled => OrderStatus::InProgress,
                            OrdState::Filled => OrderStatus::Completed
                        };
                        let pair = {
                            let mut inst_id = order_details.inst_id.split('-');
                            CurrencyPair {
                                target: Currency::from_str(inst_id.next().unwrap()).unwrap(),
                                source: Currency::from_str(inst_id.next().unwrap()).unwrap(),
                            }
                        };
                        let market_type = {
                            match order_details.td_mode {
                                TdMode::Cross => OrderMarketType::Margin(MarginMode::Cross),
                                TdMode::Isolated => OrderMarketType::Margin(MarginMode::Isolated),
                                TdMode::Cash => OrderMarketType::Spot
                            }
                        };
                        let side = match order_details.side {
                            enums::Side::Buy => Side::Buy,
                            enums::Side::Sell => Side::Sell
                        };
                        let order_type = match order_details.ord_type {
                            OrdType::Market => OrderType::Market,
                            OrdType::Limit => OrderType::Limit(order_details.px.unwrap()),
                            order_type => panic!("Unsupported order type: {order_type:?}")
                        };
                        let size = match order_details.ord_type {
                            OrdType::Market => match order_details.tgt_ccy.as_str() {
                                "quote_ccy" => Size::Source(order_details.sz),
                                "base_ccy" => Size::Target(order_details.sz),
                                _ => panic!("Empty target currency")
                            }
                            OrdType::Limit => Size::Target(order_details.sz),
                            order_type => panic!("Unsupported order type: {order_type:?}")
                        };
                        let stop_loss = if order_details.sl_trigger_px.is_some() &&
                            order_details.sl_ord_px.is_some() {
                            Trigger::new(order_details.sl_trigger_px.unwrap(), order_details.sl_ord_px.unwrap())
                        } else {
                            None
                        };
                        let take_profit = if order_details.tp_trigger_px.is_some() &&
                            order_details.tp_ord_px.is_some() {
                            Trigger::new(order_details.tp_trigger_px.unwrap(), order_details.tp_ord_px.unwrap())
                        } else {
                            None
                        };
                        let order = Order {
                            id: order_details.cl_ord_id,
                            timestamp: Utc::now(),
                            simulation_id: None,
                            status,
                            exchange: Exchange::OKX,
                            pair,
                            market_type,
                            order_type,
                            side,
                            size,
                            avg_price: order_details.avg_px,
                            stop_loss,
                            take_profit,
                        };
                        callback(order);
                    }
                }
            }
            Message::Error { code, msg, .. } => error!("Error {}: {}", code, msg),
            _ => {}
        }
    }
}

pub fn on_candles<C: Fn(Candle) + Send + 'static>(callback: C, currency_pair: CurrencyPair, market_type: MarketType) -> impl FnMut(Message) {
    Box::new(move |message| {
        match message {
            Message::Data { arg, mut data, .. } => {
                trace!("Retrieved massage with raw payload: {:?}", &data);
                let data = data.pop().unwrap();
                let candle_message: CandleResponse = from_value(data).unwrap();
                let timeframe = match arg {
                    Channel::Candle1M { .. } => Timeframe::OneM,
                    Channel::Candle5M { .. } => Timeframe::FiveM,
                    Channel::Candle15M { .. } => Timeframe::FifteenM,
                    Channel::Candle30M { .. } => Timeframe::ThirtyM,
                    Channel::Candle1H { .. } => Timeframe::OneH,
                    Channel::Candle2H { .. } => Timeframe::TwoH,
                    Channel::Candle4H { .. } => Timeframe::FourH,
                    Channel::Candle1D { .. } => Timeframe::OneD,
                    channel => panic!("Error during timeframe parsing for candle, unexpected channel: {channel:?}")
                };
                let id = format!("{}_{}_{}_{}_{}_{}", Exchange::OKX, market_type, currency_pair.target, currency_pair.source, timeframe, candle_message.0.timestamp_millis());
                let status = match candle_message.8.as_str() {
                    "0" => CandleStatus::Open,
                    "1" => CandleStatus::Close,
                    status => panic!("Error during candle status parsing, unexpected status: {status}")
                };
                let instrument_id = InstrumentId {
                    exchange: Exchange::OKX,
                    market_type,
                    pair: currency_pair,
                };
                let candle = Candle {
                    id,
                    status,
                    instrument_id,
                    timestamp: candle_message.0,
                    timeframe,
                    open_price: candle_message.1,
                    highest_price: candle_message.2,
                    lowest_price: candle_message.3,
                    close_price: candle_message.4,
                    target_volume: candle_message.6,
                    source_volume: candle_message.7,
                };
                callback(candle);
            }
            Message::Error { code, msg, .. } => error!("Error {}: {}", code, msg),
            _ => {}
        }
    })
}
