use std::{fs, thread};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_fetch::{PgFetchSettings, PostgresVersion};
use pg_embed::postgres::{PgEmbed, PgSettings};

use tracing::info;
use standalone_config::CONFIG;

const LOCAL_DEV_COMPOSE_FILE_NAME: &str = "docker-compose.localdev.yml";

pub fn run() {
    info!("===============================");
    info!("NUCANE services running locally");
    info!("===============================");
    run_capability_providers();
    let db = Arc::new(run_db());
    thread::spawn(|| { run_registry() });
    thread::spawn(|| { run_engine() });
    thread::spawn({
       let db = Arc::clone(&db);
        || { run_storage(db) }
    });
    thread::spawn(|| { run_simulator() });
    thread::spawn(|| { run_interactor() });
}

fn run_capability_providers() {
    fs::write(format!("./{LOCAL_DEV_COMPOSE_FILE_NAME}"), include_str!("../../docker-compose.localdev.yml"))
        .expect("Error during docker compose file creation");
    Command::new("docker")
        .args(["compose", "-f", LOCAL_DEV_COMPOSE_FILE_NAME, "up", "-d"])
        .output()
        .expect("Error during database running");
    fs::remove_file(format!("./{LOCAL_DEV_COMPOSE_FILE_NAME}"))
        .expect("Error during docker compose file removing");
    info!("+ capability providers running...");
    info!("  |- database running...");
    info!("  L massage-broker running...");
}

#[tokio::main]
async fn run_db() -> PgEmbed {
    info!("+ data base running...");
    let pg_settings = PgSettings {
        database_dir: PathBuf::from("postgres"),
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
    pg.setup()
        .await
        .expect("Error during db setup");
    pg.start_db()
        .await
        .expect("Error during db start");
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
