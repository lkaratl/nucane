use chrono::{DateTime, Utc};
use http::Method;
use serde::{Deserialize, Serialize};

use crate::enums::{InstType, OrdState, OrdType, PosSide, Side, TdMode};
use crate::okx::parser::ts_milliseconds;

use super::super::Request;

const STOP_LOSS_TYPE: &str = "mark";
const TAKE_PROFIT_TYPE: &str = "mark";

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
    pub ord_type: OrdType,
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
    pub fn market(inst_id: &str, td_mode: TdMode, side: Side, qty: Size, stop_loss: Option<Trigger>, take_profit: Option<Trigger>) -> Self {
        let (tgt_ccy, qty) = match qty {
            Size::Target(qty) => ("base_ccy".to_string(), qty),
            Size::Source(qty) => ("quote_ccy".to_string(), qty)
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
            ccy: None,
            tgt_ccy: Some(tgt_ccy),
            tag: None,
            side,
            pos_side: None,
            ord_type: OrdType::Market,
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

    pub fn limit(inst_id: &str, td_mode: TdMode, side: Side, price: f64, qty: Size, stop_loss: Option<Trigger>, take_profit: Option<Trigger>) -> Self {
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
            ccy: None,
            tgt_ccy: None,
            tag: None,
            side,
            pos_side: None,
            ord_type: OrdType::Limit,
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
    #[serde(deserialize_with = "crate::okx::parser::from_str_opt")]
    pub cl_ord_id: Option<String>,
    pub tag: String,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub s_code: u64,
    #[serde(deserialize_with = "crate::okx::parser::from_str_opt")]
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
    pub inst_type: InstType,
    pub inst_id: String,
    pub ccy: String,
    pub ord_id: String,
    pub cl_ord_id: String,
    pub tag: String,
    #[serde(deserialize_with = "crate::okx::parser::from_str_opt")]
    pub px: Option<f64>,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub sz: f64,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub pnl: f64,
    pub ord_type: OrdType,
    pub side: Side,
    pub tgt_ccy: String,
    // pub pos_side: PosSide,
    pub td_mode: TdMode,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub acc_fill_sz: f64,
    pub fill_px: String,
    pub trade_id: String,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub fill_sz: f64,
    pub fill_time: String,
    pub state: OrdState,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub avg_px: f64,
    pub lever: String,
    #[serde(deserialize_with = "crate::okx::parser::from_str_opt")]
    pub sl_trigger_px: Option<f64>,
    #[serde(deserialize_with = "crate::okx::parser::from_str_opt")]
    pub sl_ord_px: Option<f64>,
    #[serde(deserialize_with = "crate::okx::parser::from_str_opt")]
    pub tp_trigger_px: Option<f64>,
    #[serde(deserialize_with = "crate::okx::parser::from_str_opt")]
    pub tp_ord_px: Option<f64>,
    pub fee_ccy: String,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub fee: f64,
    pub rebate_ccy: String,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub rebate: f64,
    pub category: String,
    #[serde(deserialize_with = "ts_milliseconds")]
    pub u_time: DateTime<Utc>,
    #[serde(deserialize_with = "ts_milliseconds")]
    pub c_time: DateTime<Utc>,
}

impl Request for OrderDetailsRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = true;
    const ENDPOINT: &'static str = "/api/v5/trade/order";
    const HAS_PAYLOAD: bool = true;
    type Response = [OrderDetailsResponse; 1];
}
