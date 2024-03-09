use Currency::*;
use domain_model::Currency;

pub fn round_price(currency: Currency, value: f64) -> f64 {
    match currency {
        BTC | ETH | SOL | LTC => (value * 100.).round() / 100.,
        ARB | DOT | FIL => (value * 1000.).round() / 1000.,
        SAND | TRX | XLM | DOGE => (value * 10_000.).round() / 10_000.,
        _ => (value * 10_000.).round() / 10_000.
    }
}

pub fn round_qty(currency: Currency, value: f64) -> f64 {
    match currency {
        XLM | DOGE => (value * 10.).round() / 10.,
        XRP | SUI | TRX | MANA | SAND | MATIC | OP | WLD | ARB | ADA | APE | APEX | APT | CHZ | EOS | FIL | GMT | HFT | ICP | LDO =>
            (value * 100.).round() / 100.,
        SOL | AVAX | LINK | DOT => (value * 1000.).round() / 1000.,
        LTC => (value * 100_000.).round() / 100_000.,
        BTC | ETH => (value * 1_000_000.).round() / 1_000_000.,
        _ => (value * 10_000.).round() / 10_000.
    }
}
