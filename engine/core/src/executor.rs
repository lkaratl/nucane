use tracing::debug;
use domain_model::{Action, Tick};
use engine_config::CONFIG;
use strategy_api::{Strategy, StrategyApi};
use synapse::SynapseSend;
use crate::registry;

pub struct Executor {
    api: StrategyApi,
}

impl Executor {
    pub fn new(storage_url: &str) -> Self {
        Self {
            api: StrategyApi::new(storage_url)
        }
    }

    pub async fn handle(&self, tick: &Tick) {
        self.get_actions(tick).await
            .into_iter()
            .for_each(|action| produce_action(&action));
    }

    pub async fn get_actions(&self, tick: &Tick) -> Vec<Action> {
        let mut result = Vec::new();
        for deployment in registry::get_deployments()
            .await
            .iter_mut() {
            let mut deployment = deployment.lock().await;
            let is_simulation = deployment.simulation_id == tick.simulation_id;
            let strategy = &mut deployment.plugin.strategy;
            if is_subscribed(strategy.as_ref(), tick) && is_simulation {
                debug!("Processing tick: '{} {}-{}={}' for strategy: '{}:{}'",
                    tick.instrument_id.exchange, tick.instrument_id.pair.target, tick.instrument_id.pair.source, tick.price,
                    strategy.name(), strategy.version());
                let mut actions = strategy.on_tick_sync(tick, &self.api);
                actions.iter_mut().for_each(|action| match action {
                    Action::OrderAction(order_action) => order_action.simulation_id = deployment.simulation_id
                });
                result.append(&mut actions);
            }
        };
        result
    }
}

fn is_subscribed(strategy: &(dyn Strategy + Send), tick: &Tick) -> bool {
    let instrument_id = &tick.instrument_id;
    strategy.subscriptions()
        .iter()
        .any(|subscription|
            subscription.exchange.eq(&instrument_id.exchange) &&
                subscription.market_type.eq(&instrument_id.market_type) &&
                subscription.pair.target.eq(&instrument_id.pair.target) &&
                subscription.pair.source.eq(&instrument_id.pair.source)
        )
}

fn produce_action(action: &Action) {
    synapse::writer(&CONFIG.broker.url,).send(action)
}
