use std::{
    path::Path,
    sync::{Arc, RwLock}, fmt::Display,
};

use crate::{line_view::cmd::Cmd, Result};

#[derive(Debug, Clone, Copy, Default)]
enum Kind {
    #[default]
    Default,
    Title,
    Warning,
}

#[derive(Debug, Clone)]
pub enum Source {
    File(Arc<Path>),
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::File(src) => write!(f, "FILE:{}", src.display()),
        }
    }
}

impl From<Arc<Path>> for Source {
    fn from(value: Arc<Path>) -> Self {
        Self::File(value)
    }
}

impl From<&Arc<Path>> for Source {
    fn from(value: &Arc<Path>) -> Self {
        Self::File(Arc::clone(value))
    }
}

#[derive(Debug, Clone)]
pub struct Builder<T, P> {
    source: T,
    position: P,
    text: String,
    cmd: Option<Arc<RwLock<Cmd>>>,
    kind: Kind,
}

impl Builder<(), ()> {
    pub fn new() -> Self {
        Builder {
            source: (),
            position: (),
            text: String::new(),
            cmd: None,
            kind: Kind::default(),
        }
    }
}

impl<T, P> Builder<T, P> {
    pub fn source(self, source: Source) -> Builder<Source, P> {
        let Self {
            position,
            text,
            cmd,
            kind,
            ..
        } = self;
        Builder {
            source,
            position,
            text,
            cmd,
            kind,
        }
    }

    pub fn position(self, position: usize) -> Builder<T, usize> {
        let Self {
            source,
            text,
            cmd,
            kind,
            ..
        } = self;
        Builder {
            source,
            position,
            text,
            cmd,
            kind,
        }
    }

    pub fn text(self, text: String) -> Self {
        Self { text, ..self }
    }

    pub fn title(self) -> Self {
        Self {
            kind: Kind::Title,
            ..self
        }
    }

    pub fn warning(self) -> Self {
        Self {
            kind: Kind::Warning,
            ..self
        }
    }

    pub fn cmd(self, cmd: Arc<RwLock<Cmd>>) -> Self {
        Self {
            cmd: Some(cmd),
            ..self
        }
    }
}

impl Builder<Source, usize> {
    pub fn build(self) -> Line {
        let Self {
            source,
            position,
            text,
            cmd,
            kind,
        } = self;
        Line {
            text,
            source,
            position,
            cmd: cmd.unwrap_or_default(),
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    text: String,
    source: Source,
    position: usize,
    cmd: Arc<RwLock<Cmd>>,
    kind: Kind,
}

impl Line {
    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn line(&self) -> usize {
        self.position
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn has_command(&self) -> bool {
        !self.cmd.read().unwrap().is_empty()
    }

    pub fn is_title(&self) -> bool {
        matches!(self.kind, Kind::Title)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self.kind, Kind::Warning)
    }

    pub fn execute(&self) -> Result {
        self.cmd.read().unwrap().execute(self.position, self.source.clone(), [self.text()])
    }
}
