use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::InstrumentId;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    pub id: Uuid,
    pub deployment_id: Uuid,
    pub instrument_id: InstrumentId,
    pub label: String,
    pub icon: Option<Icon>,
    pub color: Option<Color>,
    pub text: Option<String>,
    pub coord: Coord,
}

impl Point {
    pub fn new(
        instrument_id: InstrumentId,
        deployment_id: Uuid,
        label: &str,
        icon: Option<Icon>,
        color: Option<Color>,
        text: Option<String>,
        coord: Coord,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            instrument_id,
            deployment_id,
            label: label.to_string(),
            icon,
            color,
            text,
            coord,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    pub id: Uuid,
    pub deployment_id: Uuid,
    pub instrument_id: InstrumentId,
    pub label: String,
    pub style: Option<LineStyle>,
    pub color: Option<Color>,
    pub start: Coord,
    pub end: Coord,
}

impl Line {
    pub fn new(
        instrument_id: InstrumentId,
        deployment_id: Uuid,
        label: &str,
        style: Option<LineStyle>,
        color: Option<Color>,
        start: Coord,
        end: Coord,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            instrument_id,
            deployment_id,
            label: label.to_string(),
            style,
            color,
            start,
            end,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Icon {
    Arrow,
    Pin,
    Circle,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub enum LineStyle {
    Solid,
    Dotted,
    Dashed,
}
