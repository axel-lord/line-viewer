pub mod cmd;
pub mod provide {
    pub trait Read {
        type Err;
        type BufRead<'r>: std::io::BufRead + 'r
        where
            Self: 'r;

        fn provide<'r>(&'r self, from: &str) -> Result<Self::BufRead<'r>, Self::Err>;
    }

    #[derive(Clone, Copy, Debug)]
    pub struct PathReadProvider;
    impl self::Read for PathReadProvider {
        type Err = std::io::Error;
        type BufRead<'r> = std::io::BufReader<std::fs::File>;

        fn provide<'r>(&'r self, from: &str) -> Result<Self::BufRead<'r>, Self::Err> {
            Ok(std::io::BufReader::new(std::fs::File::open(from)?))
        }
    }
}

mod error;
mod line_view;
mod path_ext;

use std::path::PathBuf;

pub use self::{
    cmd::Cmd,
    error::Error,
    line_view::{
        directive::Directive,
        directive_reader::DirectiveReader,
        directive_source::{DirectiveSource, DirectiveStream},
        line::{Line, Source as LineSource},
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
