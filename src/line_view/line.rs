use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use crate::{line_view::cmd::Cmd, Result};

#[derive(Debug, Clone)]
pub struct Builder {
    source: Arc<Path>,
    text: String,
    cmd: Option<Arc<RwLock<Cmd>>>,
    is_title: bool,
}

impl Builder {
    pub fn new(source: Arc<Path>) -> Self {
        Self {
            source,
            text: String::new(),
            cmd: None,
            is_title: false,
        }
    }

    pub fn build(self) -> Line {
        Line {
            text: self.text,
            source: self.source,
            cmd: self.cmd.unwrap_or_default(),
            is_title: self.is_title,
        }
    }

    pub fn text(self, text: String) -> Self {
        Self { text, ..self }
    }

    pub fn title(self) -> Self {
        Self { is_title: true, ..self }
    }

    pub fn cmd(self, cmd: Arc<RwLock<Cmd>>) -> Self {
        Self {
            cmd: Some(cmd),
            ..self
        }
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    text: String,
    source: Arc<Path>,
    cmd: Arc<RwLock<Cmd>>,
    is_title: bool,
}

impl Line {
    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn has_command(&self) -> bool {
        !self.cmd.read().unwrap().is_empty()
    }

    pub fn is_title(&self) -> bool {
        self.is_title
    }

    pub fn execute(&self) -> Result {
        self.cmd.read().unwrap().execute([self.text()])
    }
}
