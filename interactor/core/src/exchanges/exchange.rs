use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain_model::{Candle, CreateOrder, CurrencyPair, MarketType, Order, Timeframe};
use eac::websocket::WsMessageHandler;

#[async_trait]
pub trait ExchangeApi {
    async fn subscribe_ticks<H: WsMessageHandler>(&self, currency_pair: &CurrencyPair, market_type: &MarketType, handler: H);
    async fn unsubscribe_ticks(&self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn subscribe_candles<H: WsMessageHandler>(&self, currency_pair: &CurrencyPair, market_type: &MarketType, handler: H);
    async fn unsubscribe_candles(&self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn listen_orders<H: WsMessageHandler>(&self, handler: H);
    async fn listen_positions<H: WsMessageHandler>(&self, handler: H);
    async fn place_order(&self, create_order: &CreateOrder) -> Order;
    async fn candles_history(&self, currency_pair: &CurrencyPair, market_type: &MarketType, timeframe: Timeframe, before: Option<DateTime<Utc>>, after: Option<DateTime<Utc>>, limit: Option<u8>) -> Vec<Candle>;
}
