use std::path::PathBuf;

use crate::{history::History, line_view::LineView};

#[derive(Debug, Clone)]
pub struct State {
    pub content: LineView,
    pub history: History,
    pub file_path: PathBuf,
    pub edit: bool,
}

impl State {
    pub fn save(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub fn update_history(&mut self) {
        self.history.push(self.content.clone());
    }
}
