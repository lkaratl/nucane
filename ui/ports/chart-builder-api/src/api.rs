use std::fmt::Display;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

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
    Line(f64),
}

#[derive(Debug)]
pub struct Coord {
    pub x: DateTime<Utc>,
    pub y: f64,
}

impl From<(DateTime<Utc>, f64)> for Coord {
    fn from(value: (DateTime<Utc>, f64)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
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

#[derive(Debug)]
pub enum Icon {
    Arrow,
    Pin,
    Circle,
}

#[derive(Debug)]
pub enum Color {
    Green,
    Red,
}

impl ToString for Color {
    fn to_string(&self) -> String {
        match self {
            Color::Green => "#00FF09".to_string(),
            Color::Red => "#FF0000".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum LineStyle {
    Solid,
    Dotted,
    Dashed,
}
