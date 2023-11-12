use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use tracing::{debug, error};

use domain_model::{Candle, CandleStatus, CreateOrder, Currency, CurrencyPair, Exchange, InstrumentId, LP, MarginMode, MarketType, Order, OrderMarketType, OrderStatus, OrderType, Side, Size, Timeframe};
use eac::{enums, rest};
use eac::enums::{InstType, OrdState, OrdType, TdMode};
use eac::rest::{BalanceRequest, CandlesHistoryRequest, OkExRest, OrderDetailsResponse, OrderHistoryRequest, PlaceOrderRequest, RateLimitedRestClient, Trigger};
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
    private_client: RateLimitedRestClient,
    public_client: RateLimitedRestClient,

    engine_client: Arc<E>,
    storage_client: Arc<S>,
}

impl<E: EngineApi, S: StorageApi> OkxExchange<E, S> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        is_demo: bool,
        http_url: &str,
        ws_url: &str,
        api_key: &str,
        api_secret: &str,
        api_passphrase: &str,
        engine_client: Arc<E>,
        storage_client: Arc<S>,
    ) -> Self {
        let private_client =
            OkExRest::with_credential(http_url, is_demo, api_key, api_secret, api_passphrase);
        let public_client =
            OkExRest::new(http_url, false);
        Self {
            is_demo,
            api_key: api_key.to_owned(),
            api_secret: api_secret.to_owned(),
            api_passphrase: api_passphrase.to_owned(),
            ws_url: ws_url.to_owned(),
            sockets: Default::default(),
            private_client: RateLimitedRestClient::new(private_client),
            public_client: RateLimitedRestClient::new(public_client),
            engine_client,
            storage_client,
        }
    }

    async fn get_order_by_market(&self, market_type: MarketType, order_id: &str) -> Option<Order> {
        let inst_type = match market_type {
            MarketType::Spot => "SPOT".to_string(),
            MarketType::Margin => "MARGIN".to_string()
        };
        let request = OrderHistoryRequest {
            inst_type,
            inst_id: None,
        };
        let mut main_order = None;
        let mut lp = None;
        self.private_client.request(request)
            .await
            .unwrap()
            .into_iter()
            .filter(|info| info.cl_ord_id == order_id || info.tag == order_id)
            .for_each(|info| {
                if !info.cl_ord_id.is_empty() && info.source.is_none() {
                    main_order = Some(info);
                } else if !info.tag.is_empty() && info.source == Some(7) {
                    lp = Some(info);
                }
            });
        main_order.map(|order| convert_order_details_to_order(order, lp))
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
        if !MarketType::Spot.eq(market_type) && !MarketType::Margin.eq(market_type) {
            inst_id = format!("{}-{}", inst_id, market_type);
        }
        let id: &str = &format!("mark-price-{}", &inst_id);
        let already_exists = self.sockets.lock().await.borrow().contains_key(id);
        if !already_exists {
            let engine_client = Arc::clone(&self.engine_client);
            let handler = TickHandler::new(engine_client, *currency_pair, *market_type);
            let client = OkxWsClient::public(false, &self.ws_url, handler).await;
            client
                .send(Command::subscribe(vec![Channel::MarkPrice { inst_id }]))
                .await;
            self.sockets
                .lock()
                .await
                .borrow_mut()
                .insert(id.to_string(), client);
        }
    }

    async fn unsubscribe_ticks(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        debug!("Remove socket for ticks");
        let mut socket_id = format!(
            "mark-price-{}-{}",
            currency_pair.target, currency_pair.source
        );
        if !MarketType::Spot.eq(market_type) && !MarketType::Margin.eq(market_type) {
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
        if !MarketType::Spot.eq(market_type) && !MarketType::Margin.eq(market_type) {
            inst_id = format!("{}-{}", inst_id, market_type);
        }
        let id: &str = &format!("candles-{}", &inst_id);
        let already_exists = self.sockets.lock().await.borrow().contains_key(id);
        if !already_exists {
            let storage_client = Arc::clone(&self.storage_client);
            let handler = CandleHandler::new(*currency_pair, *market_type, storage_client);
            let client = OkxWsClient::business(false, &self.ws_url, handler).await;

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
        if !MarketType::Spot.eq(market_type) && !MarketType::Margin.eq(market_type) {
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
        let already_exists = self.sockets.lock().await.borrow().contains_key(ID);
        if !already_exists {
            let storage_client = Arc::clone(&self.storage_client);
            let handler = OrderHandler::new(storage_client);
            let client = OkxWsClient::private(
                self.is_demo,
                &self.ws_url,
                &self.api_key,
                &self.api_secret,
                &self.api_passphrase,
                handler,
            )
                .await;
            client
                .send(Command::subscribe(vec![Channel::Orders {
                    inst_type: InstType::Any,
                    inst_id: None,
                    uly: None,
                }]))
                .await;
            self.sockets
                .lock()
                .await
                .borrow_mut()
                .insert(ID.to_string(), client);
        }
    }

    async fn listen_positions(&self) {
        const ID: &str = "position";
        let already_exists = self.sockets.lock().await.borrow().contains_key(ID);
        if !already_exists {
            let storage_client = Arc::clone(&self.storage_client);
            let handler = PositionHandler::new(storage_client);
            let client = OkxWsClient::private(
                self.is_demo,
                &self.ws_url,
                &self.api_key,
                &self.api_secret,
                &self.api_passphrase,
                handler,
            )
                .await;
            client
                .send(Command::subscribe(vec![Channel::account(None)]))
                .await;
            self.sockets
                .lock()
                .await
                .borrow_mut()
                .insert(ID.to_string(), client);
        }
    }

    async fn place_order(&self, create_order: &CreateOrder) -> Order {
        let inst_id = format!("{}-{}", create_order.pair.target, create_order.pair.source);
        let mut margin_ccy = None;
        let td_mode = match create_order.market_type {
            OrderMarketType::Spot => TdMode::Cash,
            OrderMarketType::Margin(MarginMode::Cross(ccy)) => {
                margin_ccy = ccy.to_string().into();
                TdMode::Cross
            },
            OrderMarketType::Margin(MarginMode::Isolated) => TdMode::Isolated,
        };
        let side = match create_order.side {
            Side::Buy => enums::Side::Buy,
            Side::Sell => enums::Side::Sell,
        };
        let stop_loss = if let Some(stop_loss) = &create_order.stop_loss {
            let order_px = match stop_loss.order_px {
                OrderType::Limit(limit) => limit,
                OrderType::Market => -1.
            };
            Trigger::new(stop_loss.trigger_px, order_px)
        } else {
            None
        };
        let take_profit = if let Some(take_profit) = &create_order.take_profit {
            let order_px = match take_profit.order_px {
                OrderType::Limit(limit) => limit,
                OrderType::Market => -1.
            };
            Trigger::new(take_profit.trigger_px, order_px)
        } else {
            None
        };
        let size = match create_order.size {
            Size::Target(size) => rest::Size::Target(size),
            Size::Source(size) => rest::Size::Source(size),
        };
        let mut place_order_request = match create_order.order_type {
            OrderType::Limit(price) => PlaceOrderRequest::limit(
                &inst_id,
                td_mode,
                margin_ccy,
                side,
                price,
                size,
                stop_loss,
                take_profit,
            ),
            OrderType::Market => PlaceOrderRequest::market(
                &inst_id,
                td_mode,
                margin_ccy,
                side,
                size,
                stop_loss,
                take_profit,
            )
        };
        place_order_request.set_cl_ord_id(&create_order.id);
        place_order_request.set_tag(&create_order.id);
        dbg!(&place_order_request);
        let [response] = self.private_client.request(place_order_request).await.unwrap();
        debug!("Place order response: {response:?}");
        let error_message = if response.s_code != 0 {
            response.s_msg
        } else {
            None
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
                exchange: Exchange::OKX,
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

    async fn candles_history(
        &self,
        currency_pair: &CurrencyPair,
        market_type: &MarketType,
        timeframe: Timeframe,
        from_timestamp: Option<DateTime<Utc>>,
        to_timestamp: Option<DateTime<Utc>>,
        limit: Option<u8>,
    ) -> Vec<Candle> {
        // todo check that limit no more than 100
        let mut inst_id = format!("{}-{}", currency_pair.target, currency_pair.source);
        if !MarketType::Spot.eq(market_type) && !MarketType::Margin.eq(market_type) {
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

        self.public_client
            .request(request)
            .await
            .unwrap()
            .into_iter()
            .map(|dto| {
                let id = format!(
                    "{}_{}_{}_{}_{}_{}",
                    Exchange::OKX,
                    market_type,
                    currency_pair.target,
                    currency_pair.source,
                    timeframe,
                    dto.0.timestamp()
                );
                let status = match dto.8.as_str() {
                    "0" => CandleStatus::Open,
                    "1" => CandleStatus::Close,
                    status => {
                        panic!("Error during candle status parsing, unexpected status: {status}")
                    }
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

    async fn get_order(&self, order_id: &str) -> Option<Order> {
        let order = self.get_order_by_market(MarketType::Spot, order_id).await;
        if order.is_some() {
            order
        } else {
            self.get_order_by_market(MarketType::Margin, order_id).await
        }
    }

    async fn get_total_balance(&self) -> f64 {
        let request = BalanceRequest::default();
        self.private_client.request(request)
            .await.unwrap()
            .first().unwrap()
            .total_eq
    }
}

fn convert_order_details_to_order(order_details: OrderDetailsResponse, lp: Option<OrderDetailsResponse>) -> Order {
    let status = match order_details.state {
        OrdState::Canceled => OrderStatus::Canceled,
        OrdState::Live => OrderStatus::InProgress,
        OrdState::PartiallyFilled => OrderStatus::InProgress,
        OrdState::Filled => {
            if order_details.sl_trigger_px.is_none() && order_details.tp_trigger_px.is_none() {
                OrderStatus::Completed
            } else {
                OrderStatus::InProgress
            }
        }
    };
    let pair = {
        let mut inst_id = order_details.inst_id.split('-');
        CurrencyPair {
            target: Currency::from_str(inst_id.next().unwrap()).unwrap(),
            source: Currency::from_str(inst_id.next().unwrap()).unwrap(),
        }
    };
    let margin_ccy = order_details.ccy;
    let market_type = {
        match order_details.td_mode {
            TdMode::Cross => OrderMarketType::Margin(MarginMode::Cross(Currency::from_str(&margin_ccy).unwrap())),
            TdMode::Isolated => OrderMarketType::Margin(MarginMode::Isolated),
            TdMode::Cash => OrderMarketType::Spot,
        }
    };
    let side = match order_details.side {
        enums::Side::Buy => Side::Buy,
        enums::Side::Sell => Side::Sell,
    };
    let order_type = match order_details.ord_type {
        OrdType::Market => OrderType::Market,
        OrdType::Limit => OrderType::Limit(order_details.px.unwrap()),
        order_type => panic!("Unsupported order type: {order_type:?}"),
    };
    let size = match order_details.ord_type {
        OrdType::Market => match order_details.tgt_ccy.as_str() {
            "quote_ccy" => Size::Source(order_details.sz),
            "base_ccy" => Size::Target(order_details.sz),
            _ => match side {
                Side::Buy => Size::Source(order_details.sz),
                Side::Sell => Size::Target(order_details.sz)
            },
        },
        OrdType::Limit => Size::Target(order_details.sz),
        order_type => panic!("Unsupported order type: {order_type:?}"),
    };
    let stop_loss =
        if order_details.sl_trigger_px.is_some() && order_details.sl_ord_px.is_some() {
            let sl_order_px = order_details.sl_ord_px.unwrap();
            let sl_order_px = if sl_order_px == -1. {
                OrderType::Market
            } else {
                OrderType::Limit(sl_order_px)
            };
            domain_model::Trigger::new(
                order_details.sl_trigger_px.unwrap(),
                sl_order_px,
            )
        } else {
            None
        };
    let take_profit =
        if order_details.tp_trigger_px.is_some() && order_details.tp_ord_px.is_some() {
            let tp_order_px = order_details.sl_ord_px.unwrap();
            let tp_order_px = if tp_order_px == -1. {
                OrderType::Market
            } else {
                OrderType::Limit(tp_order_px)
            };
            domain_model::Trigger::new(
                order_details.tp_trigger_px.unwrap(),
                tp_order_px,
            )
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
        fee: order_details.fee.abs(),
        avg_fill_price: order_details.avg_px,
        stop_loss,
        avg_sl_price: 0.,
        take_profit,
        avg_tp_price: 0.,
    };

    if let Some(lp_order) = lp {
        let status = match lp_order.state {
            OrdState::Canceled => OrderStatus::Canceled,
            OrdState::Live => OrderStatus::InProgress,
            OrdState::PartiallyFilled => OrderStatus::InProgress,
            OrdState::Filled => {
                if lp_order.sl_trigger_px.is_none() && lp_order.tp_trigger_px.is_none() {
                    OrderStatus::Completed
                } else {
                    OrderStatus::InProgress
                }
            }
        };
        if status == OrderStatus::Completed {
            let side = match lp_order.side {
                enums::Side::Buy => Side::Buy,
                enums::Side::Sell => Side::Sell,
            };
            let size = match lp_order.ord_type {
                OrdType::Market => match lp_order.tgt_ccy.as_str() {
                    "quote_ccy" => Size::Source(lp_order.sz),
                    "base_ccy" => Size::Target(lp_order.sz),
                    _ => match side {
                        Side::Buy => Size::Source(lp_order.sz),
                        Side::Sell => Size::Target(lp_order.sz)
                    },
                },
                OrdType::Limit => Size::Target(lp_order.sz),
                order_type => panic!("Unsupported order type: {order_type:?}"),
            };

            let lp = LP {
                id: lp_order.tag,
                price: lp_order.avg_px,
                size,
            };

            add_lp_to_order(order, lp)
        } else {
            order
        }
    } else {
        order
    }
}

fn add_lp_to_order(order: Order, lp: LP) -> Order {
    let mut order = order;
    if order.stop_loss.is_some() && order.take_profit.is_some() {
        let sl_price = match order.clone().stop_loss.unwrap().order_px {
            OrderType::Limit(price) => price,
            OrderType::Market => order.clone().stop_loss.unwrap().trigger_px
        };
        let tp_price = match order.clone().take_profit.unwrap().order_px {
            OrderType::Limit(price) => price,
            OrderType::Market => order.clone().take_profit.unwrap().trigger_px
        };
        let sl_offset = (lp.price - sl_price).abs();
        let tp_offset = (lp.price - tp_price).abs();
        if tp_offset < sl_offset {
            order.avg_tp_price = lp.price;
            order.size = lp.size;
        } else {
            order.avg_sl_price = lp.price;
            order.size = lp.size;
        }
    } else if order.stop_loss.is_some() {
        order.avg_sl_price = lp.price;
        order.size = lp.size;
    } else if order.take_profit.is_some() {
        order.avg_tp_price = lp.price;
        order.size = lp.size;
    }
    order.status = OrderStatus::Completed;
    order
}
