use std::collections::HashMap;
use std::sync::Once;

use chrono::{TimeZone, Utc};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::SubscriberBuilder;

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
async fn test_grid_strategy() {
    init().await;
    let simulator_client = SimulatorClient::new(SIMULATOR_URL);

    let positions = vec![CreatePositionDto {
        exchange: Exchange::OKX,
        currency: Currency::USDT,
        side: Side::Buy,
        size: 5000.0,
    }];

    let params = HashMap::from([
        ("instrument".to_string(), "{\"exchange\": \"OKX\",\"market_type\": \"Spot\",\"pair\": {\"target\": \"FTM\",\"source\": \"USDT\"}}".to_string()),
        ("grid-start".to_string(), "0.15".to_string()),
        ("grid-end".to_string(), "0.5".to_string()),
        ("approximation".to_string(), "0.001".to_string()),
        ("order-size".to_string(), "100".to_string()),
        ("grid-density".to_string(), "0.01".to_string()),
        ("enable-state".to_string(), "false".to_string())
    ]);

    let simulation_report = simulator_client.run_simulation(Utc.timestamp_millis_opt(1686182400000).unwrap(),
                                                            Utc.timestamp_millis_opt(1686268800000).unwrap(),
                                                            positions,
                                                            "grid",
                                                            "1.0",
                                                            params)
        .await
        .unwrap();

    dbg!(&simulation_report);

    assert_eq!(simulation_report.ticks, 92689);
    assert_eq!(simulation_report.actions, 3);

    assert!(simulation_report.profit > 1.45);
    assert!(simulation_report.profit < 1.46);

    assert!(simulation_report.fees > 0.26);
    assert!(simulation_report.fees < 0.27);

    assert_eq!(simulation_report.assets.len(), 2);
    assert_eq!(simulation_report.assets.first().unwrap().exchange, Exchange::OKX);
    assert_eq!(simulation_report.assets.first().unwrap().currency, Currency::USDT);
    assert_eq!(simulation_report.assets.first().unwrap().start, 5000.0);
    assert!(simulation_report.assets.first().unwrap().end > 4902.65);
    assert!(simulation_report.assets.first().unwrap().end < 4902.66);
    assert!(simulation_report.assets.first().unwrap().diff < -97.33);
    assert!(simulation_report.assets.first().unwrap().diff > -97.35);
    assert_eq!(simulation_report.assets.first().unwrap().fees, 0.16);
    assert_eq!(simulation_report.assets.get(1).unwrap().exchange, Exchange::OKX);
    assert_eq!(simulation_report.assets.get(1).unwrap().currency, Currency::FTM);
    assert_eq!(simulation_report.assets.get(1).unwrap().start, 0.0);
    assert!(simulation_report.assets.get(1).unwrap().end > 333.99);
    assert!(simulation_report.assets.get(1).unwrap().end < 334.0);
    assert!(simulation_report.assets.get(1).unwrap().diff > 333.99);
    assert!(simulation_report.assets.get(1).unwrap().diff < 334.0);
    assert!(simulation_report.assets.get(1).unwrap().fees > 0.34);
    assert!(simulation_report.assets.get(1).unwrap().fees < 0.35);

    assert_eq!(simulation_report.active_orders.len(), 0);
}
