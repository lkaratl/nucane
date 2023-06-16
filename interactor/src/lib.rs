use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;

use axum::{Json, Router};
use axum::extract::{Path, Query, State};
use axum::routing::get;
use chrono::{DateTime, TimeZone, Utc};
use futures::executor::block_on;
use serde::{de, Deserialize, Deserializer};
use serde_inline_default::serde_inline_default;
use serde_json::{json, Value};
use tracing::{debug, info, Level, trace, warn};
use tracing_subscriber::FmtSubscriber;

use domain_model::{Action, Candle, Currency, CurrencyPair, Deployment, DeploymentEvent, Exchange, InstrumentId, MarketType, OrderAction, OrderActionType, Timeframe};
use synapse::{SynapseListen, SynapseSend, Topic};

use crate::config::CONFIG;
use crate::service::ServiceFacade;
use crate::subscription_manager::SubscriptionManager;

mod service;
mod subscription_manager;
pub mod config;

const CANDLES_HISTORY: &str = "/api/v1/interactor/candles/history";

pub async fn run() {
    if CONFIG.eac.demo {
        info!("+ interactor running in DEMO mode...");
    } else {
        info!("+ interactor running in LIVE mode...");
    }
    listen_deployment_events();
    listen_actions();

    let service_facade = ServiceFacade::new(); // todo use only one instance of service facade
    let router = Router::new()
        .route(CANDLES_HISTORY, get(get_candles_history))
        .with_state(Arc::new(service_facade));

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), CONFIG.application.port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

fn listen_deployment_events() {
    let service_facade = ServiceFacade::new();
    let mut subscription_manager = SubscriptionManager::new(service_facade);
    synapse::reader(&CONFIG.application.name).on(Topic::Deployment, move |deployment: Deployment| {
        if deployment.simulation_id.is_none() {
            match deployment.event {
                DeploymentEvent::Created => {
                    debug!("Create deployment event with id: '{}', for strategy with id: '{}' and version: '{}'", 
                        deployment.id, deployment.strategy_name, deployment.strategy_version);
                    subscription_manager.subscribe(deployment.into())
                }

                DeploymentEvent::Deleted => {
                    debug!("Delete deployment event event with id: '{}', for strategy with id: '{}' and version: '{}'",
                        deployment.id, deployment.strategy_name, deployment.strategy_version);
                    subscription_manager.unsubscribe(deployment.id)
                }
            }
        }
    });
}

fn listen_actions() {
    let service_facade = ServiceFacade::new();
    synapse::reader(&CONFIG.application.name).on(Topic::Action, move |action: Action| {
        let simulation_id = match &action { Action::OrderAction(order_action) => order_action.simulation_id };
        if simulation_id.is_none() {
            debug!("Retrieved new action event");
            trace!("Action event: {action:?}");
            match action {
                Action::OrderAction(OrderAction { order: OrderActionType::CreateOrder(create_order), exchange, .. }) =>
                    service_facade.place_order(exchange, create_order),
                action => warn!("Temporary unsupported action: {action:?}")
            }
        }
    });
}

#[serde_inline_default]
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct QueryParams {
    exchange: Exchange,
    market_type: MarketType,
    target: Currency,
    source: Currency,
    timeframe: Timeframe,
    from_timestamp: Option<i64>,
    to_timestamp: Option<i64>,
    #[serde_inline_default(100)]
    limit: u8,
}

async fn get_candles_history(Query(query_params): Query<QueryParams>,
                             State(service_facade): State<Arc<ServiceFacade>>) -> Json<Vec<Candle>> {
    let instrument_id = InstrumentId {
        exchange: query_params.exchange,
        market_type: query_params.market_type,
        pair: CurrencyPair {
            target: query_params.target,
            source: query_params.source,
        },
    };
    let result = service_facade.candles_history(&instrument_id,
                                                query_params.timeframe,
                                                query_params.from_timestamp.map(|millis| Utc.timestamp_millis_opt(millis).unwrap()),
                                                query_params.to_timestamp.map(|millis| Utc.timestamp_millis_opt(millis).unwrap()),
                                                Some(query_params.limit)).await;
    Json(result)
}
