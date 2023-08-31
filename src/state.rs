use std::{fs::File, io::BufWriter, io::Write, path::PathBuf};

use crate::{
    history::History,
    line_view::{Action, LineView},
};

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
            content:
                LineView {
                    lines,
                    action: Action { prefix, suffix },
                },
            file_path,
            ..
        } = self;
        let mut file = BufWriter::new(File::create(file_path)?);

        writeln!(file, "#!/usr/bin/env line-viewer")?;

        for pre in prefix {
            writeln!(file, "#-pre {}", pre.trim())?;
        }
        for suf in suffix {
            writeln!(file, "#-suf {}", suf.trim())?;
        }
        for line in lines {
            writeln!(file, "{}", line.trim())?;
        }

        file.flush()?;

        Ok(())
    }

    pub fn update_history(&mut self) {
        self.history.push(self.content.clone());
    }
}
