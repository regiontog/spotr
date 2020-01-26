use std::io;
use std::io::Write;

use crate::Client;
use anyhow::Result;

pub fn confirm(prompt: &str) -> Result<bool> {
    write!(io::stdout(), ":: {}? [Y/n] ", prompt)?;
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim();

    Ok(input == "" || input == "y" || input == "Y")
}

pub(super) fn new_client() -> Result<Client> {
    // FIXME: Some spiel about settings the redirect whitelist
    writeln!(
        io::stdout(),
        "To use this CLI application you need to register an application with spotify. \
        You can register an application at 'https://developer.spotify.com/dashboard/applications'. \
        It does not matter what you choose for name, description or application type. \
        When you have created the application click edit settings and add \
        'http://localhost:9524' to the redirect whitelist."
    )?;

    write!(io::stdout(), ":: Client id? ")?;

    io::stdout().flush()?;

    let mut id = String::new();
    io::stdin().read_line(&mut id)?;

    id = id.trim().to_owned();

    let secret = rpassword::read_password_from_tty(Some(":: Client secret? "))?
        .trim()
        .to_owned();

    Ok(Client::new(id, secret))
}

pub(super) fn secret_for(id: &str) -> Result<String> {
    writeln!(
        io::stdout(),
        "No client secret found for client id: '{id}'.\
         If you have already created this spotify application \
         the secret should be availiable from \
         'https://developer.spotify.com/dashboard/applications/{id}'. \
         See https://regiontog.github.io/spotr for documentation on \
         how to change client id.",
        id = id
    )?;

    Ok(rpassword::read_password_from_tty(Some("Client secret: "))?
        .trim()
        .to_owned())
}

pub(super) fn set_default(_client: &Client) -> Result<bool> {
    confirm("Set new client as default")
}

pub(super) fn display<T: serde::ser::Serialize>(value: T) -> Result<()> {
    Ok(writeln!(
        io::stdout(),
        "{}",
        serde_json::to_string_pretty(&value)?
    )?)
}
