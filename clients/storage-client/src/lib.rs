use anyhow::Error;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde::Serialize;
use serde_urlencoded::to_string as to_ustring;
use tracing::{debug, trace};

use domain_model::{Candle, Currency, Exchange, InstrumentId, MarketType, Order, OrderStatus, OrderType, Position, Side, Timeframe, AuditEvent};

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

        let endpoint = format!("{}{}", self.url, "/api/v1/storage/candles");
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_ustring(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }

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

        let endpoint = format!("{}{}", self.url, "/api/v1/storage/orders");
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_ustring(&query)?));
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
        let query = PositionQuery {
            exchange,
            currency,
            side,
        };

        let endpoint = format!("{}{}", self.url, "/api/v1/storage/positions");
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_ustring(&query)?));
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

        let endpoint = format!("{}{}", self.url, "/api/v1/storage/audit");
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_ustring(&query)?));
        debug!("Request url: {url:?}");
        let result = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }
}

#[derive(Serialize)]
struct CandlesQuery {
    exchange: Exchange,
    market_type: MarketType,
    target: Currency,
    source: Currency,
    timeframe: Option<Timeframe>,
    from_timestamp: Option<i64>,
    to_timestamp: Option<i64>,
    limit: Option<u64>,
}

#[derive(Serialize)]
struct OrdersQuery {
    id: Option<String>,
    exchange: Option<Exchange>,
    market_type: Option<MarketType>,
    target: Option<Currency>,
    source: Option<Currency>,
    status: Option<OrderStatus>,
    side: Option<Side>,
    order_type: Option<OrderType>,
}

#[derive(Serialize)]
struct PositionQuery {
    exchange: Option<Exchange>,
    currency: Option<Currency>,
    side: Option<Side>,
}

#[derive(Serialize)]
struct AuditQuery {
    from_timestamp: Option<i64>,
    tags: Option<String>,
    limit: Option<u64>,
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
