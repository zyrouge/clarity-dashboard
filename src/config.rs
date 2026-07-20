use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::LazyLock;

use axum::http::Method;
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
    #[serde(deserialize_with = "deserialize_allowed_urls")]
    pub allowed_urls: HashMap<String, ProxyAllowedUrlConfig>,
}

#[derive(Deserialize)]
pub struct ProxyAllowedUrlConfig {
    #[serde(deserialize_with = "deserialize_allowed_url_methods")]
    pub methods: HashSet<Method>,
    pub url: String,
    #[serde(default = "default_proxy_allowed_url_timeout")]
    pub timeout_secs: u64,
}

#[derive(Deserialize)]
pub struct PagesConfig {
    pub path: String,
}

static CONFIG: LazyLock<&'static Config> = LazyLock::new(|| {
    let path = env::var("CONFIG_FILE").unwrap_or_else(|_| "config.toml".to_string());
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

fn default_proxy_allowed_url_timeout() -> u64 {
    30
}

fn deserialize_allowed_urls<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, ProxyAllowedUrlConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map: Vec<ProxyAllowedUrlConfig> = Vec::deserialize(deserializer)?;
    let mut result = HashMap::new();
    for item in map {
        result.insert(item.url.clone(), item);
    }
    Ok(result)
}

fn deserialize_allowed_url_methods<'de, D>(deserializer: D) -> Result<HashSet<Method>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let methods: Vec<String> = Vec::deserialize(deserializer)?;
    let mut result = HashSet::new();
    for method_str in methods {
        let method = method_str.parse::<Method>().map_err(|_| {
            serde::de::Error::custom(format!("invalid HTTP method: {}", method_str))
        })?;
        result.insert(method);
    }
    Ok(result)
}
