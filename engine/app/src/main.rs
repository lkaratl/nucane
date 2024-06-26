use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::SubscriberBuilder;

use engine_config::CONFIG;

#[tokio::main]
async fn main() {
    init_logger();
    engine_app::run().await;
}

fn init_logger() {
    let subscriber = SubscriberBuilder::default()
        .with_env_filter(EnvFilter::new(CONFIG.logging.levels()))
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");
}
