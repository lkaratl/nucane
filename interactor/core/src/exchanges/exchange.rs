use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain_model::{Candle, CreateOrder, CurrencyPair, MarketType, Order, Position, Tick, Timeframe};

#[async_trait]
pub trait ExchangeApi {
    async fn subscribe_ticks<T: Fn(Tick) + Send + 'static>(&self, currency_pair: &CurrencyPair, market_type: &MarketType, callback: T);
    async fn unsubscribe_ticks(&self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn subscribe_candles<T: Fn(Candle) + Send + 'static>(&self, currency_pair: &CurrencyPair, market_type: &MarketType, callback: T);
    async fn unsubscribe_candles(&self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn listen_orders<T: Fn(Order) + Send + 'static>(&self, callback: T);
    async fn listen_positions<T: Fn(Position) + Send + 'static>(&self, callback: T);
    async fn place_order(&self, create_order: &CreateOrder) -> Order;
    async fn candles_history(&self, currency_pair: &CurrencyPair, market_type: &MarketType, timeframe: Timeframe, before: Option<DateTime<Utc>>, after: Option<DateTime<Utc>>, limit: Option<u8>) -> Vec<Candle>;
}
