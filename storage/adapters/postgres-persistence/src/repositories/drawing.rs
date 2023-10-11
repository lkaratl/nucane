use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveValue, ColumnTrait, Condition, ConnectionTrait, EntityTrait};
use serde_json::json;
use uuid::Uuid;

use storage_persistence_api::DrawingRepository;

use crate::entities::prelude::{Line, Point};
use crate::entities::{line, point};

pub struct DrawingPostgresRepository<T: ConnectionTrait> {
    db: Arc<T>,
}

impl<T: ConnectionTrait> DrawingPostgresRepository<T> {
    pub fn new(db: Arc<T>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<T: ConnectionTrait + Send + 'static> DrawingRepository for DrawingPostgresRepository<T> {
    async fn save_point(&self, point: domain_model::drawing::Point) -> Result<()> {
        let point = point::ActiveModel {
            id: ActiveValue::Set(point.id),
            deployment_id: ActiveValue::Set(point.deployment_id),
            pair: ActiveValue::Set(serde_json::to_string(&point.instrument_id.pair)?),
            exchange: ActiveValue::Set(point.instrument_id.exchange.to_string()),
            market_type: ActiveValue::Set(point.instrument_id.market_type.to_string()),
            label: ActiveValue::Set(point.label),
            icon: ActiveValue::Set(point.icon.map(|icon| json!(icon))),
            color: ActiveValue::Set(point.color.map(|color| json!(color))),
            text: ActiveValue::Set(point.text.map(|text| json!(text))),
            coord: ActiveValue::Set(json!(point.coord)),
        };
        Point::insert(point).exec(self.db.deref()).await?;
        Ok(())
    }

    async fn get_points(
        &self,
        deployment_id: Uuid,
        instrument_id: &domain_model::InstrumentId,
    ) -> Result<Vec<domain_model::drawing::Point>> {
        let condition = Condition::all()
            .add(point::Column::DeploymentId.eq(deployment_id))
            .add(point::Column::Exchange.eq(instrument_id.exchange.to_string()))
            .add(point::Column::Pair.contains(&instrument_id.pair.target.to_string())) // todo pair flatten in db model
            .add(point::Column::Pair.contains(&instrument_id.pair.source.to_string()))
            .add(point::Column::MarketType.eq(instrument_id.market_type.to_string()));
        let result = point::Entity::find()
            .filter(condition)
            .all(self.db.deref())
            .await?
            .into_iter()
            .map(|model| domain_model::drawing::Point {
                id: model.id,
                deployment_id: model.deployment_id,
                instrument_id: domain_model::InstrumentId {
                    exchange: domain_model::Exchange::from_str(&model.exchange).unwrap(),
                    market_type: domain_model::MarketType::from_str(&model.market_type).unwrap(),
                    pair: serde_json::from_str(&model.pair).unwrap(),
                },
                label: model.label,
                icon: model.icon.map(|icon| serde_json::from_value(icon).unwrap()),
                color: model
                    .color
                    .map(|color| serde_json::from_value(color).unwrap()),
                text: model.text.map(|text| serde_json::from_value(text).unwrap()),
                coord: serde_json::from_value(model.coord).unwrap(),
            })
            .collect();
        Ok(result)
    }

    async fn save_line(&self, line: domain_model::drawing::Line) -> Result<()> {
        let line = line::ActiveModel {
            id: ActiveValue::Set(line.id),
            deployment_id: ActiveValue::Set(line.deployment_id),
            pair: ActiveValue::Set(serde_json::to_string(&line.instrument_id.pair)?),
            exchange: ActiveValue::Set(line.instrument_id.exchange.to_string()),
            market_type: ActiveValue::Set(line.instrument_id.market_type.to_string()),
            label: ActiveValue::Set(line.label),
            style: ActiveValue::Set(line.style.map(|style| json!(style))),
            color: ActiveValue::Set(line.color.map(|color| json!(color))),
            start: ActiveValue::Set(json!(line.start)),
            end: ActiveValue::Set(json!(line.end)),
        };
        Line::insert(line).exec(self.db.deref()).await?;
        Ok(())
    }

    async fn get_lines(
        &self,
        deployment_id: Uuid,
        instrument_id: &domain_model::InstrumentId,
    ) -> Result<Vec<domain_model::drawing::Line>> {
        let condition = Condition::all()
            .add(point::Column::DeploymentId.eq(deployment_id))
            .add(point::Column::Exchange.eq(instrument_id.exchange.to_string()))
            .add(point::Column::Pair.contains(&instrument_id.pair.target.to_string())) // todo pair flatten in db model
            .add(point::Column::Pair.contains(&instrument_id.pair.source.to_string()))
            .add(point::Column::MarketType.eq(instrument_id.market_type.to_string()));
        let result = line::Entity::find()
            .filter(condition)
            .all(self.db.deref())
            .await?
            .into_iter()
            .map(|model| domain_model::drawing::Line {
                id: model.id,
                deployment_id: model.deployment_id,
                instrument_id: domain_model::InstrumentId {
                    exchange: domain_model::Exchange::from_str(&model.exchange).unwrap(),
                    market_type: domain_model::MarketType::from_str(&model.market_type).unwrap(),
                    pair: serde_json::from_str(&model.pair).unwrap(),
                },
                label: model.label,
                style: model
                    .style
                    .map(|style| serde_json::from_value(style).unwrap()),
                color: model
                    .color
                    .map(|color| serde_json::from_value(color).unwrap()),
                start: serde_json::from_value(model.start).unwrap(),
                end: serde_json::from_value(model.end).unwrap(),
            })
            .collect();
        Ok(result)
    }
}
