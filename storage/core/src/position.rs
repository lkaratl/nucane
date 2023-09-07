use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use sea_orm::{ActiveValue, Condition, ConnectionTrait, DbErr, EntityTrait, sea_query, QueryFilter, ColumnTrait};
use uuid::Uuid;
use domain_model::{Currency, Exchange, Side};

use crate::entities::{*, prelude::*};

pub struct PositionService<T: ConnectionTrait> {
    repository: PositionRepository<T>,
}

impl<T: ConnectionTrait> PositionService<T> {
    pub fn new(db: Arc<T>) -> Self {
        PositionService { repository: PositionRepository { db } }
    }

    pub async fn save(&self, position: domain_model::Position) {
        self.repository.save(position)
            .await
            .expect("Error during position saving");
    }

    pub async fn get(&self,
               exchange: Option<Exchange>,
               currency: Option<Currency>,
               side: Option<Side>) -> Vec<domain_model::Position> {
        self.repository.find_by(exchange, currency, side).await.unwrap()
    }
}

struct PositionRepository<T: ConnectionTrait> {
    db: Arc<T>,
}

impl<T: ConnectionTrait> PositionRepository<T> {
    async fn save(&self, position: domain_model::Position) -> Result<(), DbErr> {
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
            simulation_id: ActiveValue::Set(position.simulation_id.map(|id| id.as_bytes().to_vec())),
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

    async fn find_by(&self,
               exchange: Option<Exchange>,
               currency: Option<Currency>,
               side: Option<Side>) -> Result<Vec<domain_model::Position>, DbErr> {
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
                    simulation_id: model.simulation_id.map(|id| Uuid::from_slice(&id).unwrap()),
                    exchange: Exchange::from_str(&model.exchange).unwrap(),
                    currency: Currency::from_str(&model.currency).unwrap(),
                    side: Side::from_str(&model.side).unwrap(),
                    size: model.size,
                }
            }).collect();

        Ok(result)
    }
}
