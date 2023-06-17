use domain_model::{InstrumentId, Timeframe};
use indicators_calculation::moving_average;
use storage_rest_client::StorageClient;

pub struct Indicators {
    storage_client: StorageClient,
}

impl Indicators {
    pub fn new(storage_client: StorageClient) -> Self {
        Self {
            storage_client
        }
    }

    pub async fn moving_average(&self, instrument_id: &InstrumentId, timeframe: Timeframe, length: usize) -> f64 {
        let candles = self.storage_client.get_candles(instrument_id,
                                                      Some(timeframe),
                                                      None,
                                                      None,
                                                      Some(length as u64))
            .await
            .unwrap();
        let values: Vec<_> = candles.into_iter()
            .map(|candle| candle.close_price)
            .collect();
        *moving_average(&values, length)
            .unwrap()
            .first()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use domain_model::{Currency, CurrencyPair, Exchange, MarketType};
    use super::*;

    #[tokio::test]
    async fn test_moving_average() {
        let indicators = Indicators::new(StorageClient::new("http://localhost:8082"));
        let instrument_id = InstrumentId {
            exchange: Exchange::OKX,
            market_type: MarketType::Spot,
            pair: CurrencyPair {
                target: Currency::BTC,
                source: Currency::USDT,
            },
        };
        let moving_average = indicators.moving_average(&instrument_id, Timeframe::OneH, 7).await;
        println!("Moving AVG: {}", moving_average);
    }
}
