use nanoid::nanoid;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::SubscriberBuilder;

const ID_KEYS: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

pub fn string_id() -> String {
    nanoid!(32, &ID_KEYS)
}

pub fn init_logger(directives: &str) {
    let subscriber = SubscriberBuilder::default()
        .with_env_filter(EnvFilter::new(directives))
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");
}
