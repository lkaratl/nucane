//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "order")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub timestamp: DateTime,
    pub simulation_id: Option<Uuid>,
    pub status: Json,
    pub exchange: String,
    pub pair: Json,
    pub market_type: Json,
    pub order_type: Json,
    pub side: String,
    pub size: Json,
    #[sea_orm(column_type = "Double")]
    pub avg_price: f64,
    pub stop_loss: Option<Json>,
    pub take_profit: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
