use async_trait::async_trait;
use chrono::{DateTime, Utc};

pub use domain_model::drawing::{Color, Coord, Icon, LineStyle};

#[async_trait]
pub trait ChartBuilderApi: Send + Sync + 'static {
    async fn build(
        &self,
        title: &str,
        timestamps: Vec<DateTime<Utc>>,
        series: Vec<Series>,
        points: Vec<Point>,
        lines: Vec<Line>,
    ) -> String;
}

pub struct Series {
    pub label: String,
    pub data: Data,
}

impl Series {
    pub fn new(label: &str, data: Data) -> Self {
        Self {
            label: label.to_string(),
            data,
        }
    }
}

pub enum Data {
    CandleStick(Vec<Vec<f64>>),
}

#[derive(Debug)]
pub struct Point {
    pub label: String,
    pub icon: Option<Icon>,
    pub color: Option<Color>,
    pub text: Option<String>,
    pub coord: Coord,
}

impl Point {
    pub fn new(
        label: &str,
        icon: Option<Icon>,
        color: Option<Color>,
        text: Option<String>,
        coord: Coord,
    ) -> Self {
        Self {
            label: label.to_string(),
            icon,
            color,
            text,
            coord,
        }
    }
}

impl From<domain_model::drawing::Point> for Point {
    fn from(value: domain_model::drawing::Point) -> Self {
        Self {
            label: value.label,
            icon: value.icon,
            color: value.color,
            text: value.text,
            coord: value.coord,
        }
    }
}

#[derive(Debug)]
pub struct Line {
    pub label: String,
    pub style: Option<LineStyle>,
    pub color: Option<Color>,
    pub start: Coord,
    pub end: Coord,
}

impl Line {
    pub fn new(
        label: &str,
        style: Option<LineStyle>,
        color: Option<Color>,
        start: Coord,
        end: Coord,
    ) -> Self {
        Self {
            label: label.to_string(),
            style,
            color,
            start,
            end,
        }
    }
}

impl From<domain_model::drawing::Line> for Line {
    fn from(value: domain_model::drawing::Line) -> Self {
        Self {
            label: value.label,
            color: value.color,
            style: value.style,
            start: value.start,
            end: value.end,
        }
    }
}
