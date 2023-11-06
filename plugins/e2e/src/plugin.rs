use std::sync::Arc;

use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use domain_model::{Action, CurrencyPair, Exchange, InstrumentId, MarketType, Tick, Timeframe};
use domain_model::drawing::{Color, Icon, LineStyle};
use plugin_api::{Line, PluginApi, PluginInternalApi, Point, utils};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub common_executed_once: bool,

    pub spot_executed_once: bool,
    pub spot_market_order_id: Option<String>,
    pub spot_market_order_id_sl_tp_id: Option<String>,
    // pub spot_market_order_id_market_sl_market_tp_id: Option<String>, // todo
    pub spot_limit_order_id: Option<String>,
    pub spot_limit_order_with_sl_tp_id: Option<String>,
    // pub spot_limit_order_id_market_sl_market_tp_id: Option<String>,

    // todo
    pub margin_executed_once: bool,
    pub margin_market_order_id: Option<String>,
    // pub margin_market_order_id_sl_tp_id: Option<String>,
    pub margin_market_order_id_market_sl_market_tp_id: Option<String>,
    // pub margin_limit_order_id: Option<String>,
    // pub margin_limit_order_with_sl_tp_id: Option<String>,
    // pub margin_limit_order_id_market_sl_market_tp_id: Option<String>,
}

pub struct E2EPlugin {
    pub state: State,
}

impl Default for E2EPlugin {
    fn default() -> Self {
        let plugin = Self {
            state: Default::default()
        };
        utils::init_logger(
            &format!("{}-{}", plugin.id().name, plugin.id().version),
            "INFO",
        );
        plugin
    }
}

impl E2EPlugin {
    pub fn get_state(&self) -> Value {
        serde_json::to_value(&self.state).unwrap()
    }

    pub fn set_state(&mut self, state: Value) {
        self.state = serde_json::from_value(state).unwrap()
    }

    pub async fn handle_tick(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Vec<Action> {
        if !self.state.common_executed_once {
            self.state.common_executed_once = true;
            info!("Execute common scenario");

            // self.check_indicators(tick, api.clone()).await;
            self.create_drawings(tick, api.clone()).await;
        }

        vec![
            // self.handle_spot_tick(tick, api.clone()).await,
            self.handle_margin_tick(tick, api.clone()).await,
        ]
            .into_iter()
            .flatten()
            .collect()
    }

    async fn check_indicators(&self, tick: &Tick, api: Arc<dyn PluginInternalApi>) {
        self.check_moving_avg(tick.instrument_id.pair, api).await;
    }

    async fn check_moving_avg(&self, pair: CurrencyPair, api: Arc<dyn PluginInternalApi>) {
        let instrument_id = InstrumentId {
            exchange: Exchange::OKX,
            market_type: MarketType::Spot,
            pair,
        };

        let moving_average = api
            .indicators()
            .sma(&instrument_id, Timeframe::OneH, 7)
            .await;
        info!("Moving AVG: {}", moving_average);
    }

    async fn create_drawings(&self, tick: &Tick, api: Arc<dyn PluginInternalApi>) {
        let point = Point::new(
            tick.instrument_id.clone(),
            "Check Point",
            Icon::Circle.into(),
            Color::Green.into(),
            "This point created only for test purposes"
                .to_string()
                .into(),
            (tick.timestamp + Duration::days(1), tick.price).into(),
        );
        api.drawings().save_point(point).await;

        let line = Line::new(
            tick.instrument_id.clone(),
            "Check Line",
            LineStyle::Dashed.into(),
            Color::Green.into(),
            (tick.timestamp, tick.price).into(),
            (tick.timestamp + Duration::days(1), tick.price * 1.1).into(),
        );
        api.drawings().save_line(line).await;
    }
}
