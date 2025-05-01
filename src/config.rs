use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{branches::Branches, open::Open};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub open: Open,
    #[serde(default)]
    pub branches: Branches,
}

impl Config {
    pub fn new(config_file: Option<PathBuf>) -> Result<Self> {
        let config_file = Self::get_config_file(config_file)?;
        let contents = std::fs::read_to_string(&config_file)?;

        match toml::from_str(&contents) {
            Ok(c) => Ok(c),
            Err(e) => {
                let file = config_file.to_str().unwrap_or("config");
                Err(anyhow!("{file} :: {}", e.message()))
            }
        }
    }

    fn get_config_file(config_file: Option<PathBuf>) -> Result<PathBuf> where {
        if let Some(c) = config_file {
            return Ok(c);
        }

        if let Some(mut home) = dirs::config_dir() {
            home.push("bits/config.toml");
            if home.exists() {
                return Ok(home);
            }
        }

        Err(anyhow!(
            "Could not find config. Please use the `--config-file` option"
        ))
    }
}
