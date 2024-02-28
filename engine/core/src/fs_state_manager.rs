use std::{env, fs};

use dashmap::DashMap;
use tracing::warn;

pub struct FsStateManager {
    state: DashMap<String, String>,
}

const STATE_FOLDER_PATH: &str = "nucane/state";
const STATE_FILE_NAME: &str = "state.json";

impl Default for FsStateManager {
    fn default() -> Self {
        Self {
            state: load_state()
        }
    }
}

fn load_state() -> DashMap<String, String> {
    let mut state_path = env::temp_dir();
    state_path.push(format!("{STATE_FOLDER_PATH}{STATE_FILE_NAME}"));
    match fs::read_to_string(state_path) {
        Ok(state) => serde_json::from_str(&state).unwrap(),
        Err(error) => {
            warn!("Failed to load state file: '{error}'. Plugins state will be empty");
            DashMap::new()
        }
    }
}

impl FsStateManager {
    pub fn set(&self, key: &str, state: String) {
        let previous_state = self.state.insert(key.to_string(), state.clone());
        if let Some(previous_state) = previous_state {
            if state != previous_state {
                self.backup();
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.state.get(key).map(|value| value.value().clone())
    }

    fn backup(&self) {
        let content = serde_json::to_string_pretty(&self.state).unwrap();

        let mut state_path = env::temp_dir();
        state_path.push(STATE_FOLDER_PATH);
        fs::create_dir_all(&state_path).unwrap();
        state_path.push(STATE_FILE_NAME);
        fs::write(state_path, content).unwrap();
    }
}
