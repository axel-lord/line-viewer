use std::{path::Path, sync::Arc};

#[derive(Debug, Clone)]
pub(crate) struct Line {
    text: String,
    source: Arc<Path>,
}

impl Line {
    pub(crate) fn new(text: String, source: Arc<Path>) -> Self {
        Self { text, source }
    }

    pub fn source(&self) -> &Path {
        &self.source
    }
    pub fn text(&self) -> &str {
        &self.text
    }
}
