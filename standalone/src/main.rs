use std::{io, thread};
use std::time::Duration;

use tracing_subscriber::{EnvFilter, fmt, Layer};
use tracing_subscriber::layer::SubscriberExt;

use crate::config::CONFIG;

mod config;

fn main() {
    init_logger();
    standalone::run();
    thread::sleep(Duration::from_secs(u64::MAX))
}

fn init_logger() {
    let file_appender = tracing_appender::rolling::never("./logs", "nucane.log");

    let log_output = fmt::Layer::new()
        .with_writer(io::stdout)
        .with_file(true)
        .with_line_number(true)
        .and_then(
            fmt::Layer::new().with_writer(file_appender)
                .with_file(true)
                .with_line_number(true));

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::new(CONFIG.logging.levels()))
        .with(log_output);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");
}
