use std::{
    borrow::Cow,
    fmt::Debug,
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::{
    line_view::{Cmd, PathSet},
    FileReader, LineRead, ParsedLine, PathExt, Result,
};

use super::line_map::LineMapNode;

type ParseResult<T> = std::result::Result<T, Cow<'static, str>>;
#[derive(Debug)]

pub struct Source {
    pub read: Box<dyn LineRead>,
    pub path: Arc<Path>,
    pub cmd: Arc<RwLock<Cmd>>,
    pub sourced: Arc<RwLock<PathSet>>,
    pub dir: Arc<Path>,
    pub is_root: bool,
    pub line_map: Option<LineMapNode>,
}

impl Source {
    pub fn new(path: PathBuf, position: usize) -> Self {
        Self {
            read: Box::new(NullReader(position)),
            path: path.as_path().into(),
            dir: {
                let mut dir = path;
                dir.pop();
                dir.into()
            },
            sourced: Default::default(),
            cmd: Default::default(),
            is_root: false,
            line_map: None,
        }
    }

    pub fn open(path: PathBuf) -> Result<Self> {
        Ok(Source {
            read: Box::new(FileReader::new(File::open(&path)?)),
            ..Source::new(path, 0)
        })
    }

    pub fn one_shot(path: PathBuf, position: usize, directive: ParsedLine<'static>) -> Self
    {
        Source {
            read: Box::new(OneShot(position, Some(directive))),
            ..Self::new(path, position)
        }
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

        Source::open(path).map_err(|err| Cow::from(format!("could not create source, {err}")))
    }
}

#[derive(Clone, Debug)]
struct OneShot(pub usize, pub Option<ParsedLine<'static>>);

impl LineRead for OneShot
{
    fn read(&mut self) -> Result<(usize, ParsedLine<'_>)> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
struct NullReader(usize);

impl LineRead for NullReader {
    fn read(&mut self) -> Result<(usize, ParsedLine<'_>)> {
        Ok((self.0, ParsedLine::None))
    }
}
