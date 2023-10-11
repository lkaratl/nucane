use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{from_value, Value};
use tracing::trace;

use domain_model::{Currency, Exchange, Position, Side};
use eac::rest::Account;
use eac::websocket::{Action, Channel, WsMessageHandler};
use storage_core_api::StorageApi;

pub struct PositionHandler<S: StorageApi> {
    positions: HashMap<String, f64>,
    storage_client: Arc<S>,
}

impl<S: StorageApi> PositionHandler<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self {
            positions: HashMap::new(),
            storage_client,
        }
    }
}

#[async_trait]
impl<S: StorageApi> WsMessageHandler for PositionHandler<S> {
    type Type = Vec<Position>;

    async fn convert_data(
        &mut self,
        _arg: Channel,
        _action: Option<Action>,
        data: Vec<Value>,
    ) -> Option<Self::Type> {
        trace!("Retrieved massage with raw payload: {:?}", &data);
        let mut new_positions = Vec::new();
        for item in data {
            let account: Account = from_value(item).unwrap();
            for asset in account.details {
                let previous_ccy_amount = self.positions.entry(asset.ccy.clone()).or_insert(0.0);
                if *previous_ccy_amount != asset.avail_bal {
                    let currency = Currency::from_str(&asset.ccy).unwrap();
                    let size = asset.avail_bal;
                    let side = if size < 0.0 { Side::Sell } else { Side::Buy };
                    let position = Position::new(None, Exchange::OKX, currency, side, size);
                    new_positions.push(position);
                    self.positions.insert(asset.ccy, asset.avail_bal);
                }
            }
        }
        Some(new_positions)
    }

    async fn handle(&mut self, message: Self::Type) {
        for position in message {
            self.storage_client.save_position(position).await.unwrap();
        }
    }
}
