use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain_model::{Candle, CreateOrder, CurrencyPair, Exchange, MarketType, Order, Timeframe};

#[async_trait]
pub trait ExchangeApi: Send + Sync + 'static {
    fn id(&self) -> Exchange;
    async fn subscribe_ticks(&self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn unsubscribe_ticks(&self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn subscribe_candles(&self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn unsubscribe_candles(&self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn listen_orders(&self);
    async fn listen_positions(&self);
    async fn place_order(&self, create_order: &CreateOrder) -> Order;
    async fn candles_history(&self, currency_pair: &CurrencyPair, market_type: &MarketType, timeframe: Timeframe, before: Option<DateTime<Utc>>, after: Option<DateTime<Utc>>, limit: Option<u8>) -> Vec<Candle>;
    async fn get_order(&self, order_id: &str) -> Option<Order>;
}
