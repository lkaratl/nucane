use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use tracing::debug;
use uuid::Uuid;

use domain_model::{Action, DeploymentInfo, PluginId, Tick};
use engine_core_api::api::{EngineApi, EngineError};
use engine_rest_api::dtos::CreateDeploymentDto;
use engine_rest_api::endpoints::{DELETE_DEPLOYMENT, GET_DEPLOYMENTS, POST_CREATE_ACTIONS, POST_CREATE_DEPLOYMENTS, PUT_UPDATE_PLUGIN};

pub async fn run(port: u16, engine: impl EngineApi) {
    let engine = Arc::new(engine);
    let router = Router::new()
        .route(GET_DEPLOYMENTS, get(get_deployments))
        .route(POST_CREATE_DEPLOYMENTS, post(create_deployment))
        .route(DELETE_DEPLOYMENT, delete(delete_deployment))
        .route(POST_CREATE_ACTIONS, post(create_actions))
        .route(PUT_UPDATE_PLUGIN, put(update_plugin))
        .with_state(engine);

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn get_deployments(State(engine): State<Arc<dyn EngineApi>>) -> Json<Vec<DeploymentInfo>> {
    let result = engine.get_deployments_info().await;
    Json(result)
}

async fn create_deployment(State(engine): State<Arc<dyn EngineApi>>, Json(request): Json<Vec<CreateDeploymentDto>>) -> Result<Json<Vec<DeploymentInfo>>, StatusCode> {
    let mut result = Vec::new();
    for create_deployment in request {
        let deployment_info = engine
            .deploy(create_deployment.simulation_id, &create_deployment.name, create_deployment.version, &create_deployment.params)
            .await
            .map_err(|err|
                match err {
                    EngineError::PluginNotFound => StatusCode::NOT_FOUND,
                    EngineError::PluginLoadingError => StatusCode::INTERNAL_SERVER_ERROR
                })?;
        result.push(deployment_info);
    }
    Ok(Json(result))
}

async fn delete_deployment(State(engine): State<Arc<dyn EngineApi>>, Path(deployment_id): Path<String>) -> Json<Option<DeploymentInfo>> {
    debug!("Request to delete deployment with id: '{deployment_id}'");
    let deployment_id = Uuid::from_str(&deployment_id).unwrap();
    let deployment_info = engine.delete_deployment(deployment_id)
        .await;
    Json(deployment_info)
}

async fn create_actions(State(engine): State<Arc<dyn EngineApi>>, Json(request): Json<Tick>) -> Json<Vec<Action>> {
    let response = engine.get_actions(&request).await;
    Json(response)
}

async fn update_plugin(State(engine): State<Arc<dyn EngineApi>>, Json(request): Json<PluginId>) {
    engine.update_plugin(request).await;
}
