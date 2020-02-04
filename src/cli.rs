use std::io::Write;

use anyhow::{anyhow, Result};
use spotify_web::Spotify;
use structopt::StructOpt;

use crate::config::Config;
use crate::{Scope, Token};

struct LazySpotify {
    generator: fn(Option<String>, &mut Config) -> Result<Spotify<Scope>>,
    cell: Option<std::result::Result<Spotify<Scope>, crate::error::ArcAnyhowError>>,
    client_id: Option<String>,
}

impl LazySpotify {
    fn as_mut<'a>(&mut self, cfg: &'a mut Config) -> Result<&mut Spotify<Scope>, anyhow::Error> {
        let id = &mut self.client_id;
        let generator = self.generator;

        self.cell
            .get_or_insert_with(|| {
                (generator)(id.take(), cfg).map_err(crate::error::ArcAnyhowError::new)
            })
            .as_mut()
            .map_err(Into::into)
    }
}

#[derive(StructOpt)]
#[structopt(
    rename_all = "kebab-case",
    about = env!("CARGO_PKG_DESCRIPTION"),
    author = env!("CARGO_PKG_AUTHORS"),
)]
pub struct CLI {
    /// Client id of the spotify application to use
    #[structopt(long, short = "i")]
    pub client_id: Option<String>,

    /// Verbosity of logging, repeated occurrences count as higher log levels
    #[structopt(
        name = "verbose",
        long = "verbose",
        short = "v",
        parse(from_occurrences)
    )]
    pub verbose: u8,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(alias = "c")]
    Client {
        #[structopt(subcommand)]
        cmd: Client,
    },

    #[structopt(alias = "s")]
    Status(Status),

    Play(Play),
    Pause(Pause),
}

/// Edit available clients
#[derive(StructOpt)]
enum Client {
    #[structopt(alias = "n")]
    New(ClientNew),

    #[structopt(alias = "l")]
    List(ClientList),

    #[structopt(alias = "rm")]
    Remove(ClientRemove),

    #[structopt(alias = "e")]
    Eject(ClientEject),

    #[structopt(alias = "d")]
    Default(ClientDefault),
}

/// Eject a client's token
#[derive(StructOpt)]
struct ClientEject {
    /// Target clients to remove
    ids: Vec<String>,
}

/// Set default client
#[derive(StructOpt)]
struct ClientDefault {
    /// Id of new default client
    id: String,
}

/// Remove a client
#[derive(StructOpt)]
struct ClientRemove {
    /// Target clients
    ids: Vec<String>,
}

/// Add new client
#[derive(StructOpt)]
struct ClientNew {}

/// List all existing clients
#[derive(StructOpt)]
struct ClientList {}

/// Gets metadata about the currently playing song
#[derive(StructOpt)]
struct Status {}

/// Starts or resumes playback
#[derive(StructOpt)]
struct Play {}

/// Pauses playback
#[derive(StructOpt)]
struct Pause {}

impl CLI {
    pub fn run(self, config: &mut Config) -> Result<()> {
        let spotify = LazySpotify {
            client_id: self.client_id,
            generator: CLI::gen_spotify,
            cell: None,
        };

        self.cmd.run(spotify, config)
    }

    fn gen_spotify(client_id: Option<String>, config: &mut Config) -> Result<Spotify<Scope>> {
        let enc_key = crate::keyring::get_or_create_key()?;

        let id = client_id
            .or_else(|| config.default().cloned())
            .ok_or(anyhow!("Client id required!"))?;

        log::trace!("building spotify client using id = '{}'", &id);

        let (secret, token) = config
            .get_client_data(&id, &enc_key)
            .ok_or(anyhow!("No client with id = '{}'", id))??;

        let client = spotify_web::Client::new(&id, &secret, Scope::create());

        let auth = client
            .authorization()
            .redirect_uri("http://localhost:9524")
            .build();

        if let Some(token) = token {
            if token.has_expired() {
                log::debug!("token expired, refreshing");

                let token = Token::new(auth.refresh_token(token.token)?);
                config.set_token(&id, &token, &enc_key)?;

                Ok(client.with_access_token(&token.token)?)
            } else {
                log::debug!("previous token has not expired yet, reusing it");

                Ok(client.with_access_token(&token.token)?)
            }
        } else {
            log::info!("no token, fetching...");
            let code = crate::oauth::code(auth.url().as_str())?;
            let token = Token::new(auth.fetch_token2(code.as_str(), None)?);

            config.set_token(&id, &token, &enc_key)?;

            Ok(client.with_access_token(&token.token)?)
        }
    }
}

impl Command {
    fn run(self, spotify: LazySpotify, config: &mut Config) -> Result<()> {
        match self {
            Self::Status(x) => x.run(spotify, config),
            Self::Play(x) => x.run(spotify, config),
            Self::Pause(x) => x.run(spotify, config),
            Self::Client { cmd } => cmd.run(config),
        }
    }
}

impl Client {
    fn run(self, config: &mut Config) -> Result<()> {
        match self {
            Self::New(x) => x.run(config),
            Self::List(x) => x.run(config),
            Self::Remove(x) => x.run(config),
            Self::Eject(x) => x.run(config),
            Self::Default(x) => x.run(config),
        }
    }
}

impl ClientDefault {
    fn run(self, config: &mut Config) -> Result<()> {
        config.set_default(self.id).map_err(|id| {
            anyhow::anyhow!("Could not set default client to non-existing id = '{}'", id)
        })?;

        Ok(())
    }
}

impl ClientEject {
    fn run(&self, config: &mut Config) -> Result<()> {
        for id in &self.ids {
            config.eject_token(id);
        }

        Ok(())
    }
}

impl ClientList {
    fn run(&self, config: &mut Config) -> Result<()> {
        let default = config.default();

        for (client, token_is_some) in config.clients() {
            writeln!(
                std::io::stdout(),
                "{:<33}{}{}",
                client,
                if token_is_some { "[token]" } else { "       " },
                if default == Some(client) {
                    "[default]"
                } else {
                    ""
                },
            )?;
        }

        Ok(())
    }
}

impl ClientRemove {
    fn run(&self, config: &mut Config) -> Result<()> {
        for id in &self.ids {
            config.remove_client(id);
        }

        Ok(())
    }
}

impl ClientNew {
    fn run(&self, config: &mut Config) -> Result<()> {
        let enc_key = crate::keyring::get_or_create_key()?;

        let (id, secret) = crate::dialouge::new_client()?;

        if crate::dialouge::set_default()? {
            config.set_default_force(&id);
        }

        config.add_client(id.clone(), secret, &enc_key)?;

        Ok(())
    }
}

impl Status {
    fn run(&self, mut spotify: LazySpotify, config: &mut Config) -> Result<()> {
        let output = spotify.as_mut(config)?.currently_playing(None)?.text()?;
        crate::dialouge::display(&output)
    }
}

impl Play {
    fn run(&self, mut spotify: LazySpotify, config: &mut Config) -> Result<()> {
        let output = spotify.as_mut(config)?.resume_playback(None)?.text()?;
        crate::dialouge::display(&output)
    }
}

impl Pause {
    fn run(&self, mut spotify: LazySpotify, config: &mut Config) -> Result<()> {
        let output = spotify.as_mut(config)?.pause_playback(None)?.text()?;
        crate::dialouge::display(&output)
    }
}
