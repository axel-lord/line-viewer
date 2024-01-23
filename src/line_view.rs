pub(crate) mod cmd;
pub(crate) mod line;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use self::{cmd::Cmd, line::Line};
use crate::Result;

#[derive(Debug, Clone, Default)]
pub struct LineView {
    source: PathBuf,
    included: rustc_hash::FxHashSet<Arc<Path>>,
    title: String,
    lines: Vec<Line>,
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

#[derive(Debug)]
struct Source {
    read: BufReader<File>,
    path: Arc<Path>,
    cmd: Arc<RwLock<Cmd>>,
    dir: PathBuf,
    is_root: bool,
}
impl Source {
    fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            read: BufReader::new(File::open(&path)?),
            path: path.as_path().into(),
            dir: {
                let mut dir = path;
                dir.pop();
                dir
            },
            cmd: Default::default(),
            is_root: false,
        })
    }

    fn root(path: PathBuf) -> Result<Self> {
        Ok(Self {
            is_root: true,
            ..Self::new(path)?
        })
    }
}

impl LineView {
    pub fn read(path: &Path) -> Result<Self> {
        // clean path
        let path = path.canonicalize()?;

        // setup stack, and source set
        let mut sources = Vec::new();
        let mut included = rustc_hash::FxHashSet::default();

        let mut lines = Vec::new();
        let mut title = path.display().to_string();

        let root = Source::root(path.to_path_buf())?;
        included.insert(Arc::clone(&root.path));
        sources.push(root);

        while let Some(Source {
            read,
            path,
            dir,
            cmd,
            is_root,
        }) = sources.last_mut()
        {
            let mut line = String::new();
            if read.read_line(&mut line)? == 0 {
                sources.pop();
                continue;
            }
            line.truncate(line.trim_end().len());

            // Line not a comment
            if !line.starts_with('#') {
                lines.push(Line::new(line, Arc::clone(path), Arc::clone(cmd)));
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

                let line = match line.strip_prefix(HOME_PREFIX) {
                    Some(line) if line.starts_with(HOME_PREFIX) => PathBuf::from(line),
                    Some(line) => {
                        let Some(home_dir) = home::home_dir() else {
                            eprintln!("could not find user home");
                            continue;
                        };
                        home_dir.join(line)
                    }
                    None => PathBuf::from(line),
                };

                let path = match canonicalize_at(dir, &line) {
                    Ok(line) => line,
                    Err(err) => {
                        eprintln!("could not canonicalize path, {}, {err}", line.display());
                        continue;
                    }
                };

                if !path.exists() {
                    // non canonicalized is uded when printing
                    eprintln!("could not find include {}", line.display());
                    continue;
                }

                let source = match Source::new(path) {
                    Ok(source) => source,
                    Err(err) => {
                        eprintln!("could not create source, {err}");
                        continue;
                    }
                };

                // no warnings needed
                if included.contains(&source.path) {
                    continue;
                }

                included.insert(Arc::clone(&source.path));
                sources.push(source);

                continue;
            }

            let mut cmd = cmd.write().unwrap();

            if let Some(line) = get_cmd!(line, "pre") {
                cmd.pre(line);
                continue;
            }

            if let Some(line) = get_cmd!(line, "suf") {
                cmd.suf(line);
                continue;
            }

            // only for root view
            if *is_root {
                if let Some(line) = get_cmd!(line, "title") {
                    title = String::from(line);
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
