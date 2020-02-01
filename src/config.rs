use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};

use crate::error::ApplicationError;
use crate::log_err;
use anyhow::Result;
use directories::ProjectDirs;
use ring::aead::LessSafeKey;
use serde::{Deserialize, Serialize};
use spotify_web::model::Token;

#[derive(Serialize, Deserialize)]
struct Encrypted<T> {
    #[serde(with = "serde_bytes")]
    nonce: Vec<u8>,

    #[serde(with = "serde_bytes")]
    data: Vec<u8>,

    #[serde(skip)]
    _m: std::marker::PhantomData<T>,
}

impl<'a, T> Encrypted<T>
where
    T: Serialize + Deserialize<'a>,
{
    fn encrypt(value: T, key: &LessSafeKey, nonce: Vec<u8>) -> Result<Self> {
        let mut data = serde_json::to_vec(&value)?;

        key.seal_in_place_append_tag(
            ring::aead::Nonce::try_assume_unique_for_key(&nonce)
                .map_err(Into::<ApplicationError>::into)?,
            ring::aead::Aad::empty(),
            &mut data,
        )
        .map_err(Into::<ApplicationError>::into)?;

        Ok(Self {
            nonce,
            data,
            _m: std::marker::PhantomData,
        })
    }

    fn decrypt(&'a mut self, key: &LessSafeKey) -> Result<T> {
        let data = key.open_in_place(
            ring::aead::Nonce::try_assume_unique_for_key(&self.nonce)
                .map_err(Into::<ApplicationError>::into)?,
            ring::aead::Aad::empty(),
            &mut self.data,
        ).map_err(Into::<ApplicationError>::into)?;

        Ok(serde_json::from_slice(data)?)
    }
}

#[derive(Serialize, Deserialize)]
struct ClientData {
    enc_secret: Encrypted<String>,
    enc_token: Option<Encrypted<Token>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(with = "serde_bytes")]
    nonce: Vec<u8>,

    default: Option<String>,
    clients: HashMap<String, ClientData>,

    #[serde(skip)]
    dirty: bool,

    #[serde(skip)]
    path: std::path::PathBuf,
}

impl Config {
    pub fn set_default(&mut self, id: String) {
        self.dirty = true;

        self.default = Some(id);
    }

    pub fn default(&self) -> Option<&String> {
        self.default.as_ref()
    }

    pub fn clients(&self) -> impl Iterator<Item = &String> {
        self.clients.keys()
    }

    pub fn get_client_data(
        &self,
        id: &str,
        enc_key: &LessSafeKey,
    ) -> Result<(String, Option<Token>)> {
        unimplemented!()
    }

    pub fn set_token(&mut self, id: &str, token: &Token, enc_key: &LessSafeKey) -> Result<()> {
        self.dirty = true;

        unimplemented!()
    }

    pub fn add_client(&mut self, id: String, secret: String, enc_key: &LessSafeKey) -> Result<()> {
        self.dirty = true;

        self.clients.insert(
            id,
            ClientData {
                enc_secret: Encrypted::encrypt(secret, &enc_key, vec![0; 12])?,
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

            let content = toml::to_string(&self)?;
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
