use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use futures::executor::block_on;
use sea_orm::{ActiveValue, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, sea_query};
use serde_json::json;
use uuid::Uuid;

use domain_model::{Currency, Exchange, MarketType, OrderStatus, OrderType, Side};

use crate::entities::{*, prelude::*};

pub struct OrderService<T: ConnectionTrait> {
    repository: OrderRepository<T>,
}

impl<T: ConnectionTrait> OrderService<T> {
    pub fn new(db: Arc<T>) -> Self {
        OrderService { repository: OrderRepository { db } }
    }

    pub fn save(&self, order: domain_model::Order) {
        self.repository.save(order)
            .expect("Error during order saving");
    }

    pub fn get(&self,
               id: Option<String>,
               exchange: Option<Exchange>,
               market_type: Option<MarketType>,
               target: Option<Currency>,
               source: Option<Currency>,
               status: Option<OrderStatus>,
               side: Option<Side>,
               order_type: Option<OrderType>) -> Vec<domain_model::Order> {
        self.repository.find_by(
            id,
            exchange,
            market_type,
            target,
            source,
            status,
            side,
            order_type,
        ).unwrap()
    }
}

struct OrderRepository<T: ConnectionTrait> {
    db: Arc<T>,
}

impl<T: ConnectionTrait> OrderRepository<T> {
    fn save(&self, order: domain_model::Order) -> Result<(), DbErr> {
        let order = order::ActiveModel {
            id: ActiveValue::Set(order.id),
            timestamp: ActiveValue::Set(order.timestamp),
            simulation_id: ActiveValue::Set(order.simulation_id.map(|id| id.as_bytes().to_vec())),
            status: ActiveValue::Set(json!(order.status)),
            exchange: ActiveValue::Set(order.exchange.to_string()),
            pair: ActiveValue::Set(json!(order.pair)),
            market_type: ActiveValue::Set(json!(order.market_type)),
            order_type: ActiveValue::Set(json!(order.order_type)),
            side: ActiveValue::Set(order.side.to_string()),
            size: ActiveValue::Set(order.size),
            avg_price: ActiveValue::Set(order.avg_price)
        };
        block_on(Order::insert(order)
            .on_conflict(
                sea_query::OnConflict::column(order::Column::Id)
                    .update_columns(vec![
                        order::Column::Status,
                        order::Column::OrderType,
                        order::Column::Size,
                        order::Column::AvgPrice
                    ]).to_owned()
            )
            .exec(self.db.deref()))?;
        Ok(())
    }

    fn find_by(&self,
               id: Option<String>,
               exchange: Option<Exchange>,
               market_type: Option<MarketType>,
               target: Option<Currency>,
               source: Option<Currency>,
               status: Option<OrderStatus>,
               side: Option<Side>,
               order_type: Option<OrderType>) -> Result<Vec<domain_model::Order>, DbErr> {
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
        let result = block_on(order::Entity::find()
            .filter(condition)
            .order_by_asc(order::Column::Timestamp)
            .all(self.db.deref()))?
            .into_iter()
            .map(|model| {
                domain_model::Order {
                    id: model.id,
                    timestamp: model.timestamp,
                    simulation_id: model.simulation_id.map(|id| Uuid::from_slice(&id).unwrap()),
                    status: serde_json::from_value(model.status).unwrap(),
                    exchange: Exchange::from_str(&model.exchange).unwrap(),
                    pair: serde_json::from_value(model.pair).unwrap(),
                    market_type: serde_json::from_value(model.market_type).unwrap(),
                    order_type: serde_json::from_value(model.order_type).unwrap(),
                    side: Side::from_str(&model.side).unwrap(),
                    size: model.size,
                    avg_price: model.avg_price
                }
            }).collect();

        Ok(result)
    }
}
