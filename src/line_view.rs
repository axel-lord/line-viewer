pub(crate) mod cmd;
pub(crate) mod directive;
pub(crate) mod file_reader;
pub(crate) mod import;
pub(crate) mod line;
pub(crate) mod line_map;
pub(crate) mod line_read;
pub(crate) mod source;
pub(crate) mod source_action;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashSet;

use self::{cmd::Cmd, line::Line, source::Source};
use crate::{line_view::directive::Directive, Result};

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
            ..Source::open(path.to_path_buf())?
        };
        imported.insert(Arc::clone(&root.path));
        sources.push(root);

        while let Some(source) = sources.last_mut() {
            match source_action::SourceAction::perform(
                source,
                &mut imported,
                &mut lines,
                &mut title,
            )? {
                source_action::SourceAction::Noop => {}
                source_action::SourceAction::Pop => {
                    dbg!(sources.pop());
                }
                source_action::SourceAction::Push(source) => sources.push(source),
                source_action::SourceAction::Extend(source_list) => {
                    sources.reserve(source_list.len());
                    sources.extend(source_list);
                }
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
