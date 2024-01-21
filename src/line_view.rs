use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct Cmd {
    pre: Vec<String>,
    suf: Vec<String>,
}

impl Cmd {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pre(&mut self, arg: impl Into<String>) -> &mut Self {
        self.pre.push(arg.into());
        self
    }

    pub fn suf(&mut self, arg: impl Into<String>) -> &mut Self {
        self.suf.push(arg.into());
        self
    }

    pub fn execute(&self, params: impl IntoIterator<Item = impl Into<String>>) -> Result {
        let mut args = self
            .pre
            .iter()
            .cloned()
            .chain(params.into_iter().map(Into::into))
            .chain(self.suf.iter().cloned());

        let Some(program) = args.next() else {
            return Ok(());
        };

        let args = args.collect::<Vec<_>>();

        std::process::Command::new(&program)
            .args(&args)
            .spawn()
            .map_err(|err| Error::Spawn { err, program, args })?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Handle<'lines> {
    cmd: &'lines Cmd,
    line: &'lines str,
}

impl Handle<'_> {
    pub fn execute(&self) -> Result {
        self.cmd.execute([self.line])
    }
}

impl AsRef<str> for Handle<'_> {
    fn as_ref(&self) -> &str {
        self.line
    }
}

#[derive(Debug, Clone, Default)]
pub struct LineView {
    source: PathBuf,
    title: String,
    lines: Vec<String>,
    cmd: Cmd,
}

impl LineView {
    pub fn read(path: &Path) -> Result<Self> {
        let mut sources = Vec::new();
        let mut lines = Vec::new();
        let mut title = String::new();
        let mut cmd = Cmd::new();

        sources.push(BufReader::new(File::open(path)?));

        while let Some(source) = sources.last_mut() {
            let mut line = String::new();
            if source.read_line(&mut line)? == 0 {
                sources.pop();
                continue;
            }
            line.truncate(line.trim_end().len());

            // Line not a comment
            if !line.starts_with("#") {
                lines.push(line);
                continue;
            }

            // Line a regular comment
            if !line.starts_with("#-") {
                continue;
            }

            let line = line[2..].trim();

            macro_rules! get_cmd {
                ($name:expr, $prefix:literal) => {
                    $name.strip_prefix(concat!($prefix, " ")).map(|s| s.trim())
                };
            }

            if let Some(line) = get_cmd!(line, "title") {
                title = String::from(line);
                continue;
            }

            if let Some(line) = get_cmd!(line, "pre") {
                cmd.pre(line);
                continue;
            }

            if let Some(line) = get_cmd!(line, "suf") {
                cmd.suf(line);
                continue;
            }
        }

        Ok(Self {
            source: PathBuf::from(path),
            lines,
            title,
            cmd,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(AsRef::<str>::as_ref)
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn get(&self, index: usize) -> Option<Handle> {
        Some(Handle {
            cmd: &self.cmd,
            line: self.lines.get(index)?,
        })
    }
}
