use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};

use crate::error::ApplicationError;
use crate::log_err;
use anyhow::Result;
use directories::ProjectDirs;
use ring::aead::{Aad, LessSafeKey, Nonce};
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

struct ConfigSealingKey<'a> {
    key: &'a LessSafeKey,
    cfg: &'a mut Config,
}

fn advance_vec(bytes: &mut [u8]) -> Result<(), ApplicationError> {
    if bytes.iter().all(|&b| b == 0xFF) {
        return Err(ApplicationError::CryptographyError);
    }

    for byte in bytes.iter_mut().rev() {
        if *byte == 0xFF {
            *byte = 0;
        } else {
            *byte += 1;
            break;
        }
    }

    Ok(())
}

impl<'a> ConfigSealingKey<'a> {
    fn new(key: &'a LessSafeKey, cfg: &'a mut Config) -> Self {
        Self { key, cfg }
    }

    fn advance_nonce(&mut self) -> Result<Vec<u8>, ApplicationError> {
        self.cfg.dirty = true;
        advance_vec(&mut self.cfg.nonce)?;

        Ok(self.cfg.nonce.clone())
    }

    pub fn seal_in_place_append_tag<A>(
        &mut self,
        aad: Aad<A>,
        in_out: &mut Vec<u8>,
    ) -> Result<Vec<u8>, ApplicationError>
    where
        A: AsRef<[u8]>,
    {
        let nonce = self.advance_nonce()?;

        self.key.seal_in_place_append_tag(
            Nonce::try_assume_unique_for_key(&nonce)?,
            aad,
            in_out,
        )?;

        Ok(nonce)
    }
}

impl<T> Encrypted<T> {
    fn encrypt(value: &T, key: &mut ConfigSealingKey) -> Result<Self>
    where
        T: Serialize,
    {
        let mut data = serde_json::to_vec(value)?;

        let nonce = key.seal_in_place_append_tag(ring::aead::Aad::empty(), &mut data)?;

        Ok(Self {
            nonce,
            data,
            _m: std::marker::PhantomData,
        })
    }

    fn decrypt(&self, key: &LessSafeKey) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut data = self.data.to_owned();
        let data = key
            .open_in_place(
                ring::aead::Nonce::try_assume_unique_for_key(&self.nonce)
                    .map_err(Into::<ApplicationError>::into)?,
                ring::aead::Aad::empty(),
                &mut data,
            )
            .map_err(Into::<ApplicationError>::into)?;

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
    pub fn set_default(&mut self, id: String) -> Result<()> {
        anyhow::ensure!(self.clients.contains_key(&id), "Cannot set default client to non-existing client");

        self.dirty = true;

        self.default = Some(id);
        Ok(())
    }

    pub fn default(&self) -> Option<&String> {
        self.default.as_ref()
    }

    pub fn clients(&self) -> impl Iterator<Item = (&String, bool)> {
        self.clients
            .iter()
            .map(|(id, data)| (id, data.enc_token.is_some()))
    }

    pub fn get_client_data(
        &self,
        id: &str,
        enc_key: &LessSafeKey,
    ) -> Option<Result<(String, Option<Token>)>> {
        let client = self.clients.get(id);

        client.map(|client| {
            let secret = client.enc_secret.decrypt(enc_key)?;
            let token = client
                .enc_token
                .as_ref()
                .map(|enc| enc.decrypt(enc_key))
                .transpose()?;

            Ok((secret, token))
        })
    }

    pub fn set_token(&mut self, id: &str, token: &Token, enc_key: &LessSafeKey) -> Result<()> {
        self.dirty = true;

        if self.clients.contains_key(id) {
            let enc_token = Encrypted::encrypt(token, &mut ConfigSealingKey::new(enc_key, self))?;

            self.clients.get_mut(id).expect("is_some").enc_token = Some(enc_token);
        } else {
            log::warn!("Attempting to set token on non-existing client id");
        }

        Ok(())
    }

    pub fn eject_token(&mut self, id: &str) {
        self.dirty = true;

        if let Some(data) = self.clients.get_mut(id) {
            data.enc_token = None;
        } else {
            log::warn!("Attempting to eject token on non-existing client id");
        }
    }

    pub fn remove_client(&mut self, id: &str) {
        self.dirty = true;

        self.clients.remove(id);
    }

    pub fn add_client(&mut self, id: String, secret: String, enc_key: &LessSafeKey) -> Result<()> {
        self.dirty = true;

        let enc_secret = Encrypted::encrypt(&secret, &mut ConfigSealingKey::new(enc_key, self))?;

        self.clients.insert(
            id,
            ClientData {
                enc_secret,
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

            serde_json::to_writer(std::io::BufWriter::new(&mut file), &self)?;

            let position = file.seek(SeekFrom::Current(0))?;
            file.set_len(position)?;
        }

        Ok(())
    }
}

pub fn get() -> Option<Config> {
    log_err!({
        log::trace!("reading config");

        let dirs = ProjectDirs::from("rs", "regiontog", "spotr")
            .ok_or(ApplicationError::UnavailableConfigDir)?;

        let mut path = dirs.data_dir().to_owned();

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

            serde_json::from_str(&content)?
        } else {
            log::info!("empty config file, using default");
            Default::default()
        };

        config.path = path;

        if config.nonce.is_empty() {
            config.nonce.extend(&[0; ring::aead::NONCE_LEN]);
        }

        assert_eq!(ring::aead::NONCE_LEN, config.nonce.len());

        Ok(config)
    })
}
