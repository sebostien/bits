use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::open::Open;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub open: Open,
}

impl Config {
    pub fn from_file<P>(file: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let contents = std::fs::read_to_string(&file)?;

        match toml::from_str(&contents) {
            Ok(c) => Ok(c),
            Err(e) => {
                let file = file.as_ref().to_str().unwrap();
                if let Some(span) = e.span() {
                    Err(anyhow!("{file}:{} {}", span.start, e.message()))
                } else {
                    Err(anyhow!("{file} {}", e.message()))
                }
            }
        }
    }
}
