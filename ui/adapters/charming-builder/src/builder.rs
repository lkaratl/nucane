use async_trait::async_trait;
use charming::component::{Axis, DataZoom, DataZoomType, Grid, Legend};
use charming::element::{
    AreaStyle, AxisPointer, AxisPointerType, AxisType, DataBackground, Emphasis, EmphasisFocus,
    Formatter, ItemStyle, Label, LabelPosition, LineStyle, LineStyleType, SplitLine, Symbol,
    TextStyle, Tooltip, Trigger,
};
use charming::series::{Candlestick, Scatter};
use charming::theme::Theme;
use charming::{df, Chart, HtmlRenderer};
use chrono::{DateTime, Utc};

use ui_chart_builder_api::{ChartBuilderApi, Data, Icon, Line, Point, Series};

pub struct CharmingBuilder;

#[async_trait]
impl ChartBuilderApi for CharmingBuilder {
    async fn build(
        &self,
        title: &str,
        timestamps: Vec<DateTime<Utc>>,
        series: Vec<Series>,
        points: Vec<Point>,
        lines: Vec<Line>,
    ) -> String {
        HtmlRenderer::new(title, 4000, 1300)
            .theme(Theme::Default)
            .render(&self.build_chart(timestamps, series, points, lines))
            .unwrap()
    }
}

impl CharmingBuilder {
    fn build_chart(
        &self,
        timestamps: Vec<DateTime<Utc>>,
        series: Vec<Series>,
        points: Vec<Point>,
        lines: Vec<Line>,
    ) -> Chart {
        let mut chart = build_base_chart();
        chart = add_legend(chart, &series, &points, &lines);
        chart = add_x_axis(chart, timestamps);
        chart = add_series(chart, series);
        chart = add_points(chart, points);
        chart = add_lines(chart, lines);
        chart
    }
}

fn build_base_chart() -> Chart {
    Chart::new()
        .tooltip(
            Tooltip::new().trigger(Trigger::Axis).axis_pointer(
                AxisPointer::new()
                    .animation(true)
                    .type_(AxisPointerType::Cross),
            ),
        )
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
        .data_zoom(DataZoom::new().type_(DataZoomType::Inside))
}

fn add_legend(chart: Chart, series: &[Series], points: &[Point], lines: &[Line]) -> Chart {
    let legend = generate_legend(series, points, lines);
    chart.legend(Legend::new().inactive_color("#777").data(legend))
}

fn generate_legend(series: &[Series], points: &[Point], lines: &[Line]) -> Vec<String> {
    let mut legend = Vec::new();
    series
        .iter()
        .for_each(|series| legend.push(series.label.clone()));
    points
        .iter()
        .for_each(|point| legend.push(point.label.clone()));
    lines
        .iter()
        .for_each(|line| legend.push(line.label.clone()));
    legend
}

fn add_x_axis(chart: Chart, values: Vec<DateTime<Utc>>) -> Chart {
    let values = values.iter().map(|value| value.to_string()).collect();
    chart.x_axis(Axis::new().type_(AxisType::Category).data(values))
}

fn add_series(mut chart: Chart, series: Vec<Series>) -> Chart {
    for s in series {
        let s = match s.data {
            Data::CandleStick(data) => Candlestick::new().name(s.label).data(data),
        };
        chart = chart.series(s)
    }
    chart
}

fn add_points(mut chart: Chart, points: Vec<Point>) -> Chart {
    for point in points {
        let mut scatter = Scatter::new()
            .name(point.label)
            .data(df![[point.coord.x.to_string(), point.coord.y]]);

        let symbol = point.icon.map(|icon| match icon {
            Icon::Arrow => Symbol::Arrow,
            Icon::Pin => Symbol::Pin,
            Icon::Circle => Symbol::Circle,
        });
        if let Some(symbol) = symbol {
            scatter = scatter.symbol(symbol);
        }
        let style = point
            .color
            .map(|color| ItemStyle::new().color(color.to_string()));
        if let Some(style) = style {
            scatter = scatter.item_style(style);
        }

        let emphasis = point.text.map(|text| {
            Emphasis::new().focus(EmphasisFocus::Series).label(
                Label::new()
                    .show(true)
                    .formatter(Formatter::String(text))
                    .position(LabelPosition::Top),
            )
        });
        if let Some(emphasis) = emphasis {
            scatter = scatter.emphasis(emphasis);
        }
        chart = chart.series(scatter);
    }
    chart
}

#[allow(clippy::unnecessary_unwrap)]
fn add_lines(mut chart: Chart, lines: Vec<Line>) -> Chart {
    for line in lines {
        let mut mark_line = charming::series::Line::new()
            .name(line.label.as_str())
            .data(df![
                [line.start.x.to_string(), line.start.y],
                [line.end.x.to_string(), line.end.y]
            ]);
        let style = line.style.map(|style| match style {
            ui_chart_builder_api::LineStyle::Solid => LineStyleType::Solid,
            ui_chart_builder_api::LineStyle::Dotted => LineStyleType::Dotted,
            ui_chart_builder_api::LineStyle::Dashed => LineStyleType::Dashed,
        });
        let color = line.color.map(|color| color.to_string());
        if style.is_some() && color.is_some() {
            mark_line =
                mark_line.line_style(LineStyle::new().type_(style.unwrap()).color(color.unwrap()));
        } else if let Some(color) = color {
            mark_line = mark_line.line_style(LineStyle::new().color(color));
        } else if let Some(style) = style {
            mark_line = mark_line.line_style(LineStyle::new().type_(style));
        }
        chart = chart.series(mark_line);
    }
    chart
}

static ICON: &str = "path://M10.7,11.9v-1.3H9.3v1.3c-4.9,0.3-8.8,4.4-8.8,9.4c0,5,3.9,9.1,8.8,9.4v1.3h1.3v-1.3c4.9-0.3,8.8-4.4,8.8-9.4C19.5,16.3,15.6,12.2,10.7,11.9z M13.3,24.4H6.7V23h6.6V24.4z M13.3,19.6H6.7v-1.4h6.6V19.6z";
