pub(crate) mod cmd;
pub(crate) mod line;

mod import;
mod source;

use std::fmt::Debug;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rustc_hash::FxHashSet;

use self::{cmd::Cmd, line::Line, source::Source};
use crate::Result;

type PathSet = FxHashSet<Arc<Path>>;

#[derive(Debug, Clone, Default)]
pub enum ParsedLine<'s> {
    #[default]
    Empty,
    End,
    Text(&'s str),
    Warning(String),
}

pub trait LineReader: Debug {
    fn read(&mut self) -> Result<(usize, ParsedLine<'_>)>;
}

#[derive(Debug)]
pub struct FileReader<R>(BufReader<R>, usize, String);

impl<R> FileReader<R>
where
    R: Read,
{
    pub fn new(read: R) -> Self {
        Self(BufReader::new(read), 0, String::new())
    }
}

impl<R> LineReader for FileReader<R>
where
    R: Debug + Read,
{
    fn read(&mut self) -> Result<(usize, ParsedLine<'_>)> {
        let Self(read, pos, buf) = self;

        let pos = {
            *pos += 1;
            *pos - 1
        };

        buf.clear();
        if read.read_line(buf)? == 0 {
            return Ok((pos, ParsedLine::End));
        }

        let text = buf.trim_end();
        if text.is_empty() {
            return Ok((pos, ParsedLine::Empty));
        }

        Ok((pos, ParsedLine::Text(text)))
    }
}

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
            ref skip_directives,
        }) = sources.last_mut()
        {
            // makes use of bools easier
            let is_root = *is_root;
            let skip_directives = *skip_directives;

            // pop current layer of stack if averything is read
            // if read.read_line(&mut line)? == 0 {
            //     sources.pop();
            //     continue;
            // }
            let (position, parsed_line) = read.read()?;

            // shared start of builder
            let builder = line::Builder::new().source(path.into()).position(position);

            let line = match parsed_line {
                ParsedLine::Empty => {
                    lines.push(builder.build());
                    continue;
                }
                ParsedLine::End => {
                    sources.pop();
                    continue;
                }
                ParsedLine::Warning(s) => {
                    lines.push(builder.warning().text(s).build());
                    continue;
                }
                ParsedLine::Text(s) => s.trim_end(),
            };

            // Line not a comment or skip directives active
            if let Some(line) = (!line.starts_with('#'))
                .then_some(line)
                .or_else(|| line.starts_with("##").then(|| &line[1..]))
            {
                lines.push(builder.text(line.into()).cmd(Arc::clone(cmd)).build());
                continue;
            }

            // Escape # by doubling it

            // Line a regular comment, or skip directives active
            if skip_directives || !line.starts_with("#-") {
                continue;
            }

            let line = line[2..].trim_end();

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
