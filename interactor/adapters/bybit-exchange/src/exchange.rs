use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use domain_model::{Candle, CreateOrder, CurrencyPair, Exchange, MarketType, Order, Timeframe};
use eac::rest::{OkExRest, RateLimitedRestClient};
use eac::websocket::OkxWsClient;
use engine_core_api::api::EngineApi;
use interactor_exchange_api::ExchangeApi;
use storage_core_api::StorageApi;

pub struct BybitExchange<E: EngineApi, S: StorageApi> {
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

impl<E: EngineApi, S: StorageApi> BybitExchange<E, S> {
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
}

#[async_trait]
impl<E: EngineApi, S: StorageApi> ExchangeApi for BybitExchange<E, S> {
    fn id(&self) -> Exchange {
        Exchange::OKX
    }

    async fn subscribe_ticks(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        todo!()
    }

    async fn unsubscribe_ticks(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        todo!()
    }

    async fn subscribe_candles(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        todo!()
    }

    async fn unsubscribe_candles(&self, currency_pair: &CurrencyPair, market_type: &MarketType) {
        todo!()
    }

    async fn listen_orders(&self) {
        todo!()
    }

    async fn listen_positions(&self) {
        todo!()
    }

    async fn place_order(&self, create_order: &CreateOrder) -> Order {
        todo!()
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
        todo!()
    }

    async fn get_order(&self, order_id: &str) -> Option<Order> {
        todo!()
    }

    async fn get_total_balance(&self) -> f64 {
        todo!()
    }
}
