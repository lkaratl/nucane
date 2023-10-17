use std::{env, io};

use nanoid::nanoid;
use tracing_subscriber::{EnvFilter, fmt, Layer};
use tracing_subscriber::layer::SubscriberExt;

const ID_KEYS: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

pub fn string_id() -> String {
    nanoid!(32, &ID_KEYS)
}

pub fn init_logger(file_name: &str, directives: &str) {
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push("nucane/logs");
    let file_appender = tracing_appender::rolling::never(tmp_dir, format!("{file_name}.log"));

    let log_output = fmt::Layer::new()
        .with_writer(io::stdout)
        .with_file(true)
        .with_line_number(true)
        .and_then(
            fmt::Layer::new()
                .with_writer(file_appender)
                .with_file(true)
                .with_line_number(true),
        );

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::new(directives))
        .with(log_output);

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
}
