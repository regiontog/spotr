use anyhow::Result;
use spotify_web::scope::*;
use structopt::StructOpt;

mod cli;
mod config;
mod dialouge;
mod error;
mod keyring;
mod oauth;

type Scope = spotify_web::scopes![UserReadCurrentlyPlaying, UserModifyPlaybackState];

static CRYPT_ALGO: &ring::aead::Algorithm = &ring::aead::AES_256_GCM;

fn main() -> Result<()> {
    let mut config = config::get().ok();
    let cli = cli::CLI::from_args();

    if let Some(config) = config.as_mut() {
        cli.run(config)?;
    } else {
        cli.run(&mut Default::default())?;
    }

    config.map(|c| c.write_if_dirty()).unwrap_or(Ok(()))
}
