use anyhow::Result;
use keyring::Keyring;
use rspotify::spotify::oauth2::TokenInfo;

use crate::dialouge;
use crate::error::SyncError;
use crate::Client;

fn secret_key(id: &str) -> Keyring {
    Keyring::new("spotr", id)
}

fn token_key(id: &str) -> Keyring {
    Keyring::new("spotr-token", id)
}

pub(super) fn token(
    client: &Client,
    create_token: impl FnOnce() -> Result<TokenInfo>,
) -> Result<TokenInfo> {
    let key = token_key(&client.id);

    match key.get_password() {
        Err(keyring::KeyringError::NoPasswordFound) => {
            let token = create_token()?;

            key.set_password(&serde_json::to_string(&token)?)
                .map_err(SyncError::new)?;
            Ok(token)
        }
        Ok(token) => Ok(serde_json::from_str(&token)?),
        Err(e) => Err(SyncError::new(e).into()),
    }
}

pub(super) fn store_token(client: &Client, token: &TokenInfo) -> Result<()> {
    let key = token_key(&client.id);

    key.set_password(&serde_json::to_string(token)?)
        .map_err(SyncError::new)?;

    Ok(())
}

pub(super) fn get_or_create_client<'a>(id: Option<&'a str>) -> Result<Client> {
    if let Some(id) = id {
        let key = secret_key(id);

        match key.get_password() {
            Err(keyring::KeyringError::NoPasswordFound) => {
                let secret = dialouge::secret_for(id)?;

                key.set_password(&secret).map_err(|e| SyncError::new(e))?;

                Ok(Client::new(id.to_owned(), secret))
            }
            Ok(secret) => Ok(Client::new(id.to_owned(), secret)),
            Err(e) => Err(SyncError::new(e).into()),
        }
    } else {
        let client = dialouge::new_client()?;

        secret_key(&client.id)
            .set_password(&client.secret)
            .map_err(|e| SyncError::new(e))?;

        if dialouge::set_default(&client)? {
            // TODO: somehow store default id
            println!("Storing as default");
        }

        Ok(client)
    }
}
