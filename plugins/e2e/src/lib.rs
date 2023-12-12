use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use domain_model::{Action, Currency, CurrencyPair, Exchange, Indicator, InstrumentId, MarketType, PluginId, Tick};
use plugin_api::{PluginApi, PluginInternalApi};

use crate::plugin::E2EPlugin;

mod plugin;
mod margin;
mod spot;

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn load() -> Box<dyn PluginApi> {
    Box::<E2EPlugin>::default()
}

const PLUGIN_NAME: &str = "e2e";
const PLUGIN_VERSION: i64 = 2;

const PARAMETER_NAME: &str = "test-parameter";

#[async_trait]
impl PluginApi for E2EPlugin {
    fn id(&self) -> PluginId {
        PluginId::new(PLUGIN_NAME, PLUGIN_VERSION)
    }

    fn configure(&mut self, config: &HashMap<String, String>) {
        config.get(PARAMETER_NAME).unwrap().to_string();
    }

    // todo make configurable
    fn instruments(&self) -> Vec<InstrumentId> {
        vec![InstrumentId {
            exchange: Exchange::BYBIT,
            market_type: MarketType::Spot,
            pair: CurrencyPair {
                target: Currency::BTC,
                source: Currency::USDT,
            },
        }]
    }

    fn indicators(&self) -> Vec<Indicator> {
        vec![
            // Indicator::SMA(7),
            // Indicator::BB(20, 2.),
            // Indicator::PSAR
        ]
    }

    async fn get_state(&self) -> Option<Value> {
        Some(self.get_state())
    }

    async fn set_state(&mut self, state: Value) {
        self.set_state(state);
    }

    async fn on_tick(&mut self, tick: &Tick, api: Arc<dyn PluginInternalApi>) -> Vec<Action> {
        self.handle_tick(tick, api).await
    }
}
