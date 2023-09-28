use std::collections::HashMap;

use config::{Environment, File, FileFormat};
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub logging: Logging,
    pub application: Application,
    pub storage: Storage,
    pub engine: Engine,
    pub eac: EAC,
}

#[derive(Deserialize)]
pub struct EAC {
    pub demo: bool,
    pub exchanges: Exchanges,
}

#[derive(Deserialize)]
pub struct Exchanges {
    pub okx: OKX,
}

#[derive(Deserialize)]
pub struct OKX {
    pub http: OKXHttp,
    pub ws: OKXWs,
    pub auth: OKXAuth,
}

#[derive(Deserialize, Clone)]
pub struct OKXHttp {
    pub url: String
}

#[derive(Deserialize, Clone)]
pub struct OKXWs {
    pub url: String
}

#[derive(Deserialize, Clone)]
pub struct OKXAuth {
    pub key: String,
    pub secret: String,
    pub passphrase: String,
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

#[derive(Deserialize)]
pub struct Storage {
    pub url: String
}

#[derive(Deserialize)]
pub struct Engine {
    pub url: String
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
            .add_source(Environment::with_prefix("INTERACTOR")
                .try_parsing(true)
                .separator("_"))
            .build()
            .expect("Error during config creation")
            .try_deserialize()
            .expect("Error during config deserialization")
    }
}
