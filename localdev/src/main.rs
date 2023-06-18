use std::fmt::Debug;
use std::io;

use tracing::{Level, subscriber};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, FmtSubscriber, Layer, Registry, prelude::*};
use tracing_subscriber::fmt::{Formatter, Subscriber, SubscriberBuilder};
use tracing_subscriber::layer::SubscriberExt;
use tracing_appender::non_blocking::WorkerGuard;

use crate::config::CONFIG;

mod config;

fn main() {
    init_logger();
    localdev::run();
    loop {}
}

fn init_logger() {
    let file_appender = tracing_appender::rolling::never("./", "nucane.log");

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
