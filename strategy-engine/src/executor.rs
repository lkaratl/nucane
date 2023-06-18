use domain_model::{Action, Tick};
use strategy_api::{Strategy, StrategyApi};
use synapse::SynapseSend;
use crate::registry;

#[derive(Default)]
pub struct Executor {
    api: StrategyApi,
}

impl Executor {
    #[tokio::main]
    pub async fn handle(&self, tick: &Tick) {
        self.get_actions(tick).await
            .into_iter()
            .for_each(|action| produce_action(&action));
    }

    pub async fn get_actions(&self, tick: &Tick) -> Vec<Action> {
        let mut result = Vec::new();
        for deployment in registry::get_deployments()
            .await
            .lock()
            .await
            .iter_mut()
            .filter(|deployment| deployment.simulation_id == tick.simulation_id) {
            let strategy = &mut deployment.plugin.strategy;
            if is_subscribed(strategy, tick) {
                let mut actions = strategy.execute(tick, &self.api).await;
                actions.iter_mut().for_each(|action| match action {
                    Action::OrderAction(order_action) => order_action.simulation_id = deployment.simulation_id
                });
                result.append(&mut actions);
            }
        };
        result
    }
}

fn is_subscribed(strategy: &Box<dyn Strategy + Send>, tick: &Tick) -> bool {
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
    synapse::writer().send(action)
}
