use anyhow::Result;
use rspotify::spotify::client::Spotify;
use structopt::StructOpt;

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
    #[structopt(alias = "s")]
    Status(Status),
    Play(Play),
    Pause(Pause),
}

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
    pub(super) fn run(self, spotify: Spotify) -> Result<()> {
        self.cmd.run(&self, spotify)
    }
}

trait Run {
    fn run(&self, cli: &CLI, spotify: Spotify) -> Result<()>;
}

impl Run for Command {
    fn run(&self, cli: &CLI, spotify: Spotify) -> Result<()> {
        match self {
            Self::Status(x) => x.run(cli, spotify),
            Self::Play(x) => x.run(cli, spotify),
            Self::Pause(x) => x.run(cli, spotify),
        }
    }
}

impl Run for Status {
    fn run(&self, _: &CLI, spotify: Spotify) -> Result<()> {
        crate::dialouge::display(crate::get(spotify.current_playing(None))?)
    }
}

impl Run for Play {
    fn run(&self, _: &CLI, spotify: Spotify) -> Result<()> {
        crate::get(spotify.start_playback(None, None, None, None, None))
    }
}

impl Run for Pause {
    fn run(&self, _: &CLI, spotify: Spotify) -> Result<()> {
        crate::get(spotify.pause_playback(None))
    }
}
