use async_trait::async_trait;
use charming::component::{Axis, DataZoom, DataZoomType, Grid, Legend};
use charming::element::{
    AreaStyle, AxisPointer, AxisPointerType, AxisType, DataBackground, LineStyle, SplitLine,
    TextStyle, Tooltip, Trigger,
};
use charming::series::Candlestick;
use charming::theme::Theme;
use charming::{Chart, HtmlRenderer};
use chrono::{DateTime, Utc};

use ui_chart_builder_api::{ChartBuilderApi, Series};

pub struct CharmingBuilder;

#[async_trait]
impl ChartBuilderApi for CharmingBuilder {
    async fn build(&self, timestamps: Vec<DateTime<Utc>>, series: Vec<Series>) -> String {
        HtmlRenderer::new("Simulation Chart", 4000, 1300)
            .theme(Theme::Default)
            .render(&self.build_chart(timestamps, series))
            .unwrap()
    }
}

impl CharmingBuilder {
    fn build_chart(&self, timestamps: Vec<DateTime<Utc>>, series: Vec<Series>) -> Chart {
        let timestamps = timestamps
            .iter()
            .map(|timestamp| timestamp.to_string())
            .rev()
            .collect();
        let legend = series.iter().map(|s| s.label.clone()).collect();
        let mut chart = Chart::new()
            .legend(Legend::new().inactive_color("#777").data(legend))
            .tooltip(
                Tooltip::new().trigger(Trigger::Axis).axis_pointer(
                    AxisPointer::new()
                        .animation(true)
                        .type_(AxisPointerType::Cross),
                ),
            )
            .x_axis(Axis::new().type_(AxisType::Category).data(timestamps))
            .y_axis(
                Axis::new()
                    .scale(true)
                    .split_line(SplitLine::new().show(false)),
            )
            .grid(Grid::new().bottom(80))
            .data_zoom(
                DataZoom::new()
                    .handle_icon(ICON)
                    .text_style(TextStyle::new().color("#8392A5"))
                    .data_background(
                        DataBackground::new()
                            .area_style(AreaStyle::new().color("#8392A5"))
                            .line_style(LineStyle::new().color("#8392A5")),
                    )
                    .brush_select(true),
            )
            .data_zoom(DataZoom::new().type_(DataZoomType::Inside));
        for s in series {
            chart = chart.series(Candlestick::new().name(s.label).data(s.data));
        }
        chart
    }
}

static ICON: &str = "path://M10.7,11.9v-1.3H9.3v1.3c-4.9,0.3-8.8,4.4-8.8,9.4c0,5,3.9,9.1,8.8,9.4v1.3h1.3v-1.3c4.9-0.3,8.8-4.4,8.8-9.4C19.5,16.3,15.6,12.2,10.7,11.9z M13.3,24.4H6.7V23h6.6V24.4z M13.3,19.6H6.7v-1.4h6.6V19.6z";
