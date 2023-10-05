use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use domain_model::Timeframe;
use simulator_core_api::SimulatorApi;
use storage_core_api::StorageApi;
use ui_chart_builder_api::{ChartBuilderApi, Series};
use ui_core_api::UiApi;

pub struct Ui<S: SimulatorApi, R: StorageApi, C: ChartBuilderApi> {
    simulator_client: Arc<S>,
    storage_client: Arc<R>,
    chart_builder: Arc<C>,
}

impl<S: SimulatorApi, R: StorageApi, C: ChartBuilderApi> Ui<S, R, C> {
    pub fn new(simulator_client: Arc<S>, storage_client: Arc<R>, chart_builder: Arc<C>) -> Self {
        Self {
            simulator_client,
            storage_client,
            chart_builder,
        }
    }
}

#[async_trait]
impl<S: SimulatorApi, R: StorageApi, C: ChartBuilderApi> UiApi for Ui<S, R, C> {
    async fn get_simulation_chart_html(
        &self,
        simulation_id: Uuid,
        timeframe: Option<Timeframe>,
    ) -> Result<String> {
        let timeframe = timeframe.unwrap_or(Timeframe::FiveM);
        let simulation_report = self
            .simulator_client
            .get_simulation_report(simulation_id)
            .await
            .unwrap();
        let mut series = Vec::new();
        let mut timestamps = Vec::new();
        for instrument_id in simulation_report
            .deployments
            .into_iter()
            .flat_map(|deployment| deployment.subscriptions)
        {
            let candles = self
                .storage_client
                .get_candles(
                    &instrument_id,
                    Some(timeframe),
                    Some(simulation_report.start),
                    Some(simulation_report.end),
                    None,
                )
                .await
                .unwrap();
            timestamps = candles.iter().map(|candle| candle.timestamp).collect();
            let data = candles
                .iter()
                .map(|candle| {
                    vec![
                        candle.open_price,
                        candle.close_price,
                        candle.lowest_price,
                        candle.highest_price,
                    ]
                })
                .collect();
            let label = format!(
                "{}/{}",
                instrument_id.pair.target, instrument_id.pair.source
            );
            series.push(Series { label, data })
        }
        let chart_html = self.chart_builder.build(timestamps, series).await;
        Ok(chart_html)
    }
}
