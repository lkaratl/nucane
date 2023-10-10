use uuid::Uuid;

use domain_model::drawing::{Line, Point};
use domain_model::InstrumentId;
use storage_persistence_api::DrawingRepository;

pub struct DrawingService<R: DrawingRepository> {
    repository: R,
}

impl<R: DrawingRepository> DrawingService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn save_point(&self, point: Point) {
        self.repository
            .save_point(point)
            .await
            .expect("Error during point saving");
    }

    pub async fn get_points(
        &self,
        instrument_id: &InstrumentId,
        simulation_id: Option<Uuid>,
    ) -> Vec<Point> {
        self.repository
            .get_points(instrument_id, simulation_id)
            .await
            .unwrap()
    }

    pub async fn save_line(&self, line: Line) {
        self.repository
            .save_line(line)
            .await
            .expect("Error during line saving");
    }

    pub async fn get_lines(
        &self,
        instrument_id: &InstrumentId,
        simulation_id: Option<Uuid>,
    ) -> Vec<Line> {
        self.repository
            .get_lines(instrument_id, simulation_id)
            .await
            .unwrap()
    }
}
