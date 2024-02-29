use Currency::*;
use domain_model::Currency;

pub fn round_price(currency: Currency, value: f64) -> f64 {
    match currency {
        BTC | ETH | SOL => (value * 100.).round() / 100.,
        _ => (value * 10_000.).round() / 10_000.
    }
}

pub fn round_qty(currency: Currency, value: f64) -> f64 {
    match currency {
        BTC | ETH => (value * 1_000_000.).round() / 1_000_000.,
        XRP => (value * 100.).round() / 100.,
        SOL | AVAX | LINK => (value * 1000.).round() / 1000.,
        _ => (value * 10_000.).round() / 10_000.
    }
}