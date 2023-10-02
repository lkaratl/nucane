use std::sync::Arc;
use std::time::Duration;
use std::{env, thread};

use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_fetch::{PgFetchSettings, PostgresVersion};
use pg_embed::postgres::{PgEmbed, PgSettings};
use tracing::{debug, info};

use standalone_config::CONFIG;

pub fn run() {
    let db = Arc::new(run_db());
    thread::spawn(|| run_registry());
    thread::spawn(|| run_engine());
    thread::spawn({
        let db = Arc::clone(&db);
        || run_storage(db)
    });
    thread::spawn(|| run_simulator());
    thread::spawn(|| run_interactor());
    debug!(
        " ▸ standalone: Temp data folder: {}",
        env::temp_dir().display()
    );
    info!("{APP_NAME}");
}

#[tokio::main]
async fn run_db() -> PgEmbed {
    info!("▶ data base running ...");
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push("nucane/postgres");

    let pg_settings = PgSettings {
        database_dir: tmp_dir,
        port: CONFIG.db.port,
        user: CONFIG.db.user.to_string(),
        password: CONFIG.db.password.to_string(),
        auth_method: PgAuthMethod::Plain,
        persistent: CONFIG.db.persistent,
        timeout: Some(Duration::from_secs(15)),
        migration_dir: None,
    };
    let fetch_settings = PgFetchSettings {
        version: PostgresVersion(&CONFIG.db.version),
        ..Default::default()
    };
    let mut pg = PgEmbed::new(pg_settings, fetch_settings)
        .await
        .expect("Error during db creation");
    pg.setup().await.expect("Error during db setup");
    pg.start_db().await.expect("Error during db start");
    pg
}

#[tokio::main]
async fn run_registry() {
    registry_app::run().await;
}

#[tokio::main]
async fn run_engine() {
    engine_app::run().await;
}

#[tokio::main]
async fn run_storage(_db: Arc<PgEmbed>) {
    storage_app::run().await;
}

#[tokio::main]
async fn run_simulator() {
    simulator_app::run().await;
}

#[tokio::main]
async fn run_interactor() {
    interactor_app::run().await;
}

const APP_NAME: &str = "
███╗░░██╗██╗░░░██╗░█████╗░░█████╗░███╗░░██╗███████╗
████╗░██║██║░░░██║██╔══██╗██╔══██╗████╗░██║██╔════╝
██╔██╗██║██║░░░██║██║░░╚═╝███████║██╔██╗██║█████╗░░
██║╚████║██║░░░██║██║░░██╗██╔══██║██║╚████║██╔══╝░░
██║░╚███║╚██████╔╝╚█████╔╝██║░░██║██║░╚███║███████╗
╚═╝░░╚══╝░╚═════╝░░╚════╝░╚═╝░░╚═╝╚═╝░░╚══╝╚══════╝";
