use std::io;
use std::io::Write;
use std::sync::Arc;

use crate::{error::RouilleError, Client};
use anyhow::Result;
use parking_lot::Mutex;
use rspotify::spotify::oauth2::SpotifyOAuth;
use rspotify::spotify::oauth2::TokenInfo;

#[derive(Debug)]
pub(super) struct OAuth(SpotifyOAuth);

pub(super) fn build(auth: &Client) -> OAuth {
    OAuth(
        SpotifyOAuth::default()
            .client_id(&auth.id)
            .client_secret(&auth.secret)
            .redirect_uri("http://localhost:9524")
            .scope(
                "user-read-playback-state \
                 user-modify-playback-state \
                 user-read-currently-playing \
                 streaming app-remote-control \
                 playlist-read-collaborative \
                 playlist-read-private \
                 user-library-read \
                 user-top-read \
                 user-read-recently-played",
            )
            .build(),
    )
}

impl OAuth {
    pub(super) fn code(&self) -> Result<String> {
        let code = Arc::new(Mutex::new(None));
        let code2 = code.clone();

        let server = rouille::Server::new("localhost:9524", move |request| {
            *code2.lock() = Some(
                request
                    .get_param("code")
                    .expect("Spotify should provide code as a url query parameter"),
            );

            rouille::Response::text("Close me!").with_status_code(404)
        })
        .map_err(Into::<RouilleError>::into)?;

        let auth_url = self.0.get_authorize_url(None, Some(false));
        let open = open::that(&auth_url);

        if open.is_err() {
            writeln!(
                io::stdout(),
                "Open '{}' to authorize with spotify",
                auth_url
            )?;
        }

        loop {
            server.poll();
            {
                if let Some(code) = &*code.lock() {
                    return Ok(code.to_owned());
                }
            }
        }
    }

    pub(super) fn token(&self, code: &str) -> Option<TokenInfo> {
        self.0.get_access_token(code)
    }

    pub(super) fn refresh(&self, token: &TokenInfo) -> Option<TokenInfo> {
        token
            .refresh_token
            .as_ref()
            .and_then(|refresh_token| self.0.refresh_access_token(refresh_token))
    }
}
