use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub endpoint: String,
    pub token: Option<String>,
    pub model: String,
    pub webhook: String,
    pub context_size: usize,
    pub system_prompt: String,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path).context("Failed to load the config")?;
        let config = toml::from_str(&text).context("Failed to parse the config")?;
        Ok(config)
    }
}
