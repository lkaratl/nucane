use std::collections::HashMap;
use std::sync::Once;

use chrono::{TimeZone, Utc};
use tracing_subscriber::fmt::SubscriberBuilder;
use tracing_subscriber::EnvFilter;

use domain_model::{Currency, Exchange, Side};
use simulator_rest_api::dto::CreatePositionDto;
use simulator_rest_client::SimulatorClient;

static mut INITED: bool = false;
static INIT: Once = Once::new();

const SIMULATOR_URL: &'static str = "http://localhost:8084";

async fn init() {
    unsafe {
        if !INITED {
            init_logger();
            standalone::run();
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

#[tokio::test]
async fn test_e2e_simulation() {
    init().await;
    let simulator_client = SimulatorClient::new(SIMULATOR_URL);

    let positions = vec![CreatePositionDto {
        exchange: Exchange::OKX,
        currency: Currency::USDT,
        side: Side::Buy,
        size: 5000.0,
    }];

    let simulation_report = simulator_client.run_simulation(Utc.timestamp_millis_opt(1685879400000).unwrap(),
                                                            Utc.timestamp_millis_opt(1685880000000).unwrap(),
                                                            positions,
                                                            "simulation-e2e",
                                                            "1.0",
                                                            HashMap::from([(
                                                                "test-parameter".to_string(),
                                                                "test-value".to_string()
                                                            )]))
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
