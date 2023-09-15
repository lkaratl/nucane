use synapse::core::{MessageSubject};
const EXCHANGE_TICK: &str = "exchange.tick";
const EXCHANGE_CANDLE: &str = "exchange.candle";
const EXCHANGE_ORDER: &str = "exchange.order";
const EXCHANGE_POSITION: &str = "exchange.position";
const EXCHANGE_PLACE_ORDER: &str = "exchange.place-order";

pub struct TickSubject;

impl MessageSubject for TickSubject {
    type MessageType = String; // todo change
    const SUBJECT: &'static str = EXCHANGE_CANDLE;
}
