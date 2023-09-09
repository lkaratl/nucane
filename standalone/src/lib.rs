use std::{fs, thread};
use std::process::Command;

use tracing::info;

const LOCAL_DEV_COMPOSE_FILE_NAME: &str = "docker-compose.localdev.yml";

pub fn run() {
    info!("===============================");
    info!("NUCANE services running locally");
    info!("===============================");
    run_capability_providers();
    thread::spawn(|| { run_registry() });
    thread::spawn(|| { run_engine() });
    thread::spawn(|| { run_storage() });
    thread::spawn(|| { run_simulator() });
    thread::spawn(|| { run_interactor() });
}

fn run_capability_providers() {
    fs::write(format!("./{LOCAL_DEV_COMPOSE_FILE_NAME}"), include_str!("../docker-compose.localdev.yml"))
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
async fn run_registry() {
    registry_app::run().await;
}

#[tokio::main]
async fn run_engine() {
    engine_app::run().await;
}

#[tokio::main]
async fn run_storage() {
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
