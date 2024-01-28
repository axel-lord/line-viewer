pub(crate) mod cmd;
pub(crate) mod line;

mod include;
mod source;

use std::sync::Arc;
use std::{
    io::BufRead,
    path::{Path, PathBuf},
};

use rustc_hash::FxHashSet;

use self::{cmd::Cmd, line::Line, source::Source};
use crate::Result;

type PathSet = FxHashSet<Arc<Path>>;

#[derive(Debug, Clone, Default)]
pub struct LineView {
    source: PathBuf,
    included: PathSet,
    title: String,
    lines: Vec<Line>,
}

impl LineView {
    pub fn read(path: &Path) -> Result<Self> {
        // clean path
        let path = path.canonicalize()?;

        // setup stack, and source set
        let mut sources = Vec::new();
        let mut included = FxHashSet::default();

        let mut lines = Vec::new();
        let mut title = path.display().to_string();

        let root = Source {
            is_root: true,
            ..Source::new(path.to_path_buf())?
        };
        included.insert(Arc::clone(&root.path));
        sources.push(root);

        while let Some(Source {
            read,
            path,
            dir,
            cmd,
            is_root,
            sourced,
            skip_directives,
        }) = sources.last_mut()
        {
            let mut line = String::new();

            // pop current layer of stack if averything is read
            if read.read_line(&mut line)? == 0 {
                sources.pop();
                continue;
            }
            line.truncate(line.trim_end().len());

            // Line not a comment or skip directives active
            if *skip_directives || !line.starts_with('#') {
                lines.push(Line::new(line, Arc::clone(path), Arc::clone(cmd)));
                continue;
            }

            // Line a regular comment
            if !line.starts_with("#-") {
                continue;
            }

            let line = line[2..].trim();

            // convenience macro (as of now directives not followed by a space are not allowed)
            macro_rules! get_cmd {
                ($name:expr, $prefix:literal) => {
                    $name.strip_prefix(concat!($prefix, " ")).map(|s| s.trim())
                };
            }

            if let Some(line) = get_cmd!(line, "include") {
                if let Some(source) = include::include(line, dir, &mut included) {
                    sources.push(source);
                }
            } else if let Some(line) = get_cmd!(line, "source") {
                if let Some(source) = include::source(line, dir, cmd, sourced) {
                    sources.push(source)
                }
            } else if let Some(line) = get_cmd!(line, "lines") {
                if let Some(source) = include::lines(line, dir, cmd) {
                    sources.push(source)
                }
            } else if let Some(line) = get_cmd!(line, "pre") {
                cmd.write().unwrap().pre(line);
            } else if let Some(line) = get_cmd!(line, "suf") {
                cmd.write().unwrap().suf(line);
            } else if let Some(line) = get_cmd!(line, "title") {
                // only root may change title
                if *is_root {
                    title = String::from(line);
                }
            }
        }

        if title.is_empty() {
            title = path.display().to_string();
        }

        Ok(Self {
            source: path.to_path_buf(),
            included,
            lines,
            title,
        })
    }

    pub fn reload(&mut self) -> Result {
        let new = Self::read(self.source())?;
        *self = new;
        Ok(())
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(Line::text)
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn all_sources(&self) -> impl Iterator<Item = &Path> {
        self.included.iter().map(|i| i.as_ref())
    }

    pub fn get(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }
}
