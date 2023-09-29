use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use tracing::{debug, error};

use domain_model::{Candle, CandleStatus, CreateOrder, CurrencyPair, Exchange, InstrumentId, MarginMode, MarketType, Order, OrderMarketType, OrderStatus, OrderType, Side, Size, Timeframe};
use eac::{enums, rest};
use eac::enums::{InstType, TdMode};
use eac::rest::{CandlesHistoryRequest, OkExRest, PlaceOrderRequest, RateLimitedRestClient, Trigger};
use eac::websocket::{Channel, Command, OkxWsClient};
use engine_core_api::api::EngineApi;
use interactor_exchange_api::ExchangeApi;
use storage_core_api::StorageApi;

use crate::handlers::{CandleHandler, OrderHandler, PositionHandler, TickHandler};

pub struct OkxExchange<E: EngineApi, S: StorageApi> {
    is_demo: bool,
    api_key: String,
    api_secret: String,
    api_passphrase: String,
    ws_url: String,
    sockets: Arc<Mutex<RefCell<HashMap<String, OkxWsClient>>>>,
    rest_client: RateLimitedRestClient,

    engine_client: Arc<E>,
    storage_client: Arc<S>,
}

impl<E: EngineApi, S: StorageApi> OkxExchange<E, S> {
    pub fn new(is_demo: bool, http_url: &str, ws_url: &str, api_key: &str, api_secret: &str, api_passphrase: &str,
               engine_client: Arc<E>, storage_client: Arc<S>) -> Self {
        let rest_client = OkExRest::with_credential(http_url, is_demo, api_key, api_secret, api_passphrase);
        Self {
            is_demo,
            api_key: api_key.to_owned(),
            api_secret: api_secret.to_owned(),
            api_passphrase: api_passphrase.to_owned(),
            ws_url: ws_url.to_owned(),
            sockets: Default::default(),
            rest_client: RateLimitedRestClient::new(rest_client),
            engine_client,
            storage_client,
        }
    }
}

#[async_trait]
impl<E: EngineApi, S: StorageApi> ExchangeApi for OkxExchange<E, S> {
    fn id(&self) -> Exchange {
        Exchange::OKX
    }

    // todo maybe better don't create client for each subscription and use one thread to handle all messages
    async fn subscribe_ticks(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        let mut inst_id = format!("{}-{}", currency_pair.target, currency_pair.source);
        if !MarketType::Spot.eq(market_type) {
            inst_id = format!("{}-{}", inst_id, market_type);
        }
        let id: &str = &format!("mark-price-{}", &inst_id);
        let already_exists = self.sockets
            .lock()
            .await
            .borrow()
            .contains_key(id);
        if !already_exists {
            let engine_client = Arc::clone(&self.engine_client);
            let handler = TickHandler::new(engine_client, *currency_pair, *market_type);
            let client = OkxWsClient::public(false, &self.ws_url, handler).await;
            client.send(Command::subscribe(vec![Channel::MarkPrice {
                inst_id,
            }])).await;
            self.sockets
                .lock()
                .await
                .borrow_mut()
                .insert(id.to_string(), client);
        }
    }

    async fn unsubscribe_ticks(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        debug!("Remove socket for ticks");
        let mut socket_id = format!("mark-price-{}-{}", currency_pair.target, currency_pair.source);
        if !MarketType::Spot.eq(market_type) {
            socket_id = format!("{}-{}", socket_id, market_type);
        }
        self.sockets
            .lock()
            .await
            .borrow_mut()
            .retain(|key, _| !socket_id.eq(key));
    }

    async fn subscribe_candles(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        let mut inst_id = format!("{}-{}", currency_pair.target, currency_pair.source);
        if !MarketType::Spot.eq(market_type) {
            inst_id = format!("{}-{}", inst_id, market_type);
        }
        let id: &str = &format!("candles-{}", &inst_id);
        let already_exists = self.sockets
            .lock()
            .await
            .borrow()
            .contains_key(id);
        if !already_exists {
            let storage_client = Arc::clone(&self.storage_client);
            let handler = CandleHandler::new(*currency_pair, *market_type, storage_client);
            let client = OkxWsClient::business(
                self.is_demo,
                &self.ws_url,
                handler).await;

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
            client.send(subscribe_command).await;
            self.sockets
                .lock()
                .await
                .borrow_mut()
                .insert(id.to_string(), client);
        }
    }

    async fn unsubscribe_candles(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        debug!("Remove socket for candles");
        let mut socket_id = format!("candles-{}-{}", currency_pair.target, currency_pair.source);
        if !MarketType::Spot.eq(market_type) {
            socket_id = format!("{}-{}", socket_id, market_type);
        }
        self.sockets
            .lock()
            .await
            .borrow_mut()
            .retain(|key, _| !socket_id.eq(key));
    }

    async fn listen_orders(&self) {
        const ID: &str = "orders";
        let already_exists = self.sockets
            .lock()
            .await
            .borrow()
            .contains_key(ID);
        if !already_exists {
            let storage_client = Arc::clone(&self.storage_client);
            let handler = OrderHandler::new(storage_client);
            let client = OkxWsClient::private(
                self.is_demo,
                &self.ws_url,
                &self.api_key,
                &self.api_secret,
                &self.api_passphrase,
                handler).await;
            client.send(Command::subscribe(vec![Channel::Orders {
                inst_type: InstType::Any,
                inst_id: None,
                uly: None,
            }])).await;
            self.sockets
                .lock()
                .await
                .borrow_mut()
                .insert(ID.to_string(), client);
        }
    }

    async fn listen_positions(&self) {
        const ID: &str = "position";
        let already_exists = self.sockets
            .lock()
            .await
            .borrow()
            .contains_key(ID);
        if !already_exists {
            let storage_client = Arc::clone(&self.storage_client);
            let handler = PositionHandler::new(storage_client);
            let client = OkxWsClient::private(
                self.is_demo,
                &self.ws_url,
                &self.api_key,
                &self.api_secret,
                &self.api_passphrase,
                handler).await;
            client.send(Command::subscribe(vec![Channel::account(None)])).await;
            self.sockets
                .lock()
                .await
                .borrow_mut()
                .insert(ID.to_string(), client);
        }
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
        let stop_loss = if let Some(stop_loss) = &create_order.stop_loss {
            Trigger::new(stop_loss.trigger_px, stop_loss.order_px)
        } else { None };
        let take_profit = if let Some(take_profit) = &create_order.take_profit {
            Trigger::new(take_profit.trigger_px, take_profit.order_px)
        } else { None };
        let size = match create_order.size {
            Size::Target(size) => rest::Size::Target(size),
            Size::Source(size) => rest::Size::Source(size)
        };
        let error_message = match create_order.order_type {
            OrderType::Limit(price) => {
                let mut request = PlaceOrderRequest::limit(&inst_id, td_mode, side, price, size, stop_loss, take_profit);
                request.set_cl_ord_id(&create_order.id.to_string());
                let [response] = self.rest_client.request(request).await.unwrap();
                debug!("Place limit order response: {response:?}");
                if response.s_code != 0 {
                    response.s_msg
                } else { None }
            }
            OrderType::Market => {
                let mut request = PlaceOrderRequest::market(&inst_id, td_mode, side, size, stop_loss, take_profit);
                request.set_cl_ord_id(&create_order.id.to_string());
                let [response] = self.rest_client.request(request).await.unwrap();
                debug!("Place market order response: {response:?}");
                if response.s_code != 0 {
                    response.s_msg
                } else { None }
            }
        };
        if let Some(error_message) = error_message {
            error!("Failed to place order: {}", error_message);
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
                size: create_order.size.clone(),
                avg_price: 0.0,
                stop_loss: create_order.stop_loss.clone(),
                take_profit: create_order.take_profit.clone(),
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
                size: create_order.size.clone(),
                avg_price: 0.0,
                stop_loss: create_order.stop_loss.clone(),
                take_profit: create_order.take_profit.clone(),
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
            Timeframe::OneD => "1Dutc",
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
