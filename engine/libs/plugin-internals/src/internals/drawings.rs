use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use plugin_api::{DrawingsInternalApi, Line, Point};
use storage_core_api::StorageApi;

pub struct DefaultDrawingInternals<S: StorageApi> {
    deployment_id: Uuid,
    storage_client: Arc<S>,
}

impl<S: StorageApi> DefaultDrawingInternals<S> {
    pub fn new(deployment_id: Uuid, storage_client: Arc<S>) -> Self {
        Self {
            deployment_id,
            storage_client,
        }
    }
}

#[async_trait]
impl<S: StorageApi> DrawingsInternalApi for DefaultDrawingInternals<S> {
    async fn save_point(&self, point: Point) {
        let point = convert_point(point, self.deployment_id);
        self.storage_client.save_point(point).await.unwrap();
    }

    async fn save_line(&self, line: Line) {
        let line = convert_line(line, self.deployment_id);
        self.storage_client.save_line(line).await.unwrap();
    }
}

fn convert_point(point: Point, deployment_id: Uuid) -> domain_model::drawing::Point {
    domain_model::drawing::Point::new(
        point.instrument_id,
        deployment_id,
        &point.label,
        point.icon,
        point.color,
        point.text,
        point.coord,
    )
}

fn convert_line(line: Line, deployment_id: Uuid) -> domain_model::drawing::Line {
    domain_model::drawing::Line::new(
        line.instrument_id,
        deployment_id,
        &line.label,
        line.style,
        line.color,
        line.start,
        line.end,
    )
}
