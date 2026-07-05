use std::env;
use std::sync::{LazyLock};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub proxy: ProxyConfig,
    pub pages: PagesConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct ProxyConfig {
    pub timeout_secs: u64,
    pub allowed_urls: Vec<String>,
}

#[derive(Deserialize)]
pub struct PagesConfig {
    pub path: String,
}

static CONFIG: LazyLock<&'static Config> = LazyLock::new(|| {
    let path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    let config = Config::from_file(&path).expect("failed to load config");
    Box::leak(Box::new(config))
});

impl Config {
    fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn get() -> &'static Config {
        *CONFIG
    }
}
