use std::future::Future;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain_model::{Candle, CreateOrder, CurrencyPair, MarketType, Order, Position, Tick, Timeframe};

#[async_trait]
pub trait Exchange {
    async fn subscribe_ticks<T: Fn(Tick) -> F + Send + 'static, F: Future<Output=()>>(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType, callback: T);
    fn unsubscribe_ticks(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn subscribe_candles<T: Fn(Candle) + Send + 'static>(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType, callback: T);
    fn unsubscribe_candles(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn listen_orders<T: Fn(Order) + Send + 'static>(&mut self, callback: T);
    async fn listen_positions<T: Fn(Position) + Send + 'static>(&mut self, callback: T);
    async fn place_order(&mut self, create_order: &CreateOrder) -> Order;
    async fn candles_history(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType, timeframe: Timeframe, before: Option<DateTime<Utc>>, after: Option<DateTime<Utc>>, limit: Option<u8>) -> Vec<Candle>;
}