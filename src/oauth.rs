use std::io;
use std::io::Write;
use std::sync::Arc;

use crate::error::RouilleError;
use anyhow::Result;
use parking_lot::Mutex;

pub(super) fn code(url: &str) -> Result<String> {
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

    let open = open::that(&url);

    if open.is_err() {
        writeln!(io::stdout(), "Open '{}' to authorize with spotify", url)?;
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
