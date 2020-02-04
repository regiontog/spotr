use std::io;
use std::io::Write;

use anyhow::Result;

pub fn confirm(prompt: &str) -> Result<bool> {
    write!(io::stdout(), ":: {}? [Y/n] ", prompt)?;
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim();

    Ok(input == "" || input == "y" || input == "Y")
}

pub fn new_client() -> Result<(String, String)> {
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

    Ok((id, secret))
}

pub fn set_default() -> Result<bool> {
    confirm("Set new client as default")
}

pub fn display(value: &str) -> Result<()> {
    Ok(writeln!(io::stdout(), "{}", value)?)
}
