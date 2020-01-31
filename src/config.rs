use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};

use crate::error::ApplicationError;
use crate::log_err;
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

    pub fn add_client(&mut self, id: String, secret: String, enc_key: &[u8]) -> Result<()> {
        self.dirty = true;

        self.clients.insert(
            id,
            ClientData {
                //FIXME: encrypt
                enc_secret: secret,
                enc_token: None,
            },
        );

        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
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

pub fn get() -> Option<Config> {
    log_err!({
        let dirs = ProjectDirs::from("rs", "regiontog", "spotr")
            .ok_or(ApplicationError::UnavailableConfigDir)?;

        let mut path = dirs.data_dir().to_owned();
        log::trace!("config dir: {:#?}", &path);

        std::fs::create_dir_all(&path)?;
        path.push("config.toml");

        log::debug!("config path: {:#?}", &path);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let len = file.metadata()?.len();

        let mut config: Config = if len > 0 {
            let mut content = String::with_capacity(len as usize + 1);
            file.read_to_string(&mut content)?;

            toml::from_str(&content)?
        } else {
            Default::default()
        };

        config.path = path;

        Ok(config)
    })
}
