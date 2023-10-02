use std::collections::HashMap;

use config::{Environment, File, FileFormat};
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub logging: Logging,
    pub application: Application,
    pub database: Database,
    pub interactor: Interactor,
}

#[derive(Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Deserialize)]
pub struct Interactor {
    pub url: String,
}

#[derive(Deserialize)]
pub struct Application {
    pub name: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct Logging {
    level: String,
    crates: HashMap<String, String>,
}

impl Logging {
    pub fn levels(&self) -> String {
        let crate_levels = self.crates.iter().map(|(lib, loglevel)| format!("{lib}={loglevel}"))
            .collect::<Vec<_>>()
            .join(",");
        format!("{},{crate_levels}", self.level)
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);

impl Config {
    fn load() -> Self {
        config::Config::builder()
            .add_source(File::from_str(include_str!("../config.yml"), FileFormat::Yaml))
            .add_source(Environment::with_prefix("APP")
                .try_parsing(true)
                .separator("_"))
            .add_source(Environment::with_prefix("STORAGE")
                .try_parsing(true)
                .separator("_"))
            .build()
            .expect("Error during config creation")
            .try_deserialize()
            .expect("Error during config deserialization")
    }
}
