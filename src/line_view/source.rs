use std::{
    borrow::Cow,
    cell::RefCell,
    fmt::Debug,
    fs::File,
    iter::FusedIterator,
    path::Path,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::{
    escape_path,
    line_view::{line_map::LineMapNode, Cmd, PathSet},
    Directive, FileReader, LineRead, PathExt, Result, DynLineRead,
};

type ParseResult<T> = std::result::Result<T, Cow<'static, str>>;

#[derive(Debug, Default)]
pub enum Watch {
    Watching {
        occured: Vec<String>,
    },
    #[default]
    Sleeping,
}

impl Watch {
    pub fn watch(&mut self) {
        if self.is_sleeping() {
            *self = Self::Watching { occured: Vec::new() }
        }
    }
    pub fn sleep(&mut self) {
        if self.is_watching() {
            *self = Self::Sleeping
        }
    }
    pub fn is_sleeping(&self) -> bool {
        matches!(self, Watch::Sleeping)
    }
    pub fn is_watching(&self) -> bool {
        matches!(self, Watch::Watching { .. })
    }
}

#[derive(Debug)]
pub struct Source {
    pub read: DynLineRead,
    pub path: Arc<Path>,
    pub cmd: Arc<RwLock<Cmd>>,
    pub sourced: Arc<RwLock<PathSet>>,
    pub dir: Arc<Path>,
    pub is_root: bool,
    pub warning_watcher: Rc<RefCell<Watch>>,
    pub line_map: Option<LineMapNode>,
}

impl Source {
    pub fn new(path: Arc<Path>) -> Self {
        Self {
            read: DynLineRead::new(NullReader),
            dir: {
                let mut dir = path.to_path_buf();
                dir.pop();
                dir.into()
            },
            path,
            sourced: Default::default(),
            cmd: Default::default(),
            warning_watcher: Default::default(),
            is_root: false,
            line_map: None,
        }
    }

    pub fn shallow(&self) -> Self {
        Self {
            read: DynLineRead::new(NullReader),
            path: self.path.clone(),
            cmd: self.cmd.clone(),
            sourced: self.sourced.clone(),
            dir: self.dir.clone(),
            is_root: self.is_root,
            warning_watcher: self.warning_watcher.clone(),
            line_map: self.line_map.clone(),
        }
    }

    pub fn open(path: Arc<Path>) -> Result<Self> {
        Ok(Source {
            read: DynLineRead::new(FileReader::new(File::open(&path)?)),
            ..Source::new(path)
        })
    }

    pub fn one_shot(&self, position: usize, directive: Directive<'static>) -> Self {
        Source {
            read: DynLineRead::new(OneShot(position, Some(directive))),
            ..self.shallow()
        }
    }

    pub fn multiple<IntoIter>(&self, position: usize, parses: IntoIter) -> Self
    where
        IntoIter: IntoIterator + 'static,
        IntoIter::IntoIter: Debug + FusedIterator<Item = Directive<'static>>,
    {
        Source {
            read: DynLineRead::new(multiple(position, parses)),
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

        Source::open(path.into())
            .map_err(|err| Cow::from(format!("could not create source, {err}")))
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
