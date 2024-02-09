pub(crate) mod cmd;
pub(crate) mod file_reader;
pub(crate) mod line;
pub(crate) mod line_map;
pub(crate) mod line_read;

mod import;
mod source;

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashSet;

use self::{cmd::Cmd, line::Line, source::Source};
use crate::{ParsedLine, Result};

type PathSet = FxHashSet<Arc<Path>>;

#[derive(Debug, Clone, Default)]
pub struct LineView {
    source: PathBuf,
    imported: PathSet,
    title: String,
    lines: Vec<Line>,
}

impl LineView {
    pub fn read(path: &Path) -> Result<Self> {
        // clean path
        let path = path.canonicalize()?;

        // setup stack, and source set
        let mut sources = Vec::new();
        let mut imported = FxHashSet::default();

        let mut lines = Vec::new();
        let mut title = path.display().to_string();

        let root = Source {
            is_root: true,
            ..Source::new(path.to_path_buf())?
        };
        imported.insert(Arc::clone(&root.path));
        sources.push(root);

        while let Some(Source {
            read,
            ref path,
            ref dir,
            cmd,
            ref is_root,
            sourced,
            ref line_map,
        }) = sources.last_mut()
        {
            // makes use of bools easier
            let is_root = *is_root;

            // read line
            let (position, parsed_line) = read.read()?;

            // shared start of builder
            let builder = line::Builder::new().source(path.into()).position(position);

            // apply maps in reverse order
            let parsed_line = if let Some(line_map) = line_map.as_ref() {
                let mut parsed_line = parsed_line;
                for line_map in line_map {
                    parsed_line = line_map.map(parsed_line);
                }
                parsed_line
            } else {
                parsed_line
            };

            match dbg!(parsed_line) {
                ParsedLine::None | ParsedLine::Comment(_) => {}
                ParsedLine::Empty => {
                    lines.push(builder.build());
                }
                ParsedLine::End => {
                    dbg!(sources.pop());
                }
                ParsedLine::Warning(s) => {
                    lines.push(builder.warning().text(s.to_string()).build());
                }
                ParsedLine::Directive(line) => {
                    fn get_cmd<'a>(line: &'a str, lit: &str) -> Option<&'a str> {
                        line.strip_prefix(lit)
                            .filter(|s| s.is_empty() || s.starts_with(' '))
                            .map(|s| s.trim_start())
                    }

                    if let Some(line) = get_cmd(line, "") {
                        cmd.write().unwrap().pre(line);
                    } else if let Some(line) = get_cmd(line, "pre") {
                        cmd.write().unwrap().pre(line);
                    } else if let Some(line) = get_cmd(line, "suf") {
                        cmd.write().unwrap().suf(line);
                    } else if let Some(_line) = get_cmd(line, "clean") {
                        *cmd = Arc::default();
                    } else if let Some(line) = get_cmd(line, "title") {
                        // only root may change title
                        if is_root {
                            title = String::from(line);
                        }
                    } else if let Some(line) = get_cmd(line, "subtitle") {
                        lines.push(
                            line::Builder::new()
                                .source(Arc::clone(path).into())
                                .position(position)
                                .text(line.into())
                                .title()
                                .build(),
                        )
                    } else if let Some(line) = get_cmd(line, "import") {
                        if let Some(source) = import::import(line, dir, &mut imported) {
                            sources.push(source);
                        } else {
                            lines.push(
                                builder
                                    .warning()
                                    .text(format!("could not import \"{line}\""))
                                    .build(),
                            )
                        }
                    } else if let Some(line) = get_cmd(line, "source") {
                        if let Some(source) = import::source(line, dir, cmd, sourced) {
                            sources.push(Source { is_root, ..source }) // if something is sourced in root
                                                                       // context it is treated as root
                                                                       // itself
                        }
                    } else if let Some(line) = get_cmd(line, "lines") {
                        if let Some(source) = import::lines(line, dir, cmd) {
                            sources.push(source)
                        }
                    } else {
                        lines.push(
                            builder
                                .text(format!("invalid command \"{line}\""))
                                .warning()
                                .build(),
                        )
                    }
                }
                ParsedLine::Text(line) => {
                    lines.push(builder.text(line.into()).cmd(Arc::clone(cmd)).build());
                }
            };
        }

        if title.is_empty() {
            title = path.display().to_string();
        }

        Ok(Self {
            source: path.to_path_buf(),
            imported,
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
        self.imported.iter().map(|i| i.as_ref())
    }

    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    pub fn get(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }
}

impl IntoIterator for LineView {
    type Item = Line;

    type IntoIter = <Vec<Line> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

impl<'a> IntoIterator for &'a LineView {
    type Item = &'a Line;

    type IntoIter = <&'a Vec<Line> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.iter()
    }
}

impl<'a> IntoIterator for &'a mut LineView {
    type Item = &'a mut Line;

    type IntoIter = <&'a mut Vec<Line> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.iter_mut()
    }
}
