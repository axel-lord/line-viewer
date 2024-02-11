use std::{
    borrow::Cow,
    fmt::Debug,
    fs::File,
    iter::FusedIterator,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::{
    escape_path,
    line_view::{line_map::LineMapNode, Cmd, PathSet},
    Directive, FileReader, LineRead, PathExt, Result,
};

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
    pub fn new(path: PathBuf) -> Self {
        Self {
            read: Box::new(NullReader),
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

    pub fn shallow(&self) -> Self {
        let Self {
            path,
            cmd,
            sourced,
            dir,
            is_root,
            line_map,
            ..
        } = self;
        Self {
            read: Box::new(NullReader),
            path: path.clone(),
            cmd: cmd.clone(),
            sourced: sourced.clone(),
            dir: dir.clone(),
            is_root: *is_root,
            line_map: line_map.clone(),
        }
    }

    pub fn open(path: PathBuf) -> Result<Self> {
        Ok(Source {
            read: Box::new(FileReader::new(File::open(&path)?)),
            ..Source::new(path)
        })
    }

    pub fn one_shot(&self, position: usize, directive: Directive<'static>) -> Self {
        Source {
            read: Box::new(OneShot(position, Some(directive))),
            ..self.shallow()
        }
    }

    pub fn multiple<IntoIter>(&self, position: usize, parses: IntoIter) -> Self
    where
        IntoIter: IntoIterator + 'static,
        IntoIter::IntoIter: Debug + FusedIterator<Item = Directive<'static>>,
    {
        Source {
            read: Box::new(multiple(position, parses)),
            ..self.shallow()
        }
    }

    pub fn parse(line: &str, dir: &Path) -> ParseResult<Self> {
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
struct OneShot(pub usize, pub Option<Directive<'static>>);

impl LineRead for OneShot {
    fn read(&mut self) -> Result<(usize, Directive<'_>)> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
struct NullReader;

impl LineRead for NullReader {
    fn read(&mut self) -> Result<(usize, Directive<'_>)> {
        Ok((0, Directive::Noop))
    }
}

#[derive(Clone, Debug)]
struct Multiple<I>(usize, I);

fn multiple<IntoIter>(position: usize, parses: IntoIter) -> Multiple<IntoIter::IntoIter>
where
    IntoIter: IntoIterator + 'static,
    IntoIter::IntoIter: Debug + FusedIterator<Item = Directive<'static>>,
{
    Multiple(position, parses.into_iter())
}

impl<I> LineRead for Multiple<I>
where
    I: Debug + FusedIterator<Item = Directive<'static>>,
{
    fn read(&mut self) -> Result<(usize, Directive<'_>)> {
        todo!()
    }
}
