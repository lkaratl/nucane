use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{Query, State};
use axum::routing::get;
use chrono::{TimeZone, Utc};
use tracing::{debug, info, trace, warn};

use domain_model::{Action, Candle, CurrencyPair, Deployment, DeploymentEvent, InstrumentId, OrderAction, OrderActionType};
use interactor_config::CONFIG;
use interactor_core::service::ServiceFacade;
use interactor_core::subscription_manager::SubscriptionManager;
use interactor_rest_api::endpoints::GET_CANDLES_HISTORY;
use interactor_rest_api::path_query::CandlesHistoryQuery;
use synapse::{SynapseListen, Topic};

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
        .route(GET_CANDLES_HISTORY, get(get_candles_history))
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

async fn get_candles_history(Query(query_params): Query<CandlesHistoryQuery>,
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
