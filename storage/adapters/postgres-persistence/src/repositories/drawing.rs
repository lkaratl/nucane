use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveValue, ColumnTrait, Condition, ConnectionTrait, EntityTrait};
use serde_json::json;
use uuid::Uuid;

use domain_model::InstrumentId;
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
            instrument_id: ActiveValue::Set(json!(point.instrument_id)),
            simulation_id: ActiveValue::Set(point.simulation_id),
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
        instrument_id: &InstrumentId,
        simulation_id: Option<Uuid>,
    ) -> Result<Vec<domain_model::drawing::Point>> {
        let mut condition =
            Condition::all().add(point::Column::InstrumentId.eq(json!(instrument_id)));
        if let Some(simulation_id) = simulation_id {
            condition = condition.add(point::Column::SimulationId.eq(simulation_id));
        }
        let result = point::Entity::find()
            .filter(condition)
            .all(self.db.deref())
            .await?
            .into_iter()
            .map(|model| domain_model::drawing::Point {
                id: model.id,
                instrument_id: serde_json::from_value(model.instrument_id).unwrap(),
                simulation_id: model.simulation_id,
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
            instrument_id: ActiveValue::Set(json!(line.instrument_id)),
            simulation_id: ActiveValue::Set(line.simulation_id),
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
        instrument_id: &InstrumentId,
        simulation_id: Option<Uuid>,
    ) -> Result<Vec<domain_model::drawing::Line>> {
        let mut condition =
            Condition::all().add(point::Column::InstrumentId.eq(json!(instrument_id)));
        if let Some(simulation_id) = simulation_id {
            condition = condition.add(point::Column::SimulationId.eq(simulation_id));
        }
        let result = line::Entity::find()
            .filter(condition)
            .all(self.db.deref())
            .await?
            .into_iter()
            .map(|model| domain_model::drawing::Line {
                id: model.id,
                instrument_id: serde_json::from_value(model.instrument_id).unwrap(),
                simulation_id: model.simulation_id,
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
