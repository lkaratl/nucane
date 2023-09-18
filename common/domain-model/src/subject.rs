use synapse::core::MessageSubject;
use crate::{Candle, Order, Position, Tick};

pub struct ExchangeTickSubject;
pub struct ExchangeCandleSubject;
pub struct ExchangeOrderSubject;
pub struct ExchangePositionSubject;
pub struct ExchangePlaceOrderSubject;

impl MessageSubject for ExchangeTickSubject {
    type MessageType = Tick;
    const SUBJECT: &'static str = "exchange.tick";
}
impl MessageSubject for ExchangeCandleSubject {
    type MessageType = Candle;
    const SUBJECT: &'static str = "exchange.candle";
}

impl MessageSubject for ExchangeOrderSubject {
    type MessageType = Order;
    const SUBJECT: &'static str = "exchange.order";
}

impl MessageSubject for ExchangePositionSubject {
    type MessageType = Position;
    const SUBJECT: &'static str = "exchange.position";
}
