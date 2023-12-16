use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use tokio::sync::Mutex;
use tracing::{debug, error};

use domain_model::{Candle, CandleStatus, CreateOrder, Currency, CurrencyPair, Exchange, InstrumentId, MarketType, Order, OrderMarketType, OrderStatus, OrderType, Side, Size, Timeframe};
use domain_model::MarginMode::Isolated;
use eac::bybit::{enums, rest};
use eac::bybit::enums::Category;
use eac::bybit::rest::{BybitRest, CandlesRequest, OrderDetailsRequest, OrderDetailsResponse, PlaceOrderRequest, RateLimitedRestClient};
use eac::bybit::websocket::{BybitWsClient, Channel, Command};
use engine_core_api::api::EngineApi;
use interactor_exchange_api::ExchangeApi;
use storage_core_api::StorageApi;

use crate::handlers::{CandleHandler, OrderHandler, TickHandler};

pub struct BybitExchange<E: EngineApi, S: StorageApi> {
    api_key: String,
    api_secret: String,
    ws_url: String,
    sockets: Arc<Mutex<RefCell<HashMap<String, BybitWsClient>>>>,
    private_client: RateLimitedRestClient,
    public_client: RateLimitedRestClient,

    engine_client: Arc<E>,
    storage_client: Arc<S>,
}

impl<E: EngineApi, S: StorageApi> BybitExchange<E, S> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        http_url: &str,
        ws_url: &str,
        api_key: &str,
        api_secret: &str,
        engine_client: Arc<E>,
        storage_client: Arc<S>,
    ) -> Self {
        let private_client = BybitRest::with_credential(http_url, api_key, api_secret);
        let public_client = BybitRest::new(http_url);
        Self {
            api_key: api_key.to_owned(),
            api_secret: api_secret.to_owned(),
            ws_url: ws_url.to_owned(),
            sockets: Default::default(),
            private_client: RateLimitedRestClient::new(private_client),
            public_client: RateLimitedRestClient::new(public_client),
            engine_client,
            storage_client,
        }
    }
}

#[async_trait]
impl<E: EngineApi, S: StorageApi> ExchangeApi for BybitExchange<E, S> {
    fn id(&self) -> Exchange {
        Exchange::BYBIT
    }

    async fn subscribe_ticks(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        let inst_id = format!("{}{}", currency_pair.target, currency_pair.source);
        let id: &str = &format!("ticks-{}", &inst_id);
        let already_exists = self.sockets.lock().await.borrow().contains_key(id);
        if !already_exists {
            let engine_client = Arc::clone(&self.engine_client);
            let handler = TickHandler::new(engine_client, *currency_pair, *market_type);
            let client = BybitWsClient::public(&self.ws_url, handler).await;
            client
                .send(Command::subscribe(vec![Channel::Ticker(inst_id)]))
                .await;
            self.sockets
                .lock()
                .await
                .borrow_mut()
                .insert(id.to_string(), client);
        }
    }

    async fn unsubscribe_ticks(&self, currency_pair: &CurrencyPair, _market_type: &MarketType) {
        debug!("Remove socket for ticks");
        let socket_id = format!(
            "ticks-{}{}",
            currency_pair.target, currency_pair.source
        );
        self.sockets
            .lock()
            .await
            .borrow_mut()
            .retain(|key, _| !socket_id.eq(key));
    }

    async fn subscribe_candles(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        let inst_id = format!("{}{}", currency_pair.target, currency_pair.source);
        let id: &str = &format!("candles-{}", &inst_id);
        let already_exists = self.sockets.lock().await.borrow().contains_key(id);
        if !already_exists {
            let storage_client = Arc::clone(&self.storage_client);
            let handler = CandleHandler::new(*currency_pair, *market_type, storage_client);
            let client = BybitWsClient::public(&self.ws_url, handler).await;

            let subscribe_command = Command::subscribe(vec![
                Channel::Candles((enums::Timeframe::Min1, inst_id.as_str()).into()),
                Channel::Candles((enums::Timeframe::Min5, inst_id.as_str()).into()),
                Channel::Candles((enums::Timeframe::Min15, inst_id.as_str()).into()),
                Channel::Candles((enums::Timeframe::Min30, inst_id.as_str()).into()),
                Channel::Candles((enums::Timeframe::H1, inst_id.as_str()).into()),
                Channel::Candles((enums::Timeframe::H2, inst_id.as_str()).into()),
                Channel::Candles((enums::Timeframe::H4, inst_id.as_str()).into()),
                Channel::Candles((enums::Timeframe::D, inst_id.as_str()).into()),
            ]);
            client.send(subscribe_command).await;
            self.sockets
                .lock()
                .await
                .borrow_mut()
                .insert(id.to_string(), client);
        }
    }

    async fn unsubscribe_candles(&self, currency_pair: &CurrencyPair, _market_type: &MarketType) {
        debug!("Remove socket for candles");
        let socket_id = format!("candles-{}{}", currency_pair.target, currency_pair.source);
        self.sockets
            .lock()
            .await
            .borrow_mut()
            .retain(|key, _| !socket_id.eq(key));
    }

    async fn listen_orders(&self) {
        const ID: &str = "orders";
        let already_exists = self.sockets.lock().await.borrow().contains_key(ID);
        if !already_exists {
            let storage_client = Arc::clone(&self.storage_client);
            let handler = OrderHandler::new(storage_client);
            let client = BybitWsClient::private(
                &self.ws_url,
                &self.api_key,
                &self.api_secret,
                handler,
            ).await;
            client
                .send(Command::subscribe(vec![Channel::Orders]))
                .await;
            self.sockets
                .lock()
                .await
                .borrow_mut()
                .insert(ID.to_string(), client);
        }
    }

    async fn listen_positions(&self) {
        // todo unimplemented!("not supported yet")
    }

    async fn place_order(&self, create_order: &CreateOrder) -> Order {
        let inst_id = format!("{}{}", create_order.pair.target, create_order.pair.source);
        let is_leveraged = match create_order.market_type {
            OrderMarketType::Spot => false,
            OrderMarketType::Margin(_) => true,
        };
        let side = match create_order.side {
            Side::Buy => enums::Side::Buy,
            Side::Sell => enums::Side::Sell,
        };
        let size = match create_order.size {
            Size::Target(size) => rest::Size::Target(size),
            Size::Source(size) => rest::Size::Source(size),
        };
        let place_order_request = match create_order.order_type {
            OrderType::Limit(_price) => unimplemented!("not supported yet"), // todo
            OrderType::Market => PlaceOrderRequest::market(
                Some(create_order.id.clone()),
                &inst_id,
                Category::Spot,
                side,
                size,
                is_leveraged,
            )
        };
        let response = self.private_client.request(place_order_request).await;
        debug!("Place order response: {response:?}");
        if let Err(error_message) = response {
            error!("Failed to place order: {}", error_message);
            Order {
                id: create_order.id.to_owned(),
                timestamp: Utc::now(),
                simulation_id: None,
                status: OrderStatus::Failed(error_message.to_string()),
                exchange: Exchange::BYBIT,
                pair: create_order.pair,
                market_type: create_order.market_type,
                order_type: create_order.order_type,
                side: create_order.side,
                size: create_order.size.clone(),
                fee: 0.,
                avg_fill_price: 0.0,
                stop_loss: create_order.stop_loss.clone(),
                avg_sl_price: 0.0,
                take_profit: create_order.take_profit.clone(),
                avg_tp_price: 0.0,
            }
        } else {
            Order {
                id: create_order.id.to_owned(),
                timestamp: Utc::now(),
                simulation_id: None,
                status: OrderStatus::Created,
                exchange: Exchange::BYBIT,
                pair: create_order.pair,
                market_type: create_order.market_type,
                order_type: create_order.order_type,
                side: create_order.side,
                size: create_order.size.clone(),
                fee: 0.,
                avg_fill_price: 0.0,
                stop_loss: create_order.stop_loss.clone(),
                avg_sl_price: 0.0,
                take_profit: create_order.take_profit.clone(),
                avg_tp_price: 0.0,
            }
        }
    }

    async fn candles_history(&self, currency_pair: &CurrencyPair, market_type: &MarketType, timeframe: Timeframe,
                             from_timestamp: Option<DateTime<Utc>>, to_timestamp: Option<DateTime<Utc>>, limit: Option<u8>) -> Vec<Candle> {
        let symbol = format!("{}{}", currency_pair.target, currency_pair.source);
        let interval = match timeframe {
            Timeframe::OneS => unimplemented!("not supported yet"),
            Timeframe::OneM => enums::Timeframe::Min1,
            Timeframe::FiveM => enums::Timeframe::Min5,
            Timeframe::FifteenM => enums::Timeframe::Min15,
            Timeframe::ThirtyM => enums::Timeframe::Min30,
            Timeframe::OneH => enums::Timeframe::H1,
            Timeframe::TwoH => enums::Timeframe::H2,
            Timeframe::FourH => enums::Timeframe::H4,
            Timeframe::OneD => enums::Timeframe::D,
        };
        let request = CandlesRequest {
            category: Category::Spot,
            symbol,
            interval: interval.as_topic(),
            start: from_timestamp.map(|timestamp| timestamp.timestamp_millis()),
            end: to_timestamp.map(|timestamp| timestamp.timestamp_millis()),
            limit: limit.map(|limit| limit as u16),
        };

        let candles = self.public_client.request(request).await.unwrap();

        candles.list.into_iter()
            .map(|item| {
                let id = format!(
                    "{}_{}_{}_{}_{}_{}",
                    Exchange::BYBIT,
                    market_type,
                    currency_pair.target,
                    currency_pair.source,
                    timeframe,
                    item.0
                );
                let status = CandleStatus::Close;
                let instrument_id = InstrumentId {
                    exchange: Exchange::BYBIT,
                    market_type: *market_type,
                    pair: CurrencyPair {
                        target: currency_pair.target,
                        source: currency_pair.source,
                    },
                };
                let timestamp = Utc.timestamp_millis_opt(item.0.parse().unwrap()).unwrap();
                Candle {
                    id,
                    status,
                    instrument_id,
                    timestamp,
                    timeframe,
                    open_price: item.1.parse().unwrap(),
                    highest_price: item.2.parse().unwrap(),
                    lowest_price: item.3.parse().unwrap(),
                    close_price: item.4.parse().unwrap(),
                    source_volume: item.5.parse().unwrap(),
                    target_volume: item.6.parse().unwrap(),
                }
            })
            .rev()
            .collect()
    }

    async fn get_order(&self, order_id: &str) -> Option<Order> {
        let request = OrderDetailsRequest {
            category: Category::Spot,
            symbol: None,
            base_coin: None,
            settle_coin: None,
            order_id: None,
            order_link_id: Some(order_id.to_string()),
            order_filter: None,
            order_status: None,
            start_time: None,
            end_time: None,
            limit: None,
            cursor: None,
        };
        let response = self.private_client.request(request).await;
        if let Ok(response) = response {
            if let Some(order) = response.list.first() {
                if order.order_link_id == order_id {
                    return Some(convert_order_details_to_order(order));
                }
            }
        }
        None
    }

    async fn get_total_balance(&self) -> f64 {
        // todo unimplemented!("not supported yet")
        0.
    }
}

fn convert_order_details_to_order(item: &OrderDetailsResponse) -> Order {
    let status = match item.order_status {
        enums::OrderStatus::Created |
        enums::OrderStatus::Untriggered =>
            OrderStatus::Created,
        enums::OrderStatus::Cancelled |
        enums::OrderStatus::Deactivated =>
            OrderStatus::Canceled,
        enums::OrderStatus::New |
        enums::OrderStatus::PartiallyFilled |
        enums::OrderStatus::Triggered =>
            OrderStatus::InProgress,
        enums::OrderStatus::Filled |
        enums::OrderStatus::PartiallyFilledCanceled =>
            OrderStatus::Completed,
        enums::OrderStatus::Rejected => OrderStatus::Failed("Rejected".into())
    };
    let pair = {
        CurrencyPair {
            target: Currency::from_str(&item.symbol[..3]).unwrap_or(Currency::from_str(&item.symbol[..4]).unwrap()),
            source: Currency::from_str(&item.symbol[3..]).unwrap_or(Currency::from_str(&item.symbol[4..]).unwrap()),
        }
    };
    let market_type = if item.is_leverage == "1" {
        OrderMarketType::Margin(Isolated)
    } else {
        OrderMarketType::Spot
    };
    let side = match item.side {
        enums::Side::Buy => Side::Buy,
        enums::Side::Sell => Side::Sell,
    };
    let order_type = match item.order_type {
        enums::OrderType::Market => OrderType::Market,
        enums::OrderType::Limit => OrderType::Limit(item.price),
        order_type => panic!("Unsupported order type: {order_type:?}"),
    };
    let size = match order_type {
        OrderType::Market => match side {
            Side::Buy => Size::Target(item.cum_exec_qty),
            Side::Sell => Size::Source(item.cum_exec_value),
        },
        OrderType::Limit(_price) => match side {
            Side::Buy => Size::Source(item.qty),
            Side::Sell => Size::Target(item.qty),
        }
    };

    let fee = match side {
        Side::Buy => item.cum_exec_fee * item.avg_price,
        Side::Sell => item.cum_exec_fee
    };

    Order {
        id: item.order_link_id.clone(),
        timestamp: Utc::now(),
        simulation_id: None,
        status,
        exchange: Exchange::BYBIT,
        pair,
        market_type,
        order_type,
        side,
        size,
        fee,
        avg_fill_price: item.avg_price,
        stop_loss: None,
        avg_sl_price: 0.,
        take_profit: None,
        avg_tp_price: 0.,
    }
}
