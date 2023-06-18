use std::{fs, thread};
use std::process::Command;

use tracing::info;

pub fn run() {
    info!("===============================");
    info!("NUCANE services running locally");
    info!("===============================");
    run_database();
    run_massage_broker();
    thread::spawn(|| { run_registry() });
    thread::spawn(|| { run_strategy_engine() });
    thread::spawn(|| { run_storage() });
    thread::spawn(|| { run_simulator() });
    thread::spawn(|| { run_interactor() });
}

// todo throw error if run failed
fn run_database() {
    fs::write("./docker-compose.yml", include_str!("../mysql/docker-compose.yml"))
        .expect("Error during docker compose file creation");
    Command::new("docker")
        .args(["compose", "up", "-d"])
        .output()
        .expect("Error during database running");
    fs::remove_file("./docker-compose.yml")
        .expect("Error during docker compose file removing");
    info!("+ database running...");
}

// todo throw error if run failed
fn run_massage_broker() {
    fs::write("./docker-compose.yml", include_str!("../redpanda/docker-compose.yml"))
        .expect("Error during docker compose file creation");
    Command::new("docker")
        .args(["compose", "up", "-d"])
        .output()
        .expect("Error during message broker running");
    fs::remove_file("./docker-compose.yml")
        .expect("Error during docker compose file removing");
    info!("+ massage-broker running...");
}

#[tokio::main]
async fn run_registry() {
    registry_app::run().await;
}

#[tokio::main]
async fn run_strategy_engine() {
    strategy_engine::run().await;
}

#[tokio::main]
async fn run_storage() {
    storage::run().await;
}

#[tokio::main]
async fn run_simulator() {
    simulator::run().await;
}

#[tokio::main]
async fn run_interactor() {
    interactor::run().await;
}
