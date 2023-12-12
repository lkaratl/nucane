use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{from_value, Value};
use tracing::info;

use domain_model::{Currency, CurrencyPair, Exchange, MarginMode, Order, OrderMarketType, OrderStatus, OrderType, Side, Size};
use eac::bybit;
use eac::bybit::websocket::{OrderDetailsResponse, WsMessageHandler};
use storage_core_api::StorageApi;

pub struct OrderHandler<S: StorageApi> {
    storage_client: Arc<S>,
}

impl<S: StorageApi> OrderHandler<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self { storage_client }
    }
}

#[async_trait]
impl<S: StorageApi> WsMessageHandler for OrderHandler<S> {
    type Type = Vec<Order>;

    // todo implement order conversion for all variants not only for margin market orders
    async fn convert_data(&mut self, _topic: String, data: Value) -> Option<Self::Type> {
        info!("Retrieved massage with raw payload: {}", serde_json::to_string_pretty(&data).unwrap());
        let mut orders = Vec::new();
        let order_details: Vec<OrderDetailsResponse> = from_value(data).unwrap();
        for item in order_details {
            if item.order_link_id.parse::<u64>().is_err() {
                let status = match item.order_status {
                    bybit::enums::OrderStatus::Created |
                    bybit::enums::OrderStatus::Untriggered =>
                        OrderStatus::Created,
                    bybit::enums::OrderStatus::Cancelled |
                    bybit::enums::OrderStatus::Deactivated =>
                        OrderStatus::Canceled,
                    bybit::enums::OrderStatus::New |
                    bybit::enums::OrderStatus::PartiallyFilled |
                    bybit::enums::OrderStatus::Triggered =>
                        OrderStatus::InProgress,
                    bybit::enums::OrderStatus::Filled |
                    bybit::enums::OrderStatus::PartiallyFilledCanceled =>
                        OrderStatus::Completed,
                    bybit::enums::OrderStatus::Rejected => OrderStatus::Failed("Rejected".into())
                };
                let pair = {
                    CurrencyPair {
                        target: Currency::from_str(&item.symbol[0..2]).unwrap_or(Currency::from_str(&item.symbol[0..3]).unwrap()),
                        source: Currency::from_str(&item.symbol[2..]).unwrap_or(Currency::from_str(&item.symbol[3..]).unwrap()),
                    }
                };
                let market_type = if item.is_leverage == "1" {
                    OrderMarketType::Margin(MarginMode::Cross(pair.source))
                } else {
                    OrderMarketType::Spot
                };
                let side = match item.side {
                    bybit::enums::Side::Buy => Side::Buy,
                    bybit::enums::Side::Sell => Side::Sell,
                };
                let order_type = match item.order_type {
                    bybit::enums::OrderType::Market => OrderType::Market,
                    bybit::enums::OrderType::Limit => OrderType::Limit(item.price),
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

                let order = Order {
                    id: item.order_link_id,
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
                };
                orders.push(order);
            }
        }
        Some(orders)
    }

    async fn handle(&mut self, message: Self::Type) {
        for order in message {
            self.storage_client.save_order(order).await.unwrap();
        }
    }
}
