use std::sync::Arc;

use uuid::Uuid;

use domain_model::PluginId;
use engine_fs_plugin_state::FsStateManager;
use plugin_api::{
    ActionsInternalApi, DrawingsInternalApi, IndicatorsInternalApi, OrdersInternalApi,
    PluginInternalApi, PositionsInternalApi, StateInternalApi,
};
use storage_core_api::StorageApi;

use crate::actions::DefaultActionInternals;
use crate::drawings::DefaultDrawingInternals;
use crate::indicators::DefaultIndicatorInternals;
use crate::orders::DefaultOrderInternals;
use crate::positions::DefaultPositionInternals;
use crate::state::DefaultStateInternals;

pub struct DefaultPluginInternals<S: StorageApi> {
    state: Arc<DefaultStateInternals>,
    actions: Arc<DefaultActionInternals>,
    orders: Arc<DefaultOrderInternals<S>>,
    positions: Arc<DefaultPositionInternals<S>>,
    indicators: Arc<DefaultIndicatorInternals<S>>,
    drawings: Arc<DefaultDrawingInternals<S>>,
}

impl<S: StorageApi> DefaultPluginInternals<S> {
    pub fn new(
        deployment_id: Uuid,
        plugin_id: PluginId,
        simulation_id: Option<Uuid>,
        storage_client: Arc<S>,
        state_manager: Arc<FsStateManager>,
    ) -> Self {
        Self {
            state: Arc::new(DefaultStateInternals::new(deployment_id, state_manager)),
            actions: Arc::new(DefaultActionInternals::new(simulation_id, plugin_id)),
            orders: Arc::new(DefaultOrderInternals::new(Arc::clone(&storage_client))),
            positions: Arc::new(DefaultPositionInternals::new(Arc::clone(&storage_client))),
            indicators: Arc::new(DefaultIndicatorInternals::new(Arc::clone(&storage_client))),
            drawings: Arc::new(DefaultDrawingInternals::new(
                deployment_id,
                Arc::clone(&storage_client),
            )),
        }
    }
}

impl<S: StorageApi> PluginInternalApi for DefaultPluginInternals<S> {
    fn state(&self) -> Arc<dyn StateInternalApi> {
        self.state.clone()
    }

    fn actions(&self) -> Arc<dyn ActionsInternalApi> {
        self.actions.clone()
    }

    fn orders(&self) -> Arc<dyn OrdersInternalApi> {
        self.orders.clone()
    }

    fn positions(&self) -> Arc<dyn PositionsInternalApi> {
        self.positions.clone()
    }

    fn indicators(&self) -> Arc<dyn IndicatorsInternalApi> {
        self.indicators.clone()
    }

    fn drawings(&self) -> Arc<dyn DrawingsInternalApi> {
        self.drawings.clone()
    }
}
