use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{Query, State};
use axum::routing::get;
use chrono::{TimeZone, Utc};
use tokio::sync::Mutex;
use tracing::{debug, info, trace, warn};

use domain_model::{Action, Candle, CurrencyPair, Deployment, DeploymentEvent, InstrumentId, OrderAction, OrderActionType};
use interactor_config::CONFIG;
use interactor_core::service::ServiceFacade;
use interactor_core::subscription_manager::SubscriptionManager;
use interactor_rest_api::endpoints::{GET_CANDLES_HISTORY, GET_PRICE};
use interactor_rest_api::path_query::{CandlesQuery, PriceQuery};
use synapse::{SynapseListen, Topic};

pub async fn run() {
    if CONFIG.eac.demo {
        info!("+ interactor running in DEMO mode...");
    } else {
        info!("+ interactor running in LIVE mode...");
    }
    let service_facade = Arc::new(Mutex::new(ServiceFacade::new()));

    listen_deployment_events(Arc::clone(&service_facade)).await;
    listen_actions(Arc::clone(&service_facade)).await;

    let router = Router::new()
        .route(GET_CANDLES_HISTORY, get(get_candles_history))
        .route(GET_PRICE, get(get_price))
        .with_state(service_facade);

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), CONFIG.application.port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn listen_deployment_events(service_facade: Arc<Mutex<ServiceFacade>>) {
    let subscription_manager = Arc::new(Mutex::new(SubscriptionManager::new(service_facade)));
    synapse::reader(&CONFIG.application.name).on(Topic::Deployment, move |deployment: Deployment| {
        let subscription_manager = Arc::clone(&subscription_manager);
        async move {
            if deployment.simulation_id.is_none() {
                match deployment.event {
                    DeploymentEvent::Created => {
                        debug!("Create deployment event with id: '{}', for strategy with id: '{}' and version: '{}'",
                        deployment.id, deployment.strategy_name, deployment.strategy_version);
                        subscription_manager.lock()
                            .await
                            .subscribe(deployment.into())
                            .await;
                    }

                    DeploymentEvent::Deleted => {
                        debug!("Delete deployment event event with id: '{}', for strategy with id: '{}' and version: '{}'",
                        deployment.id, deployment.strategy_name, deployment.strategy_version);
                        subscription_manager.lock()
                            .await
                            .unsubscribe(deployment.id)
                            .await;
                    }
                }
            }
        }
    }).await;
}

async fn listen_actions(service_facade: Arc<Mutex<ServiceFacade>>) {
    synapse::reader(&CONFIG.application.name).on(Topic::Action,
                                                 move |action: Action| {
                                                     let service_facade = Arc::clone(&service_facade);
                                                     async move {
                                                         let simulation_id = match &action { Action::OrderAction(order_action) => order_action.simulation_id };
                                                         if simulation_id.is_none() {
                                                             debug!("Retrieved new action event");
                                                             trace!("Action event: {action:?}");
                                                             match action {
                                                                 Action::OrderAction(OrderAction { order: OrderActionType::CreateOrder(create_order), exchange, .. }) =>
                                                                     service_facade
                                                                         .lock()
                                                                         .await
                                                                         .place_order(exchange, create_order)
                                                                         .await,
                                                                 action => warn!("Temporary unsupported action: {action:?}")
                                                             }
                                                         }
                                                     }
                                                 }).await;
}

async fn get_candles_history(Query(query_params): Query<CandlesQuery>,
                             State(service_facade): State<Arc<Mutex<ServiceFacade>>>) -> Json<Vec<Candle>> {
    let instrument_id = InstrumentId {
        exchange: query_params.exchange,
        market_type: query_params.market_type,
        pair: CurrencyPair {
            target: query_params.target,
            source: query_params.source,
        },
    };
    let result = service_facade
        .lock()
        .await
        .candles_history(&instrument_id,
                         query_params.timeframe,
                         query_params.from_timestamp.map(|millis| Utc.timestamp_millis_opt(millis).unwrap()),
                         query_params.to_timestamp.map(|millis| Utc.timestamp_millis_opt(millis).unwrap()),
                         Some(query_params.limit)).await;
    Json(result)
}

async fn get_price(Query(query_params): Query<PriceQuery>,
                   State(service_facade): State<Arc<Mutex<ServiceFacade>>>) -> Json<f64> {
    let timestamp = query_params.timestamp
        .map(|millis| Utc.timestamp_millis_opt(millis).unwrap())
        .unwrap_or(Utc::now());
    let instrument_id = InstrumentId {
        exchange: query_params.exchange,
        market_type: query_params.market_type,
        pair: CurrencyPair {
            target: query_params.target,
            source: query_params.source,
        },
    };
    let price = service_facade
        .lock()
        .await
        .price(&instrument_id,
               timestamp).await;
    Json(price)
}
