use std::net::{IpAddr, SocketAddr};
use axum::extract::{Query, State};
use axum::{Json, Router};
use axum::routing::{delete, get, post};
use chrono::{TimeZone, Utc};
use domain_model::{Action, Candle, CurrencyPair, InstrumentId, Subscription};
use interactor_api::InteractorApi;
use interactor_rest_api::endpoints::{DELETE_UNSUBSCRIBE, GET_CANDLES, GET_PRICE, POST_EXECUTE_ACTIONS, POST_SUBSCRIBE};
use interactor_rest_api::path_query::{CandlesQuery, PriceQuery};

pub async fn run(port: u16, interactor: impl InteractorApi) {
    let router = Router::new()
        .route(POST_SUBSCRIBE, post(subscribe))
        .route(DELETE_UNSUBSCRIBE, delete(unsubscribe))
        .route(POST_EXECUTE_ACTIONS, post(execute_action))
        .route(GET_CANDLES, get(get_candles))
        .route(GET_PRICE, get(get_price))
        .with_state(interactor);

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn subscribe(Json(subscription): Json<Subscription>, State(interactor): State<impl InteractorApi>) {
    interactor.subscribe(subscription);
}

async fn unsubscribe(Json(subscription): Json<Subscription>, State(interactor): State<impl InteractorApi>) {
    interactor.unsubscribe(subscription);
}

async fn execute_action(Json(actions): Json<Vec<Action>>, State(interactor): State<impl InteractorApi>) {
    interactor.execute_actions(actions);
}

async fn get_candles(Query(query_params): Query<CandlesQuery>,
                     State(interactor): State<impl InteractorApi>) -> Json<Vec<Candle>> {
    let instrument_id = InstrumentId {
        exchange: query_params.exchange,
        market_type: query_params.market_type,
        pair: CurrencyPair {
            target: query_params.target,
            source: query_params.source,
        },
    };
    let timeframe = query_params.timeframe;
    let from  = query_params.from_timestamp.map(|millis| Utc.timestamp_millis_opt(millis).unwrap());
    let to = query_params.to_timestamp.map(|millis| Utc.timestamp_millis_opt(millis).unwrap());
    let limit = Some(query_params.limit);

    let result = interactor.get_canldes(&instrument_id, timeframe, from, to, limit);
    Json(result)
}

async fn get_price(Query(query_params): Query<PriceQuery>,
                   State(interactor): State<impl InteractorApi>) -> Json<f64> {
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
    let price = interactor.get_price(&instrument_id, timestamp);
    Json(price)
}
