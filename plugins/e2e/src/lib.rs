use std::collections::HashMap;

use async_trait::async_trait;

use domain_model::{Action, Currency, CurrencyPair, Exchange, InstrumentId, MarketType, Tick};
use strategy_api::{Strategy, StrategyApi};

use crate::strategy::E2EStrategy;

mod strategy;

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn load() -> Box<dyn Strategy> {
    Box::<E2EStrategy>::default()
}

const STRATEGY_NAME: &str = "e2e";
const STRATEGY_VERSION: i64 = 1;

const PARAMETER_NAME: &str = "test-parameter";

#[async_trait]
impl Strategy for E2EStrategy {
    fn tune(&mut self, config: &HashMap<String, String>) {
        config.get(PARAMETER_NAME).unwrap().to_string();
    }

    fn name(&self) -> String {
        STRATEGY_NAME.to_string()
    }

    fn version(&self) -> i64 {
        STRATEGY_VERSION
    }

    fn subscriptions(&self) -> Vec<InstrumentId> {
        vec![InstrumentId {
            exchange: Exchange::OKX,
            market_type: MarketType::Spot,
            pair: CurrencyPair {
                target: Currency::BTC,
                source: Currency::USDT,
            },
        }]
    }

    async fn on_tick(&mut self, tick: &Tick, api: &StrategyApi) -> Vec<Action> {
        self.handle_tick(tick, api).await
    }
}
