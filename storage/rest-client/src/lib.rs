use anyhow::Error;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde_urlencoded::to_string;
use tracing::{debug, trace};

use domain_model::{Candle, Currency, Exchange, InstrumentId, MarketType, Order, OrderStatus, OrderType, Position, Side, Timeframe, AuditEvent};
use storage_rest_api::dto::SyncReportDto;
use storage_rest_api::endpoints::{GET_AUDIT, GET_CANDLES, GET_ORDERS, GET_POSITIONS, POST_CANDLES_SYNC};
use storage_rest_api::path_query::{AuditQuery, CandlesQuery, CandleSyncQuery, OrdersQuery, PositionsQuery};

pub struct StorageClient {
    url: String,
    client: Client,
}

impl StorageClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Client::new(),
        }
    }

    pub async fn sync_candles(&self,
                             instrument_id: &InstrumentId,
                             timeframes: &[Timeframe],
                             from: DateTime<Utc>,
                             to: Option<DateTime<Utc>>) -> Result<Vec<SyncReportDto>, Error> {
        let timeframes = timeframes.iter()
            .map(|timeframe| timeframe.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let query = CandleSyncQuery {
            timeframes,
            from: from.timestamp_millis(),
            to: to.map(|timestamp| timestamp.timestamp_millis())
        };

        let endpoint = format!("{}{}", self.url, POST_CANDLES_SYNC);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.post(url)
            .json(instrument_id)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }

    pub async fn get_candles(&self,
                             instrument_id: &InstrumentId,
                             timeframe: Option<Timeframe>,
                             from_timestamp: Option<DateTime<Utc>>,
                             to_timestamp: Option<DateTime<Utc>>,
                             limit: Option<u64>) -> Result<Vec<Candle>, Error> {
        let query = CandlesQuery {
            exchange: instrument_id.exchange,
            market_type: instrument_id.market_type,
            target: instrument_id.pair.target,
            source: instrument_id.pair.source,
            timeframe,
            from_timestamp: from_timestamp.map(|timestamp| timestamp.timestamp_millis()),
            to_timestamp: to_timestamp.map(|timestamp| timestamp.timestamp_millis()),
            limit,
        };

        let endpoint = format!("{}{}", self.url, GET_CANDLES);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_orders(&self,
                            id: Option<String>,
                            exchange: Option<Exchange>,
                            market_type: Option<MarketType>,
                            target: Option<Currency>,
                            source: Option<Currency>,
                            status: Option<OrderStatus>,
                            side: Option<Side>,
                            order_type: Option<OrderType>) -> Result<Vec<Order>, Error> {
        let query = OrdersQuery {
            id,
            exchange,
            market_type,
            target,
            source,
            status,
            side,
            order_type,
        };

        let endpoint = format!("{}{}", self.url, GET_ORDERS);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }

    pub async fn get_positions(&self,
                               exchange: Option<Exchange>,
                               currency: Option<Currency>,
                               side: Option<Side>) -> Result<Vec<Position>, Error> {
        let query = PositionsQuery {
            exchange,
            currency,
            side,
        };

        let endpoint = format!("{}{}", self.url, GET_POSITIONS);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }

    pub async fn get_audit(&self,
                           from_timestamp: Option<DateTime<Utc>>,
                           tags: Option<Vec<String>>,
                           limit: Option<u64>) -> Result<Vec<AuditEvent>, Error> {
        let query = AuditQuery {
            from_timestamp: from_timestamp.map(|timestamp| timestamp.timestamp_millis()),
            tags: tags.map(|tags| tags.join(",")),
            limit,
        };

        let endpoint = format!("{}{}", self.url, GET_AUDIT);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        debug!("Request url: {url:?}");
        let result = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use domain_model::{CommonAuditTags, CurrencyPair};
    use super::*;

    #[tokio::test]
    async fn test_get_candles() {
        let instrument_id = InstrumentId {
            exchange: Exchange::OKX,
            market_type: MarketType::Spot,
            pair: CurrencyPair {
                target: Currency::BTC,
                source: Currency::USDT,
            },
        };

        let response = StorageClient::new("http://localhost:8082")
            .get_candles(
                &instrument_id,
                Some(Timeframe::OneD),
                None,
                None,
                None,
            ).await
            .unwrap();
        dbg!(response);
    }

    #[tokio::test]
    async fn test_get_orders() {
        let response = StorageClient::new("http://localhost:8082")
            .get_orders(
                None,
                Some(Exchange::OKX),
                None,
                Some(Currency::BTC),
                None,
                Some(OrderStatus::Completed),
                Some(Side::Sell),
                None,
            ).await
            .unwrap();
        dbg!(response);
    }

    #[tokio::test]
    async fn test_get_positions() {
        let response = StorageClient::new("http://localhost:8082")
            .get_positions(
                Some(Exchange::OKX),
                Some(Currency::BTC),
                Some(Side::Sell),
            ).await
            .unwrap();
        dbg!(response);
    }

    #[tokio::test]
    async fn test_get_audit() {
        let response = StorageClient::new("http://localhost:8082")
            .get_audit(
                None,
                Some(vec![CommonAuditTags::Order.to_string()]),
                Some(2),
            ).await
            .unwrap();
        dbg!(response);
    }
}
