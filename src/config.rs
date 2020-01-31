use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};

use crate::error::ApplicationError;
use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use spotify_web::model::Token;

#[derive(Serialize, Deserialize)]
pub struct ClientData {
    pub enc_secret: String,
    pub enc_token: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    default: Option<String>,
    clients: HashMap<String, ClientData>,

    #[serde(skip)]
    dirty: bool,

    #[serde(skip)]
    path: std::path::PathBuf,
}

impl Config {
    pub fn default(&self) -> Option<&String> {
        self.default.as_ref()
    }

    pub fn clients(&self) -> &HashMap<String, ClientData> {
        &self.clients
    }

    pub fn get_client_data(&self, id: &str, enc_key: &[u8]) -> Result<(String, Option<Token>)> {
        unimplemented!()
    }

    pub fn set_token(&mut self, id: &str, token: &Token, enc_key: &[u8]) -> Result<()> {
        self.dirty = true;

        unimplemented!()
    }

    pub fn write_if_dirty(self) -> anyhow::Result<()> {
        if self.dirty {
            let mut file = OpenOptions::new().write(true).open(&self.path)?;

            let content = toml::to_string_pretty(&self)?;
            file.write_all(content.as_bytes())?;
        }

        Ok(())
    }
}

pub fn get() -> anyhow::Result<Config> {
    let dirs = ProjectDirs::from("rs", "regiontog", "spotr")
        .ok_or(ApplicationError::UnavailableConfigDir)?;

    let mut path = dirs.data_dir().to_owned();
    path.push("config.toml");

    let mut file = OpenOptions::new().write(true).create(true).open(&path)?;
    let mut content =
        String::with_capacity(file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0));

    file.read_to_string(&mut content)?;

    let mut config: Config = toml::from_str(&content)?;

    config.path = path;

    Ok(config)
}
