use std::{fs::File, io::BufWriter, io::Write as _, path::PathBuf};

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
        let Self {
            content, file_path, ..
        } = self;
        let mut file = BufWriter::new(File::create(file_path)?);

        writeln!(file, "#!/usr/bin/env line-viewer")?;

        content.write(&mut file)?;

        file.flush()?;

        Ok(())
    }

    pub fn update_history(&mut self) {
        self.history.push(self.content.clone());
    }
}
