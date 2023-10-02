use domain_model::{Currency, Exchange, Side};
use storage_persistence_api::PositionRepository;

pub struct PositionService<R: PositionRepository> {
    repository: R,
}

impl<R: PositionRepository> PositionService<R> {
    pub fn new(repository: R) -> Self {
        PositionService { repository }
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
        self.repository.get(exchange, currency, side).await.unwrap()
    }
}
