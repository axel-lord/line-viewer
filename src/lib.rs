mod error;
mod line_view;
mod path_ext;

use std::path::PathBuf;

pub use self::{
    error::Error,
    line_view::{
        directive::Directive,
        file_reader::FileReader,
        line::{Line, Source as LineSource},
        line_read::{LineRead, LineReader},
        source::Source,
        source_action::SourceAction,
        LineView,
    },
    path_ext::PathExt,
};
pub fn escape_path(line: &str) -> std::result::Result<PathBuf, &'static str> {
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
