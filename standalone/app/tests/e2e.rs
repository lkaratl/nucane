use std::collections::HashMap;
use std::sync::Once;

use chrono::{TimeZone, Utc};
use tracing::debug;
use tracing_subscriber::fmt::SubscriberBuilder;
use tracing_subscriber::EnvFilter;

use domain_model::{CreatePosition, CreateSimulation, CreateSimulationDeployment, Currency, CurrencyPair, Exchange, InstrumentId, MarketType, PluginId, Side, Timeframe};
use simulator_core_api::SimulatorApi;
use simulator_rest_client::SimulatorRestClient;
use storage_core_api::StorageApi;
use storage_rest_client::StorageRestClient;

static mut INITED: bool = false;
static INIT: Once = Once::new();

const STORAGE_URL: &str = "http://localhost:8082";
const SIMULATOR_URL: &str = "http://localhost:8084";

fn init() {
    unsafe {
        if !INITED {
            init_logger();
            standalone_app::run();
        }
        INIT.call_once(|| INITED = true);
    }
}

fn init_logger() {
    let subscriber = SubscriberBuilder::default()
        // todo move to config or init on standalone side
        .with_env_filter(EnvFilter::new("INFO,engine=DEBUG,storage=DEBUG,simulator=DEBUG,interactor=DEBUG"))
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");
}

#[ignore = "run db and broker on ci"]
// clean storage before this test
#[tokio::test]
async fn test_e2e_candles_sync() {
    init();
    let storage_client = StorageRestClient::new(STORAGE_URL);

    let instrument_id = InstrumentId {
        exchange: Exchange::OKX,
        market_type: MarketType::Spot,
        pair: CurrencyPair {
            target: Currency::BTC,
            source: Currency::USDT,
        },
    };
    let timeframes = [Timeframe::FiveM, Timeframe::OneH, Timeframe::OneD];
    let from = Utc.timestamp_millis_opt(1682899200000).unwrap();
    let to = Utc.timestamp_millis_opt(1685577600000).unwrap();

    let reports = storage_client.sync(&instrument_id, &timeframes, from, Some(to)).await.unwrap();
    debug!("{reports:?}");
    let mut reports_iter = reports.iter();

    let report = reports_iter.next().unwrap();
    assert_eq!(report.timeframe, Timeframe::FiveM);
    assert_eq!(report.total, 8928);
    assert_eq!(report.exists, 0);
    assert_eq!(report.synced, 8928);

    let report = reports_iter.next().unwrap();
    assert_eq!(report.timeframe, Timeframe::OneH);
    assert_eq!(report.total, 744);
    assert_eq!(report.exists, 0);
    assert_eq!(report.synced, 744);

    let report = reports_iter.next().unwrap();
    assert_eq!(report.timeframe, Timeframe::OneD);
    assert_eq!(report.total, 31);
    assert_eq!(report.exists, 0);
    assert_eq!(report.synced, 31);

    let reports = storage_client.sync(&instrument_id, &timeframes, from, Some(to)).await.unwrap();
    debug!("{reports:?}");
    let mut reports_iter = reports.iter();

    let report = reports_iter.next().unwrap();
    assert_eq!(report.timeframe, Timeframe::FiveM);
    assert_eq!(report.total, 8928);
    assert_eq!(report.exists, 8928);
    assert_eq!(report.synced, 0);

    let report = reports_iter.next().unwrap();
    assert_eq!(report.timeframe, Timeframe::OneH);
    assert_eq!(report.total, 744);
    assert_eq!(report.exists, 744);
    assert_eq!(report.synced, 0);

    let report = reports_iter.next().unwrap();
    assert_eq!(report.timeframe, Timeframe::OneD);
    assert_eq!(report.total, 31);
    assert_eq!(report.exists, 31);
    assert_eq!(report.synced, 0);
}

#[ignore = "run db and broker on ci"]
#[tokio::test]
async fn test_e2e_simulation() {
    init();
    let simulator_client = SimulatorRestClient::new(SIMULATOR_URL);

    let positions = vec![CreatePosition {
        exchange: Exchange::OKX,
        currency: Currency::USDT,
        side: Side::Buy,
        size: 5000.0,
    }];
    let plugin_id = PluginId::new("simulation-e2e", 1);
    let strategy = CreateSimulationDeployment {
        simulation_id: None,
        timeframe: Timeframe::FiveM,
        plugin_id,
        params: HashMap::from([(
            "test-parameter".to_string(),
            "test-value".to_string()
        )]),
    };

    let new_simulation = CreateSimulation {
        positions,
        start: 1685879400000,
        end: 1685880000000,
        strategies: vec![strategy],
    };

    let simulation_report = simulator_client.run_simulation(new_simulation)
        .await
        .unwrap();

    dbg!(&simulation_report);

    assert_eq!(simulation_report.ticks, 44);
    assert_eq!(simulation_report.actions, 1);

    assert!(simulation_report.profit < -0.36);
    assert!(simulation_report.profit > -0.37);

    assert_eq!(simulation_report.fees, 0.2);

    assert_eq!(simulation_report.assets.len(), 2);
    assert_eq!(simulation_report.assets.first().unwrap().exchange, Exchange::OKX);
    assert_eq!(simulation_report.assets.first().unwrap().currency, Currency::USDT);
    assert_eq!(simulation_report.assets.first().unwrap().start, 5000.0);
    assert_eq!(simulation_report.assets.first().unwrap().end, 3999.8);
    assert_eq!(simulation_report.assets.first().unwrap().diff, -1000.1999999999998);
    assert_eq!(simulation_report.assets.first().unwrap().fees, 0.2);
    assert_eq!(simulation_report.assets.get(1).unwrap().exchange, Exchange::OKX);
    assert_eq!(simulation_report.assets.get(1).unwrap().currency, Currency::BTC);
    assert_eq!(simulation_report.assets.get(1).unwrap().start, 0.0);
    assert!(simulation_report.assets.get(1).unwrap().end > 0.036);
    assert!(simulation_report.assets.get(1).unwrap().end < 0.037);
    assert!(simulation_report.assets.get(1).unwrap().diff > 0.036);
    assert!(simulation_report.assets.get(1).unwrap().diff < 0.037);
    assert_eq!(simulation_report.assets.get(1).unwrap().fees, 0.0);

    assert_eq!(simulation_report.active_orders.len(), 0);
}
