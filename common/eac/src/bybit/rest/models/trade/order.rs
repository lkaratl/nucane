use chrono::{DateTime, Utc};
use http::Method;
use serde::{Deserialize, Serialize};

use crate::bybit::enums::{OrderCancelType, OrderCategory, OrderRejectReason, OrderStatus, OrderTimeInForce, OrderType, OrdState, PosSide, Side, TdMode};

use super::super::Request;

const STOP_LOSS_TYPE: &str = "last";
// todo mark
const TAKE_PROFIT_TYPE: &str = "last"; // todo mark

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderRequest {
    pub inst_id: String,
    pub td_mode: TdMode,
    pub ccy: Option<String>,
    pub tgt_ccy: Option<String>,
    pub tag: Option<String>,
    pub side: Side,
    pub pos_side: Option<PosSide>,
    pub sz: String,
    pub px: Option<String>,
    pub ban_amend: bool,
    pub cl_ord_id: Option<String>,
    pub sl_trigger_px_type: Option<String>,
    pub sl_trigger_px: Option<f64>,
    pub sl_ord_px: Option<f64>,
    pub tp_trigger_px_type: Option<String>,
    pub tp_trigger_px: Option<f64>,
    pub tp_ord_px: Option<f64>,
}

impl PlaceOrderRequest {
    pub fn market(inst_id: &str, td_mode: TdMode, ccy: Option<String>, side: Side, qty: Size, stop_loss: Option<Trigger>, take_profit: Option<Trigger>) -> Self {
        let qty = match qty {
            Size::Target(qty) => {
                if side == Side::Buy {
                    panic!("Can't create order with Target size and Buy side. Please use Source size");
                }
                qty
            }
            Size::Source(qty) => {
                if side == Side::Sell {
                    panic!("Can't create order with Source size and Sell side. Please use Target size");
                }
                qty
            }
        };
        let (sl_trigger_px_type, sl_trigger_px, sl_ord_px) = if let Some(stop_loss) = stop_loss {
            (Some(STOP_LOSS_TYPE.to_string()), Some(stop_loss.trigger_px), Some(stop_loss.order_px))
        } else {
            (None, None, None)
        };
        let (tp_trigger_px_type, tp_trigger_px, tp_ord_px) = if let Some(take_profit) = take_profit {
            (Some(TAKE_PROFIT_TYPE.to_string()), Some(take_profit.trigger_px), Some(take_profit.order_px))
        } else {
            (None, None, None)
        };
        Self {
            inst_id: inst_id.into(),
            td_mode,
            ccy,
            tgt_ccy: None,
            tag: None,
            side,
            pos_side: None,
            sz: qty.to_string(),
            px: None,
            ban_amend: true,
            cl_ord_id: None,
            sl_trigger_px_type,
            sl_trigger_px,
            sl_ord_px,
            tp_trigger_px_type,
            tp_trigger_px,
            tp_ord_px,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn limit(inst_id: &str, td_mode: TdMode, ccy: Option<String>, side: Side, price: f64, qty: Size, stop_loss: Option<Trigger>, take_profit: Option<Trigger>) -> Self {
        let qty = match qty {
            Size::Target(qty) => qty,
            Size::Source(qty) => qty / price
        };
        let (sl_trigger_px_type, sl_trigger_px, sl_ord_px) = if let Some(stop_loss) = stop_loss {
            (Some(STOP_LOSS_TYPE.to_string()), Some(stop_loss.trigger_px), Some(stop_loss.order_px))
        } else {
            (None, None, None)
        };
        let (tp_trigger_px_type, tp_trigger_px, tp_ord_px) = if let Some(take_profit) = take_profit {
            (Some(TAKE_PROFIT_TYPE.to_string()), Some(take_profit.trigger_px), Some(take_profit.order_px))
        } else {
            (None, None, None)
        };
        Self {
            inst_id: inst_id.into(),
            td_mode,
            ccy,
            tgt_ccy: None,
            tag: None,
            side,
            pos_side: None,
            sz: qty.to_string(),
            px: Some(price.to_string()),
            ban_amend: true,
            cl_ord_id: None,
            sl_trigger_px_type,
            sl_trigger_px,
            sl_ord_px,
            tp_trigger_px_type,
            tp_trigger_px,
            tp_ord_px,
        }
    }

    pub fn set_ccy(&mut self, ccy: &str) -> &mut Self {
        self.ccy = Some(ccy.to_string());
        self
    }

    pub fn set_cl_ord_id(&mut self, cl_ord_id: &str) -> &mut Self {
        self.cl_ord_id = Some(cl_ord_id.to_string());
        self
    }

    pub fn set_tag(&mut self, tag: &str) -> &mut Self {
        self.tag = Some(tag.to_string());
        self
    }
}

pub enum Size {
    Target(f64),
    Source(f64),
}

pub struct Trigger {
    pub trigger_px: f64,
    pub order_px: f64,
}

impl Trigger {
    pub fn new(trigger_px: f64, order_px: f64) -> Option<Self> {
        Some(Self {
            trigger_px,
            order_px,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderResponse {
    pub ord_id: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str_opt")]
    pub cl_ord_id: Option<String>,
    pub tag: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub s_code: u64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str_opt")]
    pub s_msg: Option<String>,
}

impl Request for PlaceOrderRequest {
    const METHOD: Method = Method::POST;
    const SIGNED: bool = true;
    const ENDPOINT: &'static str = "/api/v5/trade/order";
    const HAS_PAYLOAD: bool = true;
    type Response = [PlaceOrderResponse; 1];
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailsRequest {
    pub inst_id: String,
    pub ord_id: Option<String>,
    pub cl_ord_id: Option<String>,
}

impl OrderDetailsRequest {
    pub fn ord_id(inst_id: &str, ord_id: &str) -> Self {
        Self {
            inst_id: inst_id.into(),
            ord_id: Some(ord_id.into()),
            cl_ord_id: None,
        }
    }

    pub fn cl_ord_id(inst_id: &str, cl_ord_id: &str) -> Self {
        Self {
            inst_id: inst_id.into(),
            ord_id: None,
            cl_ord_id: Some(cl_ord_id.into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailsResponse {
    pub category: OrderCategory,
    pub order_id: String,
    pub order_link_id: String,
    pub is_leverage: String,
    pub block_trade_id: String,
    pub symbol: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub price: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub qty: f64,
    pub side: Side,
    pub position_idx: u8,
    pub order_status: OrderStatus,
    pub cancel_type: OrderCancelType,
    pub reject_reason: OrderRejectReason,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub avg_price: f64,
    pub leaves_qty: String,
    pub leaves_value: String,
    pub cum_exec_qty: String,
    pub cum_exec_value: String,
    pub cum_exec_fee: String,
    pub fee_currency: String,
    pub time_in_force: OrderTimeInForce,
    pub order_type: OrderType,
    pub stop_order_type: String,
    pub oco_trigger_type: Option<String>,
    pub order_iv: String,
    pub trigger_price: String,
    pub take_profit: String,
    pub stop_loss: String,
    pub tpsl_mode: Option<String>,
    pub tp_limit_price: Option<String>,
    pub sl_limit_price: Option<String>,
    pub tp_trigger_by: Option<String>,
    pub sl_trigger_by: Option<String>,
    pub trigger_direction: u8,
    pub trigger_by: String,
    pub last_price_on_created: String,
    pub reduce_only: bool,
    pub close_on_trigger: bool,
    pub place_type: String,
    pub smp_type: String,
    pub smp_group: i64,
    pub smp_order_id: String,
    #[serde(deserialize_with = "ts_milliseconds")]
    pub updated_time: DateTime<Utc>,
    #[serde(deserialize_with = "ts_milliseconds")]
    pub created_time: DateTime<Utc>,
}

impl Request for OrderDetailsRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = true;
    const ENDPOINT: &'static str = "/api/v5/trade/order";
    const HAS_PAYLOAD: bool = true;
    type Response = [OrderDetailsResponse; 1];
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderHistoryRequest {
    pub inst_type: String,
    pub inst_id: Option<String>,
    pub state: OrdState,
}

impl Request for OrderHistoryRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = true;
    const ENDPOINT: &'static str = "/api/v5/trade/orders-history";
    const HAS_PAYLOAD: bool = true;
    type Response = Vec<OrderDetailsResponse>;
}
