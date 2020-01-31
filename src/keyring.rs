use anyhow::Result;
use keyring::Keyring;
use ring::rand::SecureRandom;

use crate::dialouge;
use crate::error::{SyncError, ApplicationError};

fn secret_key() -> Keyring<'static> {
    Keyring::new("spotr", "")
}

fn new_secret(key: &Keyring) -> Result<Vec<u8>> {
        let mut secret = vec![0; crate::CRYPT_ALGO.key_len()];
        ring::rand::SystemRandom::new().fill(&mut secret).map_err(Into::<ApplicationError>::into)?;

        let mut b64 = String::new();
        base64::encode_config_buf(&secret, base64::STANDARD_NO_PAD, &mut b64);

        key.set_password(&b64).map_err(|e| SyncError::new(e))?;

        Ok(secret)
}

pub(super) fn get_or_create_key() -> Result<Vec<u8>> {
    let key = secret_key();

    match key.get_password() {
        Err(keyring::KeyringError::NoPasswordFound) => {
            new_secret(&key)
        }
        Ok(b64) => {
            let mut secret = vec![0; crate::CRYPT_ALGO.key_len()];

            let written = base64::decode_config_slice(&b64, base64::STANDARD_NO_PAD, &mut secret)?;

            if written != secret.len() {
                // Invalid crypto key

                secret = new_secret(&key)?;
            }

            Ok(secret)
        }
        Err(e) => Err(SyncError::new(e).into()),
    }
}
