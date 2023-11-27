use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::debug;
use uuid::Uuid;

use domain_model::{Candle, Indicator, InstrumentId, Order, OrderStatus, OrderType, Side, Timeframe};
use indicators::Indicators;
use simulator_core_api::SimulatorApi;
use storage_core_api::StorageApi;
use ui_chart_builder_api::{ChartBuilderApi, Color, Data, Icon, Line, Point, Series};
use ui_core_api::UiApi;

pub struct Ui<S: SimulatorApi, R: StorageApi, C: ChartBuilderApi> {
    simulator_client: Arc<S>,
    storage_client: Arc<R>,
    chart_builder: Arc<C>,
    indicators: Arc<Indicators<R>>,
}

impl<S: SimulatorApi, R: StorageApi, C: ChartBuilderApi> Ui<S, R, C> {
    pub fn new(simulator_client: Arc<S>, storage_client: Arc<R>, chart_builder: Arc<C>, indicators: Arc<Indicators<R>>) -> Self {
        Self {
            simulator_client,
            storage_client,
            chart_builder,
            indicators,
        }
    }

    async fn get_candles(
        &self,
        timeframe: Timeframe,
        instrument_id: &InstrumentId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<Candle> {
        let mut candles = self
            .storage_client
            .get_candles(
                instrument_id,
                Some(timeframe),
                Some(from),
                Some(to),
                None,
            )
            .await
            .unwrap();
        candles.reverse();
        candles
    }

    async fn get_series(&self, candles: Vec<Candle>, indicators: Vec<Indicator>, timeframe: Timeframe,
                        timestamps: &[DateTime<Utc>], instrument_id: &InstrumentId) -> Vec<Series> {
        debug!("Calculate series for candles and indicators: '{indicators:?}'");
        let mut series = Vec::new();
        for indicator in indicators {
            debug!("Calculate indicator: '{indicator:?}'");
            let mut set = self.get_indicator_series(timeframe, timestamps, &candles, instrument_id, indicator).await;
            series.append(&mut set);
        }
        series.push(self.get_candle_series(candles));
        series
    }

    fn get_candle_series(&self, candles: Vec<Candle>) -> Series {
        Series::new(
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
        )
    }

    async fn get_indicator_series(&self, timeframe: Timeframe, timestamps: &[DateTime<Utc>], candles: &[Candle], instrument_id: &InstrumentId, indicator: Indicator) -> Vec<Series> {
        let mut series = Vec::new();
        match indicator {
            Indicator::SMA(period) => {
                let mut data = Vec::new();
                for timestamp in timestamps {
                    let value = self.indicators
                        .simple_moving_average(instrument_id, timeframe, *timestamp, period).await;
                    data.push(value);
                }
                series.push(Series::new(
                    &indicator.to_string(),
                    Data::Line(data),
                ))
            }
            Indicator::EMA(period) => {
                let mut data = Vec::new();
                for timestamp in timestamps {
                    let value = self.indicators
                        .exponential_moving_average(instrument_id, timeframe, *timestamp, period).await;
                    data.push(value);
                }
                series.push(Series::new(
                    &indicator.to_string(),
                    Data::Line(data),
                ))
            }
            Indicator::BB(period, multiplier) => {
                let mut upper_data = Vec::new();
                let mut average_data = Vec::new();
                let mut lower_data = Vec::new();
                for timestamp in timestamps {
                    let value = self.indicators
                        .bollinger_bands(instrument_id, timeframe, *timestamp, period, multiplier).await;
                    upper_data.push(value.upper);
                    average_data.push(value.average);
                    lower_data.push(value.lower);
                }
                series.push(Series::new(
                    &indicator.to_string(),
                    Data::Line(upper_data),
                ));
                series.push(Series::new(
                    &indicator.to_string(),
                    Data::Line(average_data),
                ));
                series.push(Series::new(
                    &indicator.to_string(),
                    Data::Line(lower_data),
                ));
            }
            Indicator::PSAR => {
                let avg_candle_prices: Vec<_> = candles.iter()
                    .map(|candle| candle.avg_price())
                    .collect();
                let mut data = Vec::new();
                for (i, timestamp) in timestamps.iter().enumerate() {
                    let value = self.indicators.parabolic_sar(instrument_id, timeframe, *timestamp).await;
                    if let Some(value) = value {
                        let color = match value {
                            Side::Buy => Color::Green,
                            Side::Sell => Color::Red
                        };
                        let price = match value {
                            Side::Buy => avg_candle_prices.get(i).unwrap() * 0.98,
                            Side::Sell => avg_candle_prices.get(i).unwrap() * 1.02,
                        };
                        let coord = (*timestamp, price).into();
                        let point = Point::new(&indicator.to_string(), Icon::Pin.into(), color.into(), None, coord);
                        data.push(point)
                    }
                }
                series.push(Series::new(
                    &indicator.to_string(),
                    Data::Points(data),
                ))
            }
        };
        series
    }

    async fn get_points(
        &self,
        simulation_id: Uuid,
        deployment_id: Uuid,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
    ) -> Vec<Point> {
        debug!("Build points");
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
                    order.avg_fill_price
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
        debug!("Build lines");
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
        debug!("Start chart building for simulation: '{simulation_id}' and deployment: '{deployment_id}' with instrument: '{instrument_id:?}'");
        let timeframe = timeframe.unwrap_or(Timeframe::FiveM);
        let simulation_report = self
            .simulator_client
            .get_simulation_report(simulation_id)
            .await
            .unwrap();
        let deployment = simulation_report.deployments.iter()
            .find(|deployment| deployment.deployment_id.unwrap() == deployment_id)
            .unwrap()
            .clone();

        let candles = self
            .get_candles(timeframe, &instrument_id, simulation_report.start, simulation_report.end)
            .await;

        let timestamps: Vec<_> = candles.iter().map(|candle| candle.timestamp).collect();

        let series = self.get_series(candles, deployment.indicators, timeframe, &timestamps, &instrument_id).await;

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
            .build(&title, timestamps, series, points, lines)
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
