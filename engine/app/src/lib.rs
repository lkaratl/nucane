use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use tracing::{debug, error, info};
use uuid::Uuid;

use anyhow::Result;
use tokio::sync::MutexGuard;
use domain_model::{Action, PluginEvent, PluginEventType, Tick};
use engine_core::executor::Executor;
use engine_core::registry::Deployment;
use engine_core::service::{EngineError, EngineService};
use engine_rest_api::dto::{CreateDeploymentDto, DeploymentInfo};
use engine_rest_api::endpoints::{POST_CREATE_ACTIONS, GET_POST_DEPLOYMENTS, DELETE_DEPLOYMENTS_BY_ID};
use registry_rest_client::RegistryClient;
use synapse::{SynapseListen, Topic};

use crate::config::CONFIG;

pub mod config;

pub async fn run() {
    info!("+ engine running...");
    let executor = Arc::new(Executor::default());
    let registry_client = Arc::new(RegistryClient::new("http://localhost:8085"));
    let engine_service = Arc::new(EngineService::new(Arc::clone(&registry_client)));
    listen_ticks(Arc::clone(&executor)).await;
    // listen_plugins(Arc::clone(&engine_service)); // todo fix problem with tokio async runtime

    let router = Router::new()
        .route(GET_POST_DEPLOYMENTS, get(get_deployments).post(create_deployment))
        .route(&format!("{}{}", DELETE_DEPLOYMENTS_BY_ID, "/:id"), delete(delete_deployment))
        .with_state(Arc::clone(&engine_service))
        .with_state(Arc::clone(&registry_client))
        .route(POST_CREATE_ACTIONS, post(create_actions))
        .with_state(Arc::clone(&executor));

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), CONFIG.application.port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn listen_ticks(executor: Arc<Executor>) {
    synapse::reader(&CONFIG.application.name)
        .on(Topic::Tick, move |tick: Tick| {
            let executor = Arc::clone(&executor);
            async move {
                executor.handle(&tick).await;
            }
        }).await;
}

#[allow(unused)]
async fn listen_plugins(engine_service: Arc<EngineService>) {
    synapse::reader(&CONFIG.application.name)
        .on(Topic::Plugin, move |plugin_event: PluginEvent| {
            let engine_service = Arc::clone(&engine_service);
            async move {
                if plugin_event.event == PluginEventType::Updated {
                    let name = &plugin_event.strategy_name;
                    let version = &plugin_event.strategy_version;
                    debug!("Received update event for plugin with name: '{}', version: '{}'", name, version);
                    if let Some(err) = engine_service.update_plugin(name, version).await.err() {
                        error!("Error during update deployment with name: '{}', version: '{}'. Error: '{}'", name, version, err);
                    } else { info!("Deployments for plugin with name: '{}', version: '{}' successfully updated", name, version); }
                }
            }
        }).await;
}

async fn get_deployments(State(engine_service): State<Arc<EngineService>>) -> Json<Vec<DeploymentInfo>> {
    let mut result = Vec::new();
    for deployment in engine_service.get_deployments().await {
        let deployment = deployment.lock().await;
        result.push(convert_to_deployment_info(deployment));
    }
    Json(result)
}

async fn create_deployment(State(engine_service): State<Arc<EngineService>>, Json(request): Json<Vec<CreateDeploymentDto>>) -> Result<Json<Vec<DeploymentInfo>>, StatusCode> {
    let mut result = Vec::new();
    for create_deployment in request {
        let deployment = engine_service.add_deployment(create_deployment.simulation_id, &create_deployment.strategy_name, &create_deployment.strategy_version, &create_deployment.params).await
            .map_err(|err|
                match err {
                    EngineError::PluginNotFound => StatusCode::NOT_FOUND,
                    EngineError::PluginLoadingError => StatusCode::INTERNAL_SERVER_ERROR
                })?;
        let deployment_info = convert_to_deployment_info(deployment.lock().await);
        result.push(deployment_info);
    }
    Ok(Json(result))
}

async fn delete_deployment(State(engine_service): State<Arc<EngineService>>, Path(deployment_id): Path<String>) -> Json<Option<DeploymentInfo>> {
    debug!("Request to delete deployment with id: '{deployment_id}'");
    let deployment = engine_service.remove_deployment_by_id(Uuid::from_str(&deployment_id).unwrap()).await;
    let deployment_info = if let Some(deployment) = deployment {
        Some(convert_to_deployment_info(deployment.lock().await))
    } else {
        None
    };
    Json(deployment_info)
}

async fn create_actions(State(executor): State<Arc<Executor>>, Json(request): Json<Tick>) -> Json<Vec<Action>> {
    let response = executor.get_actions(&request).await;
    Json(response)
}

fn convert_to_deployment_info(value: MutexGuard<Deployment>) -> DeploymentInfo {
    DeploymentInfo {
        id: value.id,
        simulation_id: value.simulation_id,
        strategy_name: value.plugin.strategy.name(),
        strategy_version: value.plugin.strategy.version(),
        params: value.params.clone(),
        subscriptions: value.plugin.strategy.subscriptions(),
    }
}
