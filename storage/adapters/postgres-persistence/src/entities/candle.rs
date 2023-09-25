//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "candle")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub status: String,
    pub exchange: String,
    pub pair: Json,
    pub market_type: String,
    pub timestamp: DateTimeUtc,
    pub timeframe: String,
    #[sea_orm(column_type = "Double")]
    pub open_price: f64,
    #[sea_orm(column_type = "Double")]
    pub highest_price: f64,
    #[sea_orm(column_type = "Double")]
    pub lower_price: f64,
    #[sea_orm(column_type = "Double")]
    pub close_price: f64,
    #[sea_orm(column_type = "Double")]
    pub target_volume: f64,
    #[sea_orm(column_type = "Double")]
    pub source_volume: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}