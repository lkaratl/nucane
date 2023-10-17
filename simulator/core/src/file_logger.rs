use std::{env, fs};

use tracing::info;
use uuid::Uuid;

pub struct Logger {
    simulation_id: Uuid,
    file_content: Vec<String>,
}

impl Logger {
    pub fn new(simulation_id: Uuid) -> Self {
        Self {
            simulation_id,
            file_content: Vec::new(),
        }
    }
    pub fn log(&mut self, message: String) {
        info!(message);
        self.file_content.push(message);
    }

    pub fn save(&self) {
        let mut tmp_dir = env::temp_dir();
        tmp_dir.push("nucane/logs");
        fs::create_dir_all(&tmp_dir).unwrap();
        tmp_dir.push(format!("simulation-{}.log", self.simulation_id));
        fs::write(tmp_dir, self.file_content.join("\n"))
            .expect("Error during saving simulation log");
    }
}
