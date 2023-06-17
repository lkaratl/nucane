mod config;

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use axum::{Json, Router};
use axum::extract::{DefaultBodyLimit, Multipart, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use tracing::{error, info};
use registry_core::service::RegistryService;
use registry_rest_api::dto::{PluginBinary, PluginInfo};
use crate::config::CONFIG;
use registry_rest_api::endpoints::*;
use registry_rest_api::path_query::PluginQuery;


pub async fn run() {
    info!("+ registry running...");

    let registry_service = Arc::new(RegistryService::default());

    let router = Router::new()
        .route(GET_PLUGINS_INFO_BY_ID_OR_NAME_OR_VERSION, get(get_plugins_info))
        .route(GET_BINARY_BY_ID_OR_NAME_OR_VERSION, get(get_plugins_binary))
        .route(POST_PLUGINS, post(create_plugins))
        .route(DELETE_PLUGINS_INFO_BY_ID_OR_NAME_OR_VERSION, delete(delete_plugins_by_name_or_version))
        .with_state(Arc::clone(&registry_service))
        .layer(DefaultBodyLimit::max(104_857_600)); // 100mb

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), CONFIG.application.port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn get_plugins_info(State(registry_service): State<Arc<RegistryService>>, Query(query): Query<PluginQuery>) -> Json<Vec<PluginInfo>> {
    let plugins_info;
    if let Some(id) = query.id {
        let plugin = registry_service.get_plugin_by_id(id).await;
        plugins_info = plugin.map(|plugin| vec![plugin.into()])
            .unwrap_or_default();
    } else if query.name.is_none() && query.version.is_none() {
        let plugins = registry_service.get_plugins().await;
        plugins_info = plugins.into_iter()
            .map(PluginInfo::from)
            .collect();
    } else if query.version.is_none() {
        let plugins_by_name = registry_service.get_plugins_by_name(&query.name.unwrap()).await;
        plugins_info = plugins_by_name.into_iter()
            .map(PluginInfo::from)
            .collect();
    } else {
        plugins_info = registry_service.get_plugin_by_name_and_version(&query.name.unwrap(), &query.version.unwrap()).await
            .map(PluginInfo::from)
            .map(|plugin_info| vec![plugin_info])
            .unwrap_or_default();
    }
    Json(plugins_info)
}

async fn get_plugins_binary(State(registry_service): State<Arc<RegistryService>>, Query(query): Query<PluginQuery>) -> Json<Vec<PluginBinary>> {
    let plugins_binary;
    if let Some(id) = query.id {
        let plugin = registry_service.get_plugin_by_id(id).await;
        plugins_binary = plugin.map(|plugin| vec![plugin.into()])
            .unwrap_or_default();
    } else if query.name.is_none() && query.version.is_none() {
        let plugins = registry_service.get_plugins().await;
        plugins_binary = plugins.into_iter()
            .map(PluginBinary::from)
            .collect();
    } else if query.version.is_none() {
        let plugins_by_name = registry_service.get_plugins_by_name(&query.name.unwrap()).await;
        plugins_binary = plugins_by_name.into_iter()
            .map(PluginBinary::from)
            .collect();
    } else {
        plugins_binary = registry_service.get_plugin_by_name_and_version(&query.name.unwrap(), &query.version.unwrap()).await
            .map(PluginBinary::from)
            .map(|plugin_info| vec![plugin_info])
            .unwrap_or_default();
    }
    Json(plugins_binary)
}

async fn create_plugins(State(registry_service): State<Arc<RegistryService>>, mut multipart: Multipart) -> Result<Json<Vec<PluginInfo>>, StatusCode> {
    let mut plugins_info = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap().to_string();
        if !is_lib_name_valid(&name) {
            error!("Error during plugin creation: 'Uploaded plugin have different target OS'");
            return Err(StatusCode::BAD_REQUEST);
        }

        let data = field.bytes().await.unwrap();
        let plugin = registry_service.add_plugin(&data).await
            .map_err(|err| {
                error!("Error during plugin creation: {err}");
                StatusCode::BAD_REQUEST
            })?;
        plugins_info.push(plugin.into());
    }
    Ok(Json(plugins_info))
}

fn is_lib_name_valid(name: &str) -> bool {
    cfg!(target_os = "windows") && name.ends_with(".dll")
}

async fn delete_plugins_by_name_or_version(State(registry_service): State<Arc<RegistryService>>, Query(query): Query<PluginQuery>) -> Result<Json<Vec<PluginInfo>>, StatusCode> {
    let plugins_info;
    if let Some(id) = query.id {
        let plugin = registry_service.get_plugin_by_id(id).await;
        plugins_info = plugin.map(|plugin| vec![plugin.into()])
            .unwrap_or_default();
    } else if query.name.is_none() && query.version.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    } else if query.version.is_none() {
        let plugins_by_name = registry_service.remove_plugins_by_name(&query.name.unwrap()).await;
        plugins_info = plugins_by_name.into_iter()
            .map(PluginInfo::from)
            .collect();
    } else {
        plugins_info = registry_service.remove_plugin_by_name_and_version(&query.name.unwrap(), &query.version.unwrap()).await
            .map(PluginInfo::from)
            .map(|plugin_info| vec![plugin_info])
            .unwrap_or_default();
    }
    Ok(Json(plugins_info))
}
