use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use sea_orm::{ActiveValue, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, sea_query};
use serde_json::json;

use domain_model::{CandleStatus, CurrencyPair, Exchange, InstrumentId, MarketType, Timeframe};

use crate::entities::{*, prelude::*};

pub struct CandleService<T: ConnectionTrait> {
    repository: CandleRepository<T>,
}

impl<T: ConnectionTrait> CandleService<T> {
    pub fn new(db: Arc<T>) -> Self {
        CandleService { repository: CandleRepository { db } }
    }

    pub async fn save(&self, candle: domain_model::Candle) {
        self.repository.save(candle)
            .await
            .expect("Error during order saving");
    }

    pub async fn save_many(&self, candles: Vec<domain_model::Candle>) {
        self.repository.save_many(candles)
            .await
            .expect("Error during order saving");
    }

    pub async fn get(&self,
               instrument_id: &InstrumentId,
               timeframe: Option<Timeframe>,
               from_timestamp: Option<DateTime<Utc>>,
               to_timestamp: Option<DateTime<Utc>>,
               limit: Option<u64>) -> Vec<domain_model::Candle> {
        self.repository.find_by(
            instrument_id.exchange,
            instrument_id.market_type,
            instrument_id.pair,
            timeframe,
            from_timestamp,
            to_timestamp,
            limit,
        ).await.unwrap()
    }
}

struct CandleRepository<T: ConnectionTrait> {
    db: Arc<T>,
}

impl<T: ConnectionTrait> CandleRepository<T> {
    async fn save(&self, candle: domain_model::Candle) -> Result<(), DbErr> {
        let candle = candle::ActiveModel {
            id: ActiveValue::Set(candle.id),
            status: ActiveValue::Set(candle.status.to_string()),
            pair: ActiveValue::Set(json!(candle.instrument_id.pair)),
            exchange: ActiveValue::Set(candle.instrument_id.exchange.to_string()),
            market_type: ActiveValue::Set(candle.instrument_id.market_type.to_string()),
            timestamp: ActiveValue::Set(candle.timestamp),
            timeframe: ActiveValue::Set(candle.timeframe.to_string()),
            open_price: ActiveValue::Set(candle.open_price),
            highest_price: ActiveValue::Set(candle.highest_price),
            lower_price: ActiveValue::Set(candle.lowest_price),
            close_price: ActiveValue::Set(candle.close_price),
            target_volume: ActiveValue::Set(candle.target_volume),
            source_volume: ActiveValue::Set(candle.source_volume),
        };
        Candle::insert(candle)
            .on_conflict(
                sea_query::OnConflict::column(candle::Column::Id)
                    .update_columns(vec![
                        candle::Column::Status,
                        candle::Column::HighestPrice,
                        candle::Column::LowerPrice,
                        candle::Column::ClosePrice,
                        candle::Column::TargetVolume,
                        candle::Column::SourceVolume,
                    ]).to_owned()
            )
            .exec(self.db.deref())
            .await?;
        Ok(())
    }

    async fn save_many(&self, candles: Vec<domain_model::Candle>) -> Result<(), DbErr> {
        let candles: Vec<_> = candles.into_iter()
            .map(|candle| {
                candle::ActiveModel {
                    id: ActiveValue::Set(candle.id),
                    status: ActiveValue::Set(candle.status.to_string()),
                    pair: ActiveValue::Set(json!(candle.instrument_id.pair)),
                    exchange: ActiveValue::Set(candle.instrument_id.exchange.to_string()),
                    market_type: ActiveValue::Set(candle.instrument_id.market_type.to_string()),
                    timestamp: ActiveValue::Set(candle.timestamp),
                    timeframe: ActiveValue::Set(candle.timeframe.to_string()),
                    open_price: ActiveValue::Set(candle.open_price),
                    highest_price: ActiveValue::Set(candle.highest_price),
                    lower_price: ActiveValue::Set(candle.lowest_price),
                    close_price: ActiveValue::Set(candle.close_price),
                    target_volume: ActiveValue::Set(candle.target_volume),
                    source_volume: ActiveValue::Set(candle.source_volume),
                }
            }).collect();
        Candle::insert_many(candles)
            .on_conflict(
                sea_query::OnConflict::column(candle::Column::Id)
                    .update_columns(vec![
                        candle::Column::Status,
                        candle::Column::HighestPrice,
                        candle::Column::LowerPrice,
                        candle::Column::ClosePrice,
                        candle::Column::TargetVolume,
                        candle::Column::SourceVolume,
                    ]).to_owned()
            )
            .exec(self.db.deref())
            .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn find_by(&self,
                   exchange: Exchange,
                   market_type: MarketType,
                   pair: CurrencyPair,
                   timeframe: Option<Timeframe>,
                   from_timestamp: Option<DateTime<Utc>>,
                   to_timestamp: Option<DateTime<Utc>>,
                   limit: Option<u64>) -> Result<Vec<domain_model::Candle>, DbErr> {
        let mut condition = Condition::all()
            .add(candle::Column::Exchange.eq(exchange.to_string()))
            .add(candle::Column::Pair.contains(&pair.target.to_string())) // todo pair flatten in db model
            .add(candle::Column::Pair.contains(&pair.source.to_string()))
            .add(candle::Column::MarketType.eq(market_type.to_string()));
        if let Some(timeframe) = timeframe {
            condition = condition.add(candle::Column::Timeframe.eq(timeframe.to_string()));
        }
        if let Some(from_timestamp) = from_timestamp {
            condition = condition.add(candle::Column::Timestamp.gte(from_timestamp));
        }
        if let Some(to_timestamp) = to_timestamp {
            condition = condition.add(candle::Column::Timestamp.lte(to_timestamp));
        }

        let result = candle::Entity::find()
            .filter(condition)
            .apply_if(limit, QuerySelect::limit)
            .order_by_desc(candle::Column::Timestamp)
            .all(self.db.deref())
            .await?
            .into_iter()
            .map(|model| {
                domain_model::Candle {
                    id: model.id,
                    status: CandleStatus::from_str(&model.status).unwrap(),
                    instrument_id: InstrumentId {
                        exchange: Exchange::from_str(&model.exchange).unwrap(),
                        market_type: MarketType::from_str(&model.market_type).unwrap(),
                        pair: serde_json::from_value(model.pair).unwrap(),
                    },
                    timestamp: model.timestamp,
                    timeframe: Timeframe::from_str(&model.timeframe).unwrap(),
                    open_price: model.open_price,
                    highest_price: model.highest_price,
                    lowest_price: model.lower_price,
                    close_price: model.close_price,
                    target_volume: model.target_volume,
                    source_volume: model.source_volume,
                }
            }).collect();

        Ok(result)
    }
}
