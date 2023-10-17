use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{ActiveValue, ColumnTrait, Condition, ConnectionTrait, EntityTrait, sea_query};
use sea_orm::QueryFilter;
use serde_json::json;
use uuid::Uuid;

use simulator_persistence_api::SimulationReportRepository;

use crate::entities::*;
use crate::entities::prelude::SimulationReport;

pub struct SimulationReportPostgresRepository<T: ConnectionTrait> {
    db: Arc<T>,
}

impl<T: ConnectionTrait> SimulationReportPostgresRepository<T> {
    pub fn new(db: Arc<T>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<T: ConnectionTrait + Send + 'static> SimulationReportRepository
for SimulationReportPostgresRepository<T>
{
    async fn save(&self, simulation_report: simulator_core_api::SimulationReport) -> Result<()> {
        let simulation_report = simulation_report::ActiveModel {
            simulation_id: ActiveValue::Set(simulation_report.simulation_id),
            start: ActiveValue::Set(simulation_report.start.naive_utc()),
            end: ActiveValue::Set(simulation_report.end.naive_utc()),
            deployments: ActiveValue::Set(json!(simulation_report.deployments)),
            ticks: ActiveValue::Set(simulation_report.ticks as i32),
            actions: ActiveValue::Set(simulation_report.actions as i32),
            profit: ActiveValue::Set(simulation_report.profit),
            profit_clear: ActiveValue::Set(simulation_report.profit_clear),
            fees: ActiveValue::Set(simulation_report.fees),
            assets: ActiveValue::Set(json!(simulation_report.assets)),
            active_orders: ActiveValue::Set(json!(simulation_report.active_orders)),
        };
        SimulationReport::insert(simulation_report)
            .on_conflict(
                sea_query::OnConflict::column(simulation_report::Column::SimulationId)
                    .update_columns(vec![
                        simulation_report::Column::Deployments,
                        simulation_report::Column::Ticks,
                        simulation_report::Column::Actions,
                        simulation_report::Column::Profit,
                        simulation_report::Column::ProfitClear,
                        simulation_report::Column::Fees,
                        simulation_report::Column::Assets,
                        simulation_report::Column::ActiveOrders,
                    ])
                    .to_owned(),
            )
            .exec(self.db.deref())
            .await?;
        Ok(())
    }

    async fn get(&self, id: Option<Uuid>) -> Vec<simulator_core_api::SimulationReport> {
        let mut condition = Condition::all();
        if let Some(id) = id {
            condition = condition.add(simulation_report::Column::SimulationId.eq(id));
        }
        simulation_report::Entity::find()
            .filter(condition)
            .all(self.db.deref())
            .await
            .unwrap()
            .into_iter()
            .map(|model| simulator_core_api::SimulationReport {
                simulation_id: model.simulation_id,
                start: model.start.and_utc(),
                end: model.end.and_utc(),
                deployments: serde_json::from_value(model.deployments).unwrap(),
                ticks: model.ticks as u32,
                actions: model.actions as u32,
                profit: model.profit,
                profit_clear: model.profit_clear,
                fees: model.fees,
                assets: serde_json::from_value(model.assets).unwrap(),
                active_orders: serde_json::from_value(model.active_orders).unwrap(),
            })
            .collect()
    }
}
