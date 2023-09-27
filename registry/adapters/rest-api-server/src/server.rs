use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{DefaultBodyLimit, Multipart, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use tracing::error;

use domain_model::{Plugin, PluginId, PluginInfo};
use registry_core_api::RegistryApi;
use registry_rest_api::endpoints::{DELETE_PLUGINS, GET_PLUGIN_BINARY, GET_PLUGINS_INFO, POST_CREATE_PLUGINS};
use registry_rest_api::path_query::{AddPluginQuery, PluginQuery};

pub async fn run(port: u16, interactor: impl RegistryApi) {
    let interactor = Arc::new(interactor);
    let router = Router::new()
        .route(GET_PLUGINS_INFO, get(get_plugins_info))
        .route(GET_PLUGIN_BINARY, get(get_plugin_binary))
        .route(POST_CREATE_PLUGINS, post(create_plugins))
        .route(DELETE_PLUGINS, delete(delete_plugin))
        .with_state(interactor)
        .layer(DefaultBodyLimit::max(104_857_600));

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn get_plugins_info(State(registry): State<Arc<dyn RegistryApi>>, Query(query): Query<PluginQuery>) -> Json<Vec<PluginInfo>> {
    let plugins_info = if let Some(name) = query.name {
        registry.get_plugins_info_by_name(&name).await
    } else {
        registry.get_plugins_info().await
    };
    Json(plugins_info)
}

async fn get_plugin_binary(State(registry): State<Arc<dyn RegistryApi>>, Query(query): Query<PluginQuery>) -> Result<Json<Plugin>, StatusCode> {
    let plugin_id = PluginId::new(&query.name.unwrap(), query.version.unwrap());
    registry.get_plugin_binary(plugin_id).await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_plugins(State(registry): State<Arc<dyn RegistryApi>>, Query(query): Query<AddPluginQuery>, mut multipart: Multipart) -> Result<Json<Vec<PluginInfo>>, StatusCode> {
    let mut plugins_info = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap().to_string();
        if !is_lib_name_valid(&name) {
            error!("Error during plugin creation: 'Uploaded plugin have different target OS'");
            return Err(StatusCode::BAD_REQUEST);
        }

        let data = field.bytes().await.unwrap();
        let force = query.force;
        let plugin = registry.add_plugin(&data, force).await
            .map_err(|err| {
                error!("Error during plugin creation: {err}");
                StatusCode::BAD_REQUEST
            })?;
        plugins_info.push(plugin.into());
    }
    Ok(Json(plugins_info))
}

fn is_lib_name_valid(name: &str) -> bool {
    cfg!(target_os = "windows") && name.ends_with(".dll") || cfg!(target_os = "linux") && name.ends_with(".so")
}

async fn delete_plugin(State(registry): State<Arc<dyn RegistryApi>>, Query(query): Query<PluginQuery>) {
    let plugin_id = PluginId::new(&query.name.unwrap(), query.version.unwrap());
    registry.delete_plugin(plugin_id).await;
}
