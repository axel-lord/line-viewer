use std::path::Path;

use super::Line;
use crate::{Cmd, Result};

#[derive(Debug, Clone, Copy)]
pub struct Handle<'lines> {
    cmd: &'lines Cmd,
    line: &'lines Line,
}

impl<'lines> Handle<'lines> {
    pub(crate) fn new(cmd: &'lines Cmd, line: &'lines Line) -> Self {
        Self { cmd, line }
    }

    pub fn execute(&self) -> Result {
        self.cmd.execute([self.line.text()])
    }

    pub fn text(&self) -> &str {
        self.line.text()
    }

    pub fn source(&self) -> &Path {
        self.line.source()
    }
}

impl AsRef<str> for Handle<'_> {
    fn as_ref(&self) -> &str {
        self.text()
    }
}

