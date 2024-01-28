use std::{
    borrow::Cow,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::{
    line_view::{Cmd, PathSet},
    PathExt, Result,
};

type ParseResult<T> = std::result::Result<T, Cow<'static, str>>;
#[derive(Debug)]

pub struct Source {
    pub read: BufReader<File>,
    pub path: Arc<Path>,
    pub cmd: Arc<RwLock<Cmd>>,
    pub sourced: Arc<RwLock<PathSet>>,
    pub dir: PathBuf,
    pub is_root: bool,
    pub skip_directives: bool,
}

impl Source {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            read: BufReader::new(File::open(&path)?),
            path: path.as_path().into(),
            dir: {
                let mut dir = path;
                dir.pop();
                dir
            },
            sourced: Default::default(),
            cmd: Default::default(),
            is_root: false,
            skip_directives: false,
        })
    }

    pub fn parse(line: &str, dir: &Path) -> ParseResult<Self> {
        fn escape_path(line: &str) -> ParseResult<PathBuf> {
            const HOME_PREFIX: &str = "~/";

            Ok(match line.strip_prefix(HOME_PREFIX) {
                Some(line) if line.starts_with(HOME_PREFIX) => PathBuf::from(line),
                Some(line) => {
                    let Some(home_dir) = home::home_dir() else {
                        return Err("could not find user home".into());
                    };
                    home_dir.join(line)
                }
                None => PathBuf::from(line),
            })
        }

        let line = escape_path(line)?;

        let path = line.canonicalize_at(dir).map_err(|err| {
            Cow::Owned(format!(
                "could not canonicalize path, {}, {err}",
                line.display()
            ))
        })?;

        if !path.exists() {
            // non canonicalized is uded when printing
            return Err(Cow::from(format!("could not find {}", line.display())));
        }

        Source::new(path).map_err(|err| Cow::from(format!("could not create source, {err}")))
    }
}
