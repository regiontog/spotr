use anyhow::Result;
use rspotify::spotify::client::Spotify;
use structopt::StructOpt;

mod cli;
mod dialouge;
mod error;
mod keyring;
mod oauth;

struct Client {
    id: String,
    secret: String,
}

impl Client {
    fn new(id: String, secret: String) -> Self {
        Self { id, secret }
    }
}

fn main() -> Result<()> {
    let args = cli::CLI::from_args();

    let client = keyring::get_or_create_client(args.client_id.as_ref().map(|s| s.as_str()))?;
    let auth = oauth::build(&client);

    let mut created_now = false;

    let mut token = keyring::token(&client, || {
        created_now = true;
        let code = auth.code()?;

        Ok(auth
            .token(&code)
            .ok_or(error::ApplicationError::TokenRequestFailed)?)
    })?;

    if !created_now {
        // TODO: Maybe don't refresh if token is very fresh
        if let Some(t) = auth.refresh(&token) {
            token = t;
            keyring::store_token(&client, &token)?;
        }
    }

    args.run(Spotify::default().access_token(&token.access_token).build())
}

fn get<T>(value: std::result::Result<T, failure::Error>) -> Result<T> {
    value.map_err(|f| f.compat().into())
}
