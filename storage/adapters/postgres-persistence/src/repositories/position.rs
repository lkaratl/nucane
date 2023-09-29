use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{ActiveValue, Condition, ConnectionTrait, EntityTrait, sea_query};
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;

use storage_persistence_api::PositionRepository;

use crate::entities::position;
use crate::entities::prelude::Position;

pub struct PositionPostgresRepository<T: ConnectionTrait> {
    db: Arc<T>,
}

impl<T: ConnectionTrait> PositionPostgresRepository<T> {
    pub fn new(db: Arc<T>) -> Self {
        Self {
            db
        }
    }
}

#[async_trait]
impl<T: ConnectionTrait+Send+'static> PositionRepository for PositionPostgresRepository<T> {
    async fn save(&self, position: domain_model::Position) -> Result<()> {
        let exchange = position.exchange.to_string();
        let currency = position.currency.to_string();
        let id = {
            let mut id = format!("{exchange}_{currency}");
            if let Some(simulation_id) = position.simulation_id {
                id = format!("{id}-{}", simulation_id);
            }
            id
        };
        let position = position::ActiveModel {
            id: ActiveValue::Set(id),
            simulation_id: ActiveValue::Set(position.simulation_id),
            exchange: ActiveValue::Set(exchange),
            currency: ActiveValue::Set(currency),
            side: ActiveValue::Set(position.side.to_string()),
            size: ActiveValue::Set(position.size),
        };
        Position::insert(position)
            .on_conflict(
                sea_query::OnConflict::column(position::Column::Id)
                    .update_columns(vec![
                        position::Column::Size,
                        position::Column::Side,
                    ]).to_owned()
            )
            .exec(self.db.deref())
            .await?;
        Ok(())
    }

    async fn get(&self, exchange: Option<domain_model::Exchange>, currency: Option<domain_model::Currency>, side: Option<domain_model::Side>) -> Result<Vec<domain_model::Position>> {
        let mut condition = Condition::all();
        if let Some(exchange) = exchange {
            condition = condition.add(position::Column::Exchange.eq(exchange.to_string()));
        }
        if let Some(currency) = currency {
            condition = condition.add(position::Column::Currency.eq(currency.to_string()));
        }
        if let Some(side) = side {
            condition = condition.add(position::Column::Side.eq(side.to_string()));
        }
        let result = position::Entity::find()
            .filter(condition)
            .all(self.db.deref())
            .await?
            .into_iter()
            .map(|model| {
                domain_model::Position {
                    id: model.id,
                    simulation_id: model.simulation_id,
                    exchange: domain_model::Exchange::from_str(&model.exchange).unwrap(),
                    currency: domain_model::Currency::from_str(&model.currency).unwrap(),
                    side: domain_model::Side::from_str(&model.side).unwrap(),
                    size: model.size,
                }
            }).collect();
        Ok(result)
    }
}
