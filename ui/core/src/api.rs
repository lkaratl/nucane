use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use domain_model::{Candle, InstrumentId, Order, OrderStatus, OrderType, Side, Timeframe};
use simulator_core_api::SimulatorApi;
use storage_core_api::StorageApi;
use ui_chart_builder_api::{ChartBuilderApi, Color, Data, Icon, Line, Point, Series};
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

    async fn get_candles(
        &self,
        simulation_id: Uuid,
        timeframe: Timeframe,
        instrument_id: &InstrumentId,
    ) -> Vec<Candle> {
        let simulation_report = self
            .simulator_client
            .get_simulation_report(simulation_id)
            .await
            .unwrap();
        let mut candles = self
            .storage_client
            .get_candles(
                instrument_id,
                Some(timeframe),
                Some(simulation_report.start),
                Some(simulation_report.end),
                None,
            )
            .await
            .unwrap();
        candles.reverse();
        candles
    }

    async fn get_points(
        &self,
        simulation_id: Uuid,
        deployment_id: Uuid,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
    ) -> Vec<Point> {
        let mut points = Vec::new();
        points.extend(self.get_order_points(simulation_id).await);
        points.extend(self.get_custom_points(deployment_id, instrument_id).await);

        points
            .iter_mut()
            .for_each(|point| point.coord.x = align_timestamp(point.coord.x, timeframe));
        points
    }

    async fn get_order_points(&self, simulation_id: Uuid) -> Vec<Point> {
        self.storage_client
            .get_orders(
                None,
                Some(simulation_id),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap()
            .into_iter()
            .map(|order| {
                let name = match order.side {
                    Side::Buy => "Buy",
                    Side::Sell => "Sell",
                };
                let icon = match order.status {
                    OrderStatus::Completed => Icon::Arrow,
                    _ => Icon::Circle,
                };
                let color = match order.side {
                    Side::Buy => Color::Green,
                    Side::Sell => Color::Red,
                };
                let info = order_to_info(&order);
                let x = order.timestamp;
                let y = if let OrderType::Limit(limit) = order.order_type {
                    limit
                } else {
                    order.avg_price
                };
                Point::new(name, icon.into(), Some(color), info.into(), (x, y).into())
            })
            .collect()
    }

    async fn get_custom_points(
        &self,
        deployment_id: Uuid,
        instrument_id: &InstrumentId,
    ) -> Vec<Point> {
        self.storage_client
            .get_points(deployment_id, instrument_id)
            .await
            .unwrap()
            .into_iter()
            .map(|point| point.into())
            .collect()
    }

    async fn get_custom_lines(
        &self,
        deployment_id: Uuid,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
    ) -> Vec<Line> {
        self.storage_client
            .get_lines(deployment_id, instrument_id)
            .await
            .unwrap()
            .into_iter()
            .map(Line::from)
            .map(|mut line| {
                line.start.x = align_timestamp(line.start.x, timeframe);
                line.end.x = align_timestamp(line.end.x, timeframe);
                line
            })
            .collect()
    }
}

#[async_trait]
impl<S: SimulatorApi, R: StorageApi, C: ChartBuilderApi> UiApi for Ui<S, R, C> {
    async fn get_simulation_chart_html(
        &self,
        simulation_id: Uuid,
        deployment_id: Uuid,
        timeframe: Option<Timeframe>,
        instrument_id: InstrumentId,
    ) -> Result<String> {
        let timeframe = timeframe.unwrap_or(Timeframe::FiveM);
        let candles = self
            .get_candles(simulation_id, timeframe, &instrument_id)
            .await;

        let timestamps = candles.iter().map(|candle| candle.timestamp).collect();
        let candle_series = Series::new(
            "Candles",
            Data::CandleStick(
                candles
                    .iter()
                    .map(|candle| {
                        vec![
                            candle.open_price,
                            candle.close_price,
                            candle.lowest_price,
                            candle.highest_price,
                        ]
                    })
                    .collect(),
            ),
        );
        let points: Vec<_> = self
            .get_points(simulation_id, deployment_id, &instrument_id, timeframe)
            .await;
        let lines = self
            .get_custom_lines(deployment_id, &instrument_id, timeframe)
            .await;
        let title = format!(
            "{}/{}",
            instrument_id.pair.target, instrument_id.pair.source
        );
        let chart_html = self
            .chart_builder
            .build(&title, timestamps, vec![candle_series], points, lines)
            .await;
        Ok(chart_html)
    }
}

fn align_timestamp(timestamp: DateTime<Utc>, timeframe: Timeframe) -> DateTime<Utc> {
    timestamp - Duration::from_secs((timestamp.timestamp() % timeframe.as_sec()) as u64)
}

fn order_to_info(order: &Order) -> String {
    format!(
        "status: {:?}\ntype: {:?}\nsize: {:?}\nsl: {:?}\ntp: {:?}\n",
        order.status, order.order_type, order.size, order.stop_loss, order.take_profit
    )
}
