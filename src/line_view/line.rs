use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use crate::{line_view::cmd::Cmd, Result};

#[derive(Debug, Clone)]
pub struct Line {
    text: String,
    source: Arc<Path>,
    cmd: Arc<RwLock<Cmd>>,
}

impl Line {
    pub fn new(text: String, source: Arc<Path>, cmd: Arc<RwLock<Cmd>>) -> Self {
        Self { text, source, cmd }
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn has_command(&self) -> bool {
        !self.cmd.read().unwrap().is_empty()
    }

    pub fn execute(&self) -> Result {
        self.cmd.read().unwrap().execute([self.text()])
    }
}
