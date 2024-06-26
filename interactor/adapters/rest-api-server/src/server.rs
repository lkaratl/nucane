use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{Query, State};
use axum::routing::{delete, get, post};
use chrono::{TimeZone, Utc};

use domain_model::{Action, Candle, CurrencyPair, InstrumentId, Order, Subscription, Subscriptions};
use interactor_core_api::InteractorApi;
use interactor_rest_api::endpoints::{DELETE_UNSUBSCRIBE, GET_CANDLES, GET_ORDER, GET_PRICE, GET_SUBSCRIPTIONS, GET_TOTAL_BALANCE, POST_EXECUTE_ACTIONS, POST_SUBSCRIBE};
use interactor_rest_api::path_queries::{CandlesQuery, OrderQuery, PriceQuery, TotalBalanceQuery};

pub async fn run(port: u16, interactor: impl InteractorApi) {
    let interactor = Arc::new(interactor);
    let router = Router::new()
        .route(GET_SUBSCRIPTIONS, get(subscriptions))
        .route(POST_SUBSCRIBE, post(subscribe))
        .route(DELETE_UNSUBSCRIBE, delete(unsubscribe))
        .route(POST_EXECUTE_ACTIONS, post(execute_action))
        .route(GET_CANDLES, get(get_candles))
        .route(GET_PRICE, get(get_price))
        .route(GET_ORDER, get(get_order))
        .route(GET_TOTAL_BALANCE, get(get_total_balance))
        .with_state(interactor);

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn subscriptions(
    State(interactor): State<Arc<dyn InteractorApi>>,
) -> Json<Vec<Subscriptions>> {
    let subscriptions = interactor.subscriptions().await.unwrap();
    Json(subscriptions)
}

async fn subscribe(
    State(interactor): State<Arc<dyn InteractorApi>>,
    Json(subscription): Json<Subscription>,
) {
    interactor.subscribe(subscription).await.unwrap();
}

async fn unsubscribe(
    State(interactor): State<Arc<dyn InteractorApi>>,
    Json(subscription): Json<Subscription>,
) {
    interactor.unsubscribe(subscription).await.unwrap();
}

async fn execute_action(
    State(interactor): State<Arc<dyn InteractorApi>>,
    Json(actions): Json<Vec<Action>>,
) {
    interactor.execute_actions(actions).await.unwrap();
}

async fn get_candles(
    Query(query_params): Query<CandlesQuery>,
    State(interactor): State<Arc<dyn InteractorApi>>,
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
    let limit = Some(query_params.limit);

    let result = interactor
        .get_candles(&instrument_id, timeframe, from, to, limit)
        .await
        .unwrap();
    Json(result)
}

async fn get_price(
    Query(query_params): Query<PriceQuery>,
    State(interactor): State<Arc<dyn InteractorApi>>,
) -> Json<f64> {
    let timestamp = query_params
        .timestamp
        .map(|millis| Utc.timestamp_millis_opt(millis).unwrap());
    let instrument_id = InstrumentId {
        exchange: query_params.exchange,
        market_type: query_params.market_type,
        pair: CurrencyPair {
            target: query_params.target,
            source: query_params.source,
        },
    };
    let price = interactor
        .get_price(&instrument_id, timestamp)
        .await
        .unwrap();
    Json(price)
}

async fn get_order(
    Query(query_params): Query<OrderQuery>,
    State(interactor): State<Arc<dyn InteractorApi>>,
) -> Json<Option<Order>> {
    let order_id = &query_params.order_id;
    let exchange = query_params.exchange;
    let order = interactor
        .get_order(exchange, order_id)
        .await
        .unwrap();
    Json(order)
}

async fn get_total_balance(
    Query(query_params): Query<TotalBalanceQuery>,
    State(interactor): State<Arc<dyn InteractorApi>>,
) -> Json<f64> {
    let exchange = query_params.exchange;
    let total_balance = interactor
        .get_total_balance(exchange)
        .await
        .unwrap();
    Json(total_balance)
}
