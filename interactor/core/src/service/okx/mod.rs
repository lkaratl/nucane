use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crossbeam_channel::{Sender};
use serde_json::from_value;
use tracing::{debug, error, info, trace};
use uuid::Uuid;

use domain_model::{Candle, CandleStatus, CreateOrder, Currency, CurrencyPair, Exchange, InstrumentId, MarginMode, MarketType, Order, OrderAction, OrderMarketType, OrderStatus, OrderType, Position, Side, Tick, Timeframe};
use eac::enums;
use eac::enums::{InstType, OrdState, OrdType, TdMode};
use eac::rest::{Account, CandleResponse, CandlesHistoryRequest, MarkPriceResponse, OkExRest, OrderDetailsResponse, PlaceOrderRequest};
use eac::websocket::{Channel, Command, Message, OkExWebsocket};
use interactor_config::CONFIG;

use crate::service::Service;

pub struct OKXService {
    is_demo: bool,
    api_key: String,
    api_secret: String,
    api_passphrase: String,
    ws_url: String,
    sockets: HashMap<String, OkExWebsocket>,
    rest_client: OkExRest,
}

impl OKXService {
    pub fn new() -> Self {
        let is_demo = CONFIG.eac.demo;
        let http_url = &CONFIG.eac.exchanges.okx.http_url;
        let ws_url = &CONFIG.eac.exchanges.okx.ws_url;
        let api_key = &CONFIG.eac.exchanges.okx.auth.api_key;
        let api_secret = &CONFIG.eac.exchanges.okx.auth.api_secret;
        let api_passphrase = &CONFIG.eac.exchanges.okx.auth.api_passphrase;

        let rest_client = OkExRest::with_credential(http_url, is_demo, api_key, api_secret, api_passphrase);
        Self {
            is_demo,
            api_key: api_key.to_owned(),
            api_secret: api_secret.to_owned(),
            api_passphrase: api_passphrase.to_owned(),
            ws_url: ws_url.to_owned(),
            sockets: HashMap::new(),
            rest_client,
        }
    }
}

#[async_trait]
impl Service for OKXService {
    // todo maybe better don't create client for each subscription and use one thread to handle all messages
    fn subscribe_ticks<T: Fn(Tick) + Send + 'static>(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType, callback: T) {
        let mut inst_id = format!("{}-{}", currency_pair.target, currency_pair.source);
        if !MarketType::Spot.eq(market_type) {
            inst_id = format!("{}-{}", inst_id, market_type);
        }
        let entry = self.sockets.entry(format!("mark-price-{}", &inst_id));
        entry.or_insert({
            let mut client = OkExWebsocket::public(false, &self.ws_url).unwrap(); // todo use demo flag from self
            client.send(Command::subscribe(vec![Channel::MarkPrice {
                inst_id,
            }])).unwrap();
            let target = currency_pair.target;
            let source = currency_pair.source;
            let market_type = *market_type;
            let mut price = 0f64;
            client.handle_message(move |message| {
                match message.unwrap() {
                    Message::Data { arg: _, mut data, .. } => {
                        trace!("Retrieved massage with raw payload: {:?}", &data);
                        let data = data.pop().unwrap();
                        let mark_price: MarkPriceResponse = from_value(data).unwrap();
                        if price == mark_price.mark_px {
                            return;
                        } else {
                            price = mark_price.mark_px;
                        }
                        let tick = Tick {
                            id: Uuid::new_v4(),
                            simulation_id: None,
                            timestamp: mark_price.ts,
                            instrument_id: InstrumentId {
                                exchange: Exchange::OKX,
                                market_type,
                                pair: CurrencyPair {
                                    target,
                                    source,
                                },
                            },
                            price: mark_price.mark_px,
                        };
                        callback(tick);
                    }
                    Message::Error { code, msg, .. } => error!("Error {}: {}", code, msg),
                    _ => {}
                }
            });
            client
        });
    }

    fn unsubscribe_ticks(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        debug!("Remove socket for ticks");
        let mut socket_id = format!("mark-price-{}-{}", currency_pair.target, currency_pair.source);
        if !MarketType::Spot.eq(market_type) {
            socket_id = format!("{}-{}", socket_id, market_type);
        }
        self.sockets.retain(|key, _| !socket_id.eq(key));
    }

    fn subscribe_candles<T: Fn(Candle) + Send + 'static>(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType, callback: T) {
        let mut inst_id = format!("{}-{}", currency_pair.target, currency_pair.source);
        if !MarketType::Spot.eq(market_type) {
            inst_id = format!("{}-{}", inst_id, market_type);
        }
        let entry = self.sockets.entry(format!("candles-{}", &inst_id));
        entry.or_insert({
            let mut client = OkExWebsocket::business(self.is_demo, &self.ws_url).unwrap();
            let subscribe_command = Command::subscribe(vec![
                Channel::candle_1m(&inst_id),
                Channel::candle_5m(&inst_id),
                Channel::candle_15m(&inst_id),
                Channel::candle_30m(&inst_id),
                Channel::candle_1h(&inst_id),
                Channel::candle_2h(&inst_id),
                Channel::candle_4h(&inst_id),
                Channel::candle_1d(&inst_id),
            ]);
            client.send(subscribe_command).unwrap();
            let target = currency_pair.target;
            let source = currency_pair.source;
            let market_type = *market_type;
            client.handle_message(move |message| {
                match message.unwrap() {
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
                        let id = format!("{}_{}_{}_{}_{}_{}", Exchange::OKX, market_type, target, source, timeframe, candle_message.0.timestamp_millis());
                        let status = match candle_message.8.as_str() {
                            "0" => CandleStatus::Open,
                            "1" => CandleStatus::Close,
                            status => panic!("Error during candle status parsing, unexpected status: {status}")
                        };
                        let instrument_id = InstrumentId {
                            exchange: Exchange::OKX,
                            market_type,
                            pair: CurrencyPair {
                                target,
                                source,
                            },
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
            });
            client
        });
    }

    fn unsubscribe_candles(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        debug!("Remove socket for candles");
        let mut socket_id = format!("candles-{}-{}", currency_pair.target, currency_pair.source);
        if !MarketType::Spot.eq(market_type) {
            socket_id = format!("{}-{}", socket_id, market_type);
        }
        self.sockets.retain(|key, _| !socket_id.eq(key));
    }

    fn listen_orders<T: Fn(Order) + Send + 'static>(&mut self, callback: T) {
        let entry = self.sockets.entry("orders".to_string());
        entry.or_insert({
            let mut client = OkExWebsocket::private(self.is_demo, &self.ws_url, &self.api_key, &self.api_secret, &self.api_passphrase)
                .unwrap();
            client.send(Command::subscribe(vec![Channel::Orders {
                inst_type: InstType::Any,
                inst_id: None,
                uly: None,
            }])).unwrap();
            client.handle_message(move |message| {
                match message.unwrap() {
                    Message::Data { arg: _, data, .. } => {
                        debug!("Retrieved massage with raw payload: {:?}", &data);
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
                                let order_type = {
                                    match order_details.ord_type {
                                        OrdType::Market => OrderType::Market,
                                        OrdType::Limit => OrderType::Limit(order_details.px.unwrap()),
                                        order_type => panic!("Unsupported order type: {order_type:?}")
                                    }
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
                                    size: order_details.sz,
                                    avg_price: order_details.avg_px,
                                };
                                callback(order);
                            }
                        }
                    }
                    Message::Error { code, msg, .. } => error!("Error {}: {}", code, msg),
                    _ => {}
                }
            });
            client
        });
    }

    fn listen_account<T: Fn(Position) + Send + 'static>(&mut self, callback: T) {
        let entry = self.sockets.entry("account".to_string());
        entry.or_insert({
            let mut client = OkExWebsocket::private(self.is_demo, &self.ws_url, &self.api_key, &self.api_secret, &self.api_passphrase)
                .unwrap();
            client.send(Command::subscribe(vec![Channel::account(None)])).unwrap();
            let mut positions = HashMap::new();
            client.handle_message(move |message| {
                match message.unwrap() {
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
            });
            client
        });
    }

    async fn place_order(&self, create_order: &CreateOrder) -> Order {
        let inst_id = format!("{}-{}",
                              create_order.pair.target,
                              create_order.pair.source);
        let td_mode = match create_order.market_type {
            OrderMarketType::Spot => TdMode::Cash,
            OrderMarketType::Margin(MarginMode::Cross) => TdMode::Cross,
            OrderMarketType::Margin(MarginMode::Isolated) => TdMode::Isolated
        };
        let side = match create_order.side {
            Side::Buy => { enums::Side::Buy }
            Side::Sell => { enums::Side::Sell }
        };

        let error_message = match create_order.order_type {
            OrderType::Limit(price) => {
                let mut request = PlaceOrderRequest::limit(&inst_id, td_mode, side, price, create_order.size);
                request.set_cl_ord_id(&create_order.id.to_string());
                let [response] = self.rest_client.request(request).await.unwrap();
                debug!("Place limit order response: {response:?}");
                if response.s_code != 0 {
                    response.s_msg
                } else { None }
            }
            OrderType::Market => {
                let mut request = PlaceOrderRequest::market(&inst_id, td_mode, side, create_order.size);
                request.set_cl_ord_id(&create_order.id.to_string());
                let [response] = self.rest_client.request(request).await.unwrap();
                debug!("Place market order response: {response:?}");
                if response.s_code != 0 {
                    response.s_msg
                } else { None }
            }
        };
        if let Some(error_message) = error_message {
            Order {
                id: create_order.id.to_owned(),
                timestamp: Utc::now(),
                simulation_id: None,
                status: OrderStatus::Failed(error_message),
                exchange: Exchange::OKX,
                pair: create_order.pair,
                market_type: create_order.market_type,
                order_type: create_order.order_type,
                side: create_order.side,
                size: create_order.size,
                avg_price: 0.0,
            }
        } else {
            Order {
                id: create_order.id.to_owned(),
                timestamp: Utc::now(),
                simulation_id: None,
                status: OrderStatus::Created,
                exchange: Exchange::OKX,
                pair: create_order.pair,
                market_type: create_order.market_type,
                order_type: create_order.order_type,
                side: create_order.side,
                size: create_order.size,
                avg_price: 0.0,
            }
        }
    }

    async fn candles_history(&self,
                             currency_pair: &CurrencyPair,
                             market_type: &MarketType,
                             timeframe: Timeframe,
                             from_timestamp: Option<DateTime<Utc>>,
                             to_timestamp: Option<DateTime<Utc>>,
                             limit: Option<u8>) -> Vec<Candle> { // todo check that limit no more than 100
        let mut inst_id = format!("{}-{}", currency_pair.target, currency_pair.source);
        if !MarketType::Spot.eq(market_type) {
            inst_id = format!("{}-{}", inst_id, market_type);
        };
        let bar = match timeframe {
            Timeframe::OneS => "1s",
            Timeframe::OneM => "1m",
            Timeframe::FiveM => "5m",
            Timeframe::FifteenM => "15m",
            Timeframe::ThirtyM => "30m",
            Timeframe::OneH => "1H",
            Timeframe::TwoH => "2H",
            Timeframe::FourH => "4H",
            Timeframe::OneD => "1D",
        };
        let request = CandlesHistoryRequest {
            inst_id,
            bar: Some(bar.to_string()),
            before: from_timestamp.map(|timestamp| timestamp.timestamp_millis().to_string()),
            after: to_timestamp.map(|timestamp| timestamp.timestamp_millis().to_string()),
            limit,
        };

        self.rest_client.request(request).await.unwrap()
            .into_iter()
            .map(|dto| {
                let id = format!("{}_{}_{}_{}_{}_{}", Exchange::OKX, market_type, currency_pair.target, currency_pair.source, timeframe, dto.0.timestamp());
                let status = match dto.8.as_str() {
                    "0" => CandleStatus::Open,
                    "1" => CandleStatus::Close,
                    status => panic!("Error during candle status parsing, unexpected status: {status}")
                };
                let instrument_id = InstrumentId {
                    exchange: Exchange::OKX,
                    market_type: *market_type,
                    pair: CurrencyPair {
                        target: currency_pair.target,
                        source: currency_pair.source,
                    },
                };
                Candle {
                    id,
                    status,
                    instrument_id,
                    timestamp: dto.0,
                    timeframe,
                    open_price: dto.1,
                    highest_price: dto.2,
                    lowest_price: dto.3,
                    close_price: dto.4,
                    target_volume: dto.6,
                    source_volume: dto.7,
                }
            })
            .rev()
            .collect()
    }
}
