use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::routing::{get, post, put};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};
use tracing::log::trace;
use uuid::Uuid;

use anyhow::Result;
use domain_model::{Action, DeploymentEvent, InstrumentId, PluginEvent, Tick};
use registry_rest_client::RegistryClient;
use synapse::{SynapseListen, SynapseSend, Topic};

use crate::config::CONFIG;
use crate::executor::Executor;
use crate::registry::Deployment;

mod executor;
mod registry;
pub mod config;
pub mod utils;

const DEPLOYMENTS_PATH_URL: &str = "/api/v1/engine/deployments";
const ACTIONS_PATH_URL: &str = "/api/v1/engine/actions";

pub async fn run() {
    info!("+ engine running...");
    let executor = Arc::new(Executor::default());
    let registry_client = Arc::new(RegistryClient::new("http://localhost:8085"));
    listen_ticks(Arc::clone(&executor));

    let router = Router::new()
        .route(DEPLOYMENTS_PATH_URL, get(get_deployments).post(create_deployment))
        .route(&format!("{}{}", DEPLOYMENTS_PATH_URL, "/:id"), put(replace_deployment).delete(delete_deployment))
        .with_state(Arc::clone(&registry_client))
        .route(ACTIONS_PATH_URL, post(create_actions))
        .with_state(Arc::clone(&executor));

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), CONFIG.application.port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

fn listen_ticks(executor: Arc<Executor>) {
    synapse::reader(&CONFIG.application.name)
        .on(Topic::Tick, move |tick: Tick| executor.handle(&tick));
}

#[derive(Debug, Deserialize, Serialize)]
struct DeploymentInfo {
    pub id: Uuid,
    pub simulation_id: Option<Uuid>,
    pub strategy_id: String,
    pub strategy_version: String,
    pub params: HashMap<String, String>,
    pub subscriptions: Vec<InstrumentId>,
}

impl From<&Deployment> for DeploymentInfo {
    fn from(value: &Deployment) -> Self {
        DeploymentInfo {
            id: value.id,
            simulation_id: value.simulation_id,
            strategy_id: value.plugin.strategy.name(),
            strategy_version: value.plugin.strategy.version(),
            params: value.params.clone(),
            subscriptions: value.plugin.strategy.subscriptions(),
        }
    }
}

async fn get_deployments() -> Json<Vec<DeploymentInfo>> {
    let deployments: Vec<DeploymentInfo> = registry::get_deployments()
        .await
        .lock()
        .await
        .iter()
        .map(|deployment| deployment.into())
        .collect();
    Json(deployments)
}

#[derive(Deserialize, Serialize, Debug)]
struct CreateDeployment {
    pub simulation_id: Option<Uuid>,
    pub strategy_name: String,
    pub strategy_version: String,
    pub params: HashMap<String, String>,
}

// todo business logic in handlers, move business logic to  service
async fn create_deployment(State(registry_client): State<Arc<RegistryClient>>, Json(request): Json<CreateDeployment>) -> Result<Json<DeploymentInfo>, StatusCode> {
    debug!("Create deployment request, for strategy with name: '{}' and version: '{}'", request.strategy_name, request.strategy_version);
    trace!("Deployment body: {request:?}");
    let params = request.params;

    let plugin = registry_client.get_binary(None, Some(request.strategy_name), Some(request.strategy_version)).await
        .map_err(|err| {
            error!("Error during loading plugin from registry: '{err}'");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let plugin = plugin.first().ok_or(StatusCode::NOT_FOUND)?;
    let mut plugin = plugin_loader::load(&plugin.binary)
        .map_err(|err|{
            error!("Error during plugin initiation: '{err}'");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    plugin.strategy.tune(&params);

    let deployment = Deployment {
        id: Uuid::new_v4(),
        simulation_id: request.simulation_id,
        params,
        plugin,
    };

    let deployment_info = DeploymentInfo::from(&deployment);
    synapse::writer().send(&deployment_create_event(&deployment)); // todo send event after success
    registry::add_deployment(deployment).await;
    Ok(Json(deployment_info))
}

async fn replace_deployment(State(registry_client): State<Arc<RegistryClient>>, Path(deployment_id): Path<String>, Json(request): Json<CreateDeployment>) -> Result<(), StatusCode> {
    let params = request.params;

    let plugin = registry_client.get_binary(None, Some(request.strategy_name), Some(request.strategy_version)).await
        .map_err(|err| {
            error!("Error during loading plugin from registry: '{err}'");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let plugin = plugin.first().ok_or(StatusCode::NOT_FOUND)?;
    let mut plugin = plugin_loader::load(&plugin.binary)
        .map_err(|err|{
            error!("Error during plugin initiation: '{err}'");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    plugin.strategy.tune(&params);

    let deployment = Deployment {
        id: Uuid::new_v4(),
        simulation_id: request.simulation_id,
        params,
        plugin,
    };
    let deleted_deployment = registry::delete_deployment(&Uuid::from_str(&deployment_id).unwrap()).await;
    if let Some(deleted_deployment) = deleted_deployment {
        synapse::writer().send(&deployment_delete_event(&deleted_deployment));
    }
    synapse::writer().send(&deployment_create_event(&deployment)); // todo send event after success
    registry::add_deployment(deployment).await;
    Ok(())
}

async fn delete_deployment(Path(deployment_id): Path<String>) {
    debug!("Request to delete deployment with id: '{deployment_id}'");
    let deployment = registry::delete_deployment(&Uuid::from_str(&deployment_id).unwrap()).await;
    if let Some(deployment) = deployment {
        synapse::writer().send(&deployment_delete_event(&deployment));
    }
}

async fn create_actions(State(executor): State<Arc<Executor>>, Json(request): Json<Tick>) -> Json<Vec<Action>> {
    let response = executor.get_actions(&request).await;
    Json(response)
}

fn deployment_create_event(deployment: &Deployment) -> domain_model::Deployment {
    domain_model::Deployment {
        id: deployment.id,
        simulation_id: deployment.simulation_id,
        event: DeploymentEvent::Created,
        strategy_name: deployment.plugin.strategy.name(),
        strategy_version: deployment.plugin.strategy.version(),
        params: deployment.params.clone(),
        subscriptions: deployment.plugin.strategy.subscriptions(),
    }
}

fn deployment_delete_event(deployment: &Deployment) -> domain_model::Deployment {
    domain_model::Deployment {
        id: deployment.id,
        simulation_id: deployment.simulation_id,
        event: DeploymentEvent::Deleted,
        strategy_name: deployment.plugin.strategy.name(),
        strategy_version: deployment.plugin.strategy.version(),
        params: deployment.params.clone(),
        subscriptions: deployment.plugin.strategy.subscriptions(),
    }
}
