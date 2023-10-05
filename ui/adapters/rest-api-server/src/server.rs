use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use tracing::error;
use uuid::Uuid;

use ui_core_api::UiApi;
use ui_rest_api::endpoints::GET_SIMULATION_CHART;
use ui_rest_api::path_queries::SimulationChartQuery;

pub async fn run(port: u16, ui: impl UiApi) {
    let ui = Arc::new(ui);
    let router = Router::new()
        .route(GET_SIMULATION_CHART, get(get_simulation_chart))
        .with_state(ui);

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn get_simulation_chart(
    State(ui): State<Arc<dyn UiApi>>,
    Query(query): Query<SimulationChartQuery>,
    Path(simulation_id): Path<Uuid>,
) -> Html<String> {
    let chart_html = ui
        .get_simulation_chart_html(simulation_id, query.timeframe)
        .await
        .map_err(|err| error!("Error during simulation chart building: '{err}'"))
        .unwrap_or("<p>Error during chart building</p>".to_string());
    Html(chart_html)
}
