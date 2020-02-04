use anyhow::Result;
use env_logger::Builder;
use log::LevelFilter;
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

// TODO
// * improve token aquire code
// * devices

fn main() -> Result<()> {
    let cli = cli::CLI::from_args();

    Builder::from_default_env()
        .format_timestamp(None)
        .filter_level(match cli.verbose {
            0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .init();

    let mut config = config::get();

    if let Some(config) = config.as_mut() {
        cli.run(config)?;
    } else {
        let mut tmp_cfg = Default::default();
        cli.run(&mut tmp_cfg)?;

        anyhow::ensure!(
            !tmp_cfg.is_dirty(),
            "Could not read config but config was changed!"
        );
    }

    config.map(|c| c.write_if_dirty()).unwrap_or(Ok(()))
}

#[macro_export]
macro_rules! log_err {
    ($result:expr) => {{
        match (|| $result)() {
            Ok::<_, ::anyhow::Error>(x) => Some(x),
            Err(e) => {
                ::log::error!("{}", e);
                None
            }
        }
    }};
}
