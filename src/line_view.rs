pub(crate) mod cmd;
pub(crate) mod handle;
pub(crate) mod line;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::Arc,
};

use self::{cmd::Cmd, handle::Handle, line::Line};
use crate::Result;

#[derive(Debug, Clone, Default)]
pub struct LineView {
    source: PathBuf,
    included: rustc_hash::FxHashSet<Arc<Path>>,
    title: String,
    lines: Vec<Line>,
    cmd: Cmd,
}

fn canonicalize_at(dest: &Path, path: &Path) -> Result<PathBuf> {
    fn internal(dest: &Path, path: &Path) -> Result<PathBuf> {
        std::env::set_current_dir(dest)?;
        Ok(path.canonicalize()?)
    }

    let s = std::env::current_dir()?;
    let r = internal(dest, path);
    std::env::set_current_dir(s)?;
    r
}

impl LineView {
    pub fn read(path: &Path) -> Result<Self> {
        // clean path
        let path = path.canonicalize()?;

        // setup stack, and source set
        struct Source {
            read: BufReader<File>,
            path: Arc<Path>,
            dir: PathBuf,
        }
        let mut sources = Vec::new();
        let mut included = rustc_hash::FxHashSet::default();

        let mut lines = Vec::new();
        let mut title = String::new();
        let mut cmd = Cmd::new();

        // push path as first source
        {
            let path: Arc<Path> = path.as_path().into();
            let dir = {
                let mut dir = path.to_path_buf();
                dir.pop();
                dir
            };

            included.insert(Arc::clone(&path));
            sources.push(Source {
                read: BufReader::new(File::open(&path)?),
                dir,
                path,
            });
        }

        while let Some(Source { read, path, dir }) = sources.last_mut() {
            let mut line = String::new();
            if read.read_line(&mut line)? == 0 {
                sources.pop();
                continue;
            }
            line.truncate(line.trim_end().len());

            // Line not a comment
            if !line.starts_with('#') {
                lines.push(Line::new(line, Arc::clone(path)));
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

            if let Some(line) = get_cmd!(line, "include") {
                const HOME_PREFIX: &str = "~/";
                let line = if let Some(line) = line.strip_prefix(HOME_PREFIX) {
                    if line.starts_with(HOME_PREFIX) {
                        PathBuf::from(line)
                    } else {
                        // print error and continue without include if home does not exist
                        let Some(home_dir) = home::home_dir() else {
                            eprintln!("could not find user home");
                            continue;
                        };
                        home_dir.join(line)
                    }
                } else {
                    PathBuf::from(line)
                };
                dbg!(&line);
                if !line.exists() {
                    eprintln!("could not find include {}", line.display());
                    continue;
                }
                let new_source = canonicalize_at(dir, &line).and_then(|path| {
                    Ok(Source {
                        read: BufReader::new(File::open(&path)?),
                        dir: {
                            let mut dir = path.clone();
                            dir.pop();
                            dir
                        },
                        path: path.into(),
                    })
                });
                if let Ok(source) = new_source {
                    if !included.contains(&source.path) {
                        included.insert(Arc::clone(&source.path));
                        sources.push(source);
                    }
                }
                continue;
            }

            // only for root view
            if sources.len() == 1 {
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
        }

        if title.is_empty() {
            title = path.display().to_string();
        }

        Ok(Self {
            source: path.to_path_buf(),
            included,
            lines,
            title,
            cmd,
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

    pub fn get(&self, index: usize) -> Option<Handle> {
        Some(Handle::new(&self.cmd, self.lines.get(index)?))
    }
}
