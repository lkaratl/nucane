use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{ActiveValue, ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryOrder, sea_query};
use serde_json::json;
use uuid::Uuid;
use sea_orm::QueryFilter;

use domain_model::{Currency, Exchange, MarketType, OrderStatus, OrderType, Side};
use storage_persistence_api::OrderRepository;

use crate::entities::{*};
use crate::entities::prelude::Order;

pub struct OrderPostgresRepository<T: ConnectionTrait> {
    db: Arc<T>,
}

impl<T: ConnectionTrait> OrderPostgresRepository<T> {
    pub fn new(db: Arc<T>) -> Self {
        Self {
            db
        }
    }
}

#[async_trait]
impl<T: ConnectionTrait+ Send +'static> OrderRepository for OrderPostgresRepository<T> {
    async fn save(&self, order: domain_model::Order) -> Result<()> {
        let order = order::ActiveModel {
            id: ActiveValue::Set(order.id),
            timestamp: ActiveValue::Set(order.timestamp.naive_utc()),
            simulation_id: ActiveValue::Set(order.simulation_id),
            status: ActiveValue::Set(json!(order.status)),
            exchange: ActiveValue::Set(order.exchange.to_string()),
            pair: ActiveValue::Set(json!(order.pair)),
            market_type: ActiveValue::Set(json!(order.market_type)),
            order_type: ActiveValue::Set(json!(order.order_type)),
            side: ActiveValue::Set(order.side.to_string()),
            size: ActiveValue::Set(json!(order.size)),
            avg_price: ActiveValue::Set(order.avg_price),
            stop_loss: ActiveValue::Set(order.stop_loss.map(|sl| json!(sl))),
            take_profit: ActiveValue::Set(order.take_profit.map(|tp| json!(tp))),
        };
        Order::insert(order)
            .on_conflict(
                sea_query::OnConflict::column(order::Column::Id)
                    .update_columns(vec![
                        order::Column::Status,
                        order::Column::OrderType,
                        order::Column::Size,
                        order::Column::AvgPrice,
                        order::Column::StopLoss,
                        order::Column::TakeProfit,
                    ]).to_owned()
            )
            .exec(self.db.deref())
            .await?;
        Ok(())
    }

    async fn get(&self, id: Option<String>, exchange: Option<Exchange>, market_type: Option<MarketType>, target: Option<Currency>, source: Option<Currency>, status: Option<OrderStatus>, side: Option<Side>, order_type: Option<OrderType>) -> Result<Vec<domain_model::Order>> {
        let mut condition = Condition::all();
        if let Some(id) = id {
            condition = condition.add(order::Column::Id.eq(id));
        }
        if let Some(exchange) = exchange {
            condition = condition.add(order::Column::Exchange.eq(exchange.to_string()));
        }
        if let Some(target) = target {
            condition = condition.add(order::Column::Pair.contains(&target.to_string())); // todo make pair flatten
        }
        if let Some(source) = source {
            condition = condition.add(order::Column::Pair.contains(&source.to_string()));
        }
        if let Some(market_type) = market_type {
            condition = condition.add(order::Column::MarketType.eq(market_type.to_string()));
        }
        if let Some(status) = status {
            condition = condition.add(order::Column::Status.contains(&serde_json::to_string(&status).unwrap()));
        }
        if let Some(side) = side {
            condition = condition.add(order::Column::Side.eq(side.to_string()));
        }
        if let Some(order_type) = order_type {
            condition = condition.add(order::Column::OrderType.eq(json!(order_type)));
        }
        let result = order::Entity::find()
            .filter(condition)
            .order_by_asc(order::Column::Timestamp)
            .all(self.db.deref())
            .await?
            .into_iter()
            .map(|model| {
                domain_model::Order {
                    id: model.id,
                    timestamp: model.timestamp.and_utc(),
                    simulation_id: model.simulation_id,
                    status: serde_json::from_value(model.status).unwrap(),
                    exchange: Exchange::from_str(&model.exchange).unwrap(),
                    pair: serde_json::from_value(model.pair).unwrap(),
                    market_type: serde_json::from_value(model.market_type).unwrap(),
                    order_type: serde_json::from_value(model.order_type).unwrap(),
                    side: Side::from_str(&model.side).unwrap(),
                    size: serde_json::from_value(model.size).unwrap(),
                    avg_price: model.avg_price,
                    stop_loss: model.stop_loss.map(|sl| serde_json::from_value(sl).unwrap()),
                    take_profit: model.take_profit.map(|tp| serde_json::from_value(tp).unwrap()),
                }
            }).collect();
        Ok(result)
    }
}

