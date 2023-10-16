use std::sync::Arc;

use uuid::Uuid;

use domain_model::PluginId;
use plugin_api::{
    ActionsInternalApi, DrawingsInternalApi, IndicatorsInternalApi, OrdersInternalApi,
    PluginInternalApi, PositionsInternalApi,
};
use storage_core_api::StorageApi;

use crate::actions::DefaultActionInternals;
use crate::drawings::DefaultDrawingInternals;
use crate::indicators::DefaultIndicatorInternals;
use crate::orders::DefaultOrderInternals;
use crate::positions::DefaultPositionInternals;

pub struct DefaultPluginInternals<S: StorageApi> {
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
    ) -> Self {
        Self {
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
