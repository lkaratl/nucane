use std::net::{IpAddr, SocketAddr};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use chrono::{Duration, TimeZone, Utc};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr};
use tracing::{error, info, warn};

use domain_model::{AuditEvent, Candle, CurrencyPair, Deployment, DeploymentEvent, InstrumentId, Order, Position, Simulation, Timeframe};
use interactor_rest_client::InteractorClient;
use storage_config::CONFIG;
use storage_core::audit::AuditService;
use storage_core::candle::CandleService;
use storage_core::candle_sync::{CandleSyncService};
use storage_core::order::OrderService;
use storage_core::position::PositionService;
use storage_migration::{Migrator, MigratorTrait};
use storage_rest_api::dto::SyncReportDto;
use storage_rest_api::endpoints::{GET_AUDIT, POST_CANDLES_SYNC, GET_CANDLES, GET_ORDERS, GET_POSITIONS};
use storage_rest_api::path_query::{AuditQuery, CandlesQuery, CandleSyncQuery, OrdersQuery, PositionsQuery};
use synapse::{SynapseListen, Topic};

pub async fn run() {
    info!("+ storage running...");
    let db = init_db().await;

    let order_service = Arc::new(OrderService::new(Arc::clone(&db)));
    let position_service = Arc::new(PositionService::new(Arc::clone(&db)));
    let candle_service = Arc::new(CandleService::new(Arc::clone(&db)));
    let audit_service = Arc::new(AuditService::new(Arc::clone(&db)));
    listen_entity_events(Arc::clone(&order_service),
                         Arc::clone(&position_service),
                         Arc::clone(&candle_service),
                         Arc::clone(&audit_service)).await;

    let interactor_client = Arc::new(InteractorClient::new(&CONFIG.interactor.url)); // todo unhardcode
    let candle_sync_service = Arc::new(CandleSyncService::new(Arc::clone(&candle_service), Arc::clone(&interactor_client)));
    listen_deployment_events(Arc::clone(&candle_sync_service), Arc::clone(&audit_service)).await;
    listen_auditable_events(Arc::clone(&audit_service)).await;

    let router = Router::new()
        .route(GET_CANDLES, get(get_candles))
        .with_state(Arc::clone(&candle_service))
        .route(GET_ORDERS, get(get_orders))
        .with_state(Arc::clone(&order_service))
        .route(GET_POSITIONS, get(get_positions))
        .with_state(Arc::clone(&position_service))
        .route(GET_AUDIT, get(get_audit))
        .with_state(Arc::clone(&audit_service))
        .route(POST_CANDLES_SYNC, post(candles_sync))
        .with_state(Arc::clone(&candle_sync_service));

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), CONFIG.application.port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn init_db() -> Arc<DatabaseConnection> {
    let db = Database::connect(format!("{}/postgres", &CONFIG.database.url)).await
        .expect("storage: Error during connecting to database");
    let _ = db.execute_unprepared(&format!("CREATE DATABASE {};", &CONFIG.application.name))
        .await.map_err(|err|
        match err {
            DbErr::Exec(err) => warn!("{}", err),
            err => error!("{}", err)
        });

    let db = Arc::new(Database::connect(format!("{}/{}", &CONFIG.database.url, &CONFIG.application.name)).await
        .expect("storage: Error during connecting to 'storage' database"));
    Migrator::up(db.deref(), None).await.expect("storage: Failed apply db migrations");
    db
}

async fn listen_entity_events(order_service: Arc<OrderService<DatabaseConnection>>,
                              position_service: Arc<PositionService<DatabaseConnection>>,
                              candle_service: Arc<CandleService<DatabaseConnection>>,
                              audit_service: Arc<AuditService<DatabaseConnection>>) {
    synapse::reader(&CONFIG.broker.url, &CONFIG.application.name).on(Topic::Order,
                                                                     {
                                                                         let audit_service = Arc::clone(&audit_service);
                                                                         move |order: Order| {
                                                                             let order_service = Arc::clone(&order_service);
                                                                             let audit_service = Arc::clone(&audit_service);
                                                                             async move {
                                                                                 order_service.save(order.clone()).await;
                                                                                 audit_service.log_order(order).await;
                                                                             }
                                                                         }
                                                                     }).await;
    synapse::reader(&CONFIG.broker.url, &CONFIG.application.name).on(Topic::Position, move |position: Position| {
        let position_service = Arc::clone(&position_service);
        let audit_service = Arc::clone(&audit_service);
        async move {
            position_service.save(position.clone()).await;
            audit_service.log_position(position).await;
        }
    }).await;
    synapse::reader(&CONFIG.broker.url, &CONFIG.application.name).on(Topic::Candle, move |candle: Candle| {
        let candle_service = Arc::clone(&candle_service);
        async move {
            candle_service.save(candle).await;
        }
    }).await;
}

async fn listen_deployment_events(candle_sync_service: Arc<CandleSyncService>, audit_service: Arc<AuditService<DatabaseConnection>>) {
    synapse::reader(&CONFIG.broker.url, &CONFIG.application.name).on(Topic::Deployment, move |deployment: Deployment| {
        let audit_service = Arc::clone(&audit_service);
        let candle_sync_service = Arc::clone(&candle_sync_service);
        async move {
            if deployment.simulation_id.is_none() {
                match deployment.event {
                    DeploymentEvent::Created => {
                        let from = Utc::now() - Duration::days(30);
                        let timeframes = [Timeframe::FifteenM, Timeframe::ThirtyM, Timeframe::OneH, Timeframe::FourH, Timeframe::OneD];
                        for instrument_id in &deployment.subscriptions {
                            let _ = candle_sync_service.sync(instrument_id, &timeframes, from, None).await;
                        }
                    }
                    DeploymentEvent::Deleted => {}
                }
            }
            audit_service.log_deployment(deployment).await;
        }
    }).await;
}

async fn listen_auditable_events(audit_service: Arc<AuditService<DatabaseConnection>>) {
    synapse::reader(&CONFIG.broker.url, &CONFIG.application.name).on(Topic::Simulation, move |simulation: Simulation| {
        let audit_service = Arc::clone(&audit_service);
        async move {
            audit_service.log_simulation(simulation).await;
        }
    }).await;
}

async fn get_candles(Query(query_params): Query<CandlesQuery>,
                     State(candle_service): State<Arc<CandleService<DatabaseConnection>>>) -> Json<Vec<Candle>> {
    let instrument_id = InstrumentId {
        exchange: query_params.exchange,
        market_type: query_params.market_type,
        pair: CurrencyPair {
            target: query_params.target,
            source: query_params.source,
        },
    };
    let result = candle_service.get(&instrument_id,
                                    query_params.timeframe,
                                    query_params.from_timestamp.map(|millis| Utc.timestamp_millis_opt(millis).unwrap()),
                                    query_params.to_timestamp.map(|millis| Utc.timestamp_millis_opt(millis).unwrap()),
                                    query_params.limit).await;
    Json(result)
}

async fn get_orders(Query(query_params): Query<OrdersQuery>,
                    State(order_service): State<Arc<OrderService<DatabaseConnection>>>) -> Json<Vec<Order>> {
    let result = order_service.get(
        query_params.id,
        query_params.exchange,
        query_params.market_type,
        query_params.target,
        query_params.source,
        query_params.status,
        query_params.side,
        query_params.order_type,
    ).await;
    Json(result)
}

async fn get_positions(Query(query_params): Query<PositionsQuery>,
                       State(position_service): State<Arc<PositionService<DatabaseConnection>>>) -> Json<Vec<Position>> {
    let result = position_service.get(query_params.exchange, query_params.currency, query_params.side).await;
    Json(result)
}

async fn get_audit(Query(query_params): Query<AuditQuery>,
                   State(audit_service): State<Arc<AuditService<DatabaseConnection>>>) -> Json<Vec<AuditEvent>> {
    let result = audit_service.get(
        query_params.from_timestamp.map(|millis| Utc.timestamp_millis_opt(millis).unwrap()),
        query_params.tags
            .map(|tags_string|
                tags_string.split(',')
                    .map(|str| str.to_string())
                    .collect())
            .unwrap_or(Vec::new()),
        query_params.limit).await;
    Json(result)
}

async fn candles_sync(Query(query_params): Query<CandleSyncQuery>,
                      State(candle_sync_service): State<Arc<CandleSyncService>>,
                      Json(request): Json<InstrumentId>) -> Result<Json<Vec<SyncReportDto>>, StatusCode> {
    let timeframes: Vec<_> = query_params.timeframes.split(',')
        .map(Timeframe::from_str)
        .map(|timeframe| timeframe.unwrap())
        .collect();
    let from = Utc.timestamp_millis_opt(query_params.from).unwrap();
    let to = query_params.to.map(|millis| Utc.timestamp_millis_opt(millis).unwrap());
    let result = candle_sync_service.sync(&request, &timeframes, from, to).await
        .map_err(|err| {
            error!("Error syncing candles: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_iter()
        .map(SyncReportDto::from)
        .collect();
    Ok(Json(result))
}
