use anyhow::{anyhow, Result};
use spotify_web::Spotify;
use structopt::StructOpt;

use crate::config::Config;
use crate::Scope;

#[derive(StructOpt)]
#[structopt(
    rename_all = "kebab-case",
    about = env!("CARGO_PKG_DESCRIPTION"),
    author = env!("CARGO_PKG_AUTHORS"),
)]
pub(super) struct CLI {
    /// Client id of the spotify application to use
    #[structopt(long, short = "i")]
    pub(super) client_id: Option<String>,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    Client {
        #[structopt(subcommand)]
        cmd: Client
    },

    #[structopt(alias = "s")]
    Status(Status),

    Play(Play),
    Pause(Pause),
}

/// Edit available clients
#[derive(StructOpt)]
enum Client {
    New(ClientNew),
    List(ClientList),
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
    pub(super) fn run(self, config: &mut Config) -> Result<()> {
        self.cmd.run(&self, config)
    }

    fn spotify(&self, config: &mut Config) -> Result<Spotify<Scope>> {
        let enc_key = crate::keyring::get_or_create_key()?;

        let id = self
            .client_id
            .clone()
            .or_else(|| config.default().cloned())
            .ok_or(anyhow!("Client id required!"))?;

        let (secret, token) = config.get_client_data(&id, &enc_key)?;

        let client = spotify_web::Client::new(&id, &secret, Scope::create());

        let auth = client
            .authorization()
            .redirect_uri("http://localhost:9524")
            .build();

        // TODO: Maybe don't refresh if token is very fresh
        if let Some(token) = token
            .and_then(|token| auth.refresh_token(&token))
            .transpose()?
        {
            config.set_token(&id, &token, &enc_key)?;

            Ok(client.with_access_token(&token)?)
        } else {
            let code = crate::oauth::code(auth.url().as_str())?;
            let token = auth.fetch_token2(code.as_str(), None)?;

            config.set_token(&id, &token, &enc_key)?;

            Ok(client.with_access_token(&token)?)
        }
    }
}

impl Command {
    fn run(&self, cli: &CLI, config: &mut Config) -> Result<()> {
        match self {
            Self::Status(x) => x.run(cli, config),
            Self::Play(x) => x.run(cli, config),
            Self::Pause(x) => x.run(cli, config),
            Self::Client { cmd } => cmd.run(cli, config),
        }
    }
}

impl Client {
    fn run(&self, cli: &CLI, config: &mut Config) -> Result<()> {
        match self {
            Self::New(x) => x.run(cli, config),
            Self::List(x) => x.run(cli, config),
        }
    }
}

impl ClientList {
    fn run(&self, cli: &CLI, config: &mut Config) -> Result<()> {
        for client in config.clients().keys() {
            crate::dialouge::display(client)?;
        }

        Ok(())
    }
}

impl ClientNew {
    fn run(&self, cli: &CLI, config: &mut Config) -> Result<()> {
        unimplemented!()
    }
}

impl Status {
    fn run(&self, cli: &CLI, config: &mut Config) -> Result<()> {
        let output = cli.spotify(config)?.currently_playing(None)?.text()?;
        crate::dialouge::display(&output)
    }
}

impl Play {
    fn run(&self, cli: &CLI, config: &mut Config) -> Result<()> {
        let output = cli.spotify(config)?.resume_playback(None)?.text()?;
        crate::dialouge::display(&output)
    }
}

impl Pause {
    fn run(&self, cli: &CLI, config: &mut Config) -> Result<()> {
        let output = cli.spotify(config)?.pause_playback(None)?.text()?;
        crate::dialouge::display(&output)
    }
}
