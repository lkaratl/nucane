use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use chrono::{TimeZone, Utc};
use tracing::error;

use domain_model::{Candle, CurrencyPair, InstrumentId, LP, Order, Position, Timeframe};
use domain_model::drawing::{Line, Point};
use storage_core_api::{StorageApi, SyncReport};
use storage_rest_api::endpoints::{GET_CANDLES, GET_LINES, GET_ORDERS, GET_POINTS, GET_POSITIONS, POST_CANDLES, POST_LINE, POST_LP, POST_ORDERS, POST_POINT, POST_POSITIONS, POST_SYNC};
use storage_rest_api::path_queries::{
    CandlesQuery, CandleSyncQuery, DrawingQuery, OrdersQuery, PositionsQuery,
};

pub async fn run(port: u16, storage: impl StorageApi) {
    let storage = Arc::new(storage);
    let router = Router::new()
        .route(GET_CANDLES, get(get_candles))
        .route(POST_CANDLES, post(create_candle))
        .route(GET_ORDERS, get(get_orders))
        .route(POST_ORDERS, post(create_order))
        .route(POST_LP, post(add_lp))
        .route(GET_POSITIONS, get(get_positions))
        .route(POST_POSITIONS, post(create_position))
        .route(POST_SYNC, post(sync))
        .route(GET_POINTS, get(get_points))
        .route(POST_POINT, post(create_point))
        .route(GET_LINES, get(get_lines))
        .route(POST_LINE, post(create_line))
        .with_state(storage);

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn get_candles(
    Query(query_params): Query<CandlesQuery>,
    State(storage): State<Arc<dyn StorageApi>>,
) -> Json<Vec<Candle>> {
    let instrument_id = InstrumentId {
        exchange: query_params.exchange,
        market_type: query_params.market_type,
        pair: CurrencyPair {
            target: query_params.target,
            source: query_params.source,
        },
    };
    let timeframe = query_params.timeframe;
    let from = query_params
        .from
        .map(|millis| Utc.timestamp_millis_opt(millis).unwrap());
    let to = query_params
        .to
        .map(|millis| Utc.timestamp_millis_opt(millis).unwrap());
    let limit = query_params.limit;

    let result = storage
        .get_candles(&instrument_id, timeframe, from, to, limit)
        .await
        .unwrap();
    Json(result)
}

async fn create_candle(State(storage): State<Arc<dyn StorageApi>>, Json(candle): Json<Candle>) {
    storage.save_candle(candle).await.unwrap();
}

async fn get_orders(
    Query(query_params): Query<OrdersQuery>,
    State(storage): State<Arc<dyn StorageApi>>,
) -> Json<Vec<Order>> {
    let result = storage
        .get_orders(
            query_params.id,
            query_params.simulation_id,
            query_params.exchange,
            query_params.market_type,
            query_params.target,
            query_params.source,
            query_params.status,
            query_params.side,
            query_params.order_type,
        )
        .await
        .unwrap();
    Json(result)
}

async fn create_order(State(storage): State<Arc<dyn StorageApi>>, Json(order): Json<Order>) {
    storage.save_order(order).await.unwrap();
}

async fn add_lp(State(storage): State<Arc<dyn StorageApi>>, Json(lp): Json<LP>) {
    storage.save_lp(lp).await.unwrap();
}

async fn get_positions(
    Query(query_params): Query<PositionsQuery>,
    State(storage): State<Arc<dyn StorageApi>>,
) -> Json<Vec<Position>> {
    let result = storage
        .get_positions(
            query_params.exchange,
            query_params.currency,
            query_params.side,
        )
        .await
        .unwrap();
    Json(result)
}

async fn create_position(
    State(storage): State<Arc<dyn StorageApi>>,
    Json(position): Json<Position>,
) {
    storage.save_position(position).await.unwrap();
}

async fn sync(
    Query(query_params): Query<CandleSyncQuery>,
    State(storage): State<Arc<dyn StorageApi>>,
    Json(request): Json<InstrumentId>,
) -> Result<Json<Vec<SyncReport>>, StatusCode> {
    let timeframes: Vec<_> = query_params
        .timeframes
        .split(',')
        .map(Timeframe::from_str)
        .map(|timeframe| timeframe.unwrap())
        .collect();
    let from = Utc.timestamp_millis_opt(query_params.from).unwrap();
    let to = query_params
        .to
        .map(|millis| Utc.timestamp_millis_opt(millis).unwrap());
    let result = storage
        .sync(&request, &timeframes, from, to)
        .await
        .map_err(|err| {
            error!("Error syncing candles: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(result))
}

async fn get_points(
    Query(query_params): Query<DrawingQuery>,
    State(storage): State<Arc<dyn StorageApi>>,
) -> Json<Vec<Point>> {
    let instrument_id = InstrumentId {
        exchange: query_params.exchange,
        market_type: query_params.market_type,
        pair: CurrencyPair {
            target: query_params.target,
            source: query_params.source,
        },
    };
    let result = storage
        .get_points(query_params.deployment_id, &instrument_id)
        .await
        .unwrap();
    Json(result)
}

async fn create_point(State(storage): State<Arc<dyn StorageApi>>, Json(point): Json<Point>) {
    storage.save_point(point).await.unwrap();
}

async fn get_lines(
    Query(query_params): Query<DrawingQuery>,
    State(storage): State<Arc<dyn StorageApi>>,
) -> Json<Vec<Line>> {
    let instrument_id = InstrumentId {
        exchange: query_params.exchange,
        market_type: query_params.market_type,
        pair: CurrencyPair {
            target: query_params.target,
            source: query_params.source,
        },
    };
    let result = storage
        .get_lines(query_params.deployment_id, &instrument_id)
        .await
        .unwrap();
    Json(result)
}

async fn create_line(State(storage): State<Arc<dyn StorageApi>>, Json(line): Json<Line>) {
    storage.save_line(line).await.unwrap();
}
