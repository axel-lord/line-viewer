mod cmd;
mod error;
mod line_view;
mod path_ext;

pub mod provide;

pub use self::{
    cmd::Cmd,
    error::Error,
    line_view::{directive::Directive, LineView},
};

use std::path::PathBuf;
fn escape_path(line: &str) -> std::result::Result<PathBuf, &'static str> {
    const HOME_PREFIX: &str = "~/";

    Ok(match line.strip_prefix(HOME_PREFIX) {
        Some(line) if line.starts_with(HOME_PREFIX) => PathBuf::from(line),
        Some(line) => {
            let Some(home_dir) = home::home_dir() else {
                return Err("could not find user home");
            };
            home_dir.join(line)
        }
        None => PathBuf::from(line),
    })
}

pub type Result<T = ()> = std::result::Result<T, Error>;
