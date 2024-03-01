mod directive_reader;
mod directive_source;
mod import;
mod line_map;
mod source;
mod source_action;

pub(crate) mod directive;
pub(crate) mod line;

use std::{path::Path, sync::Arc};

use rustc_hash::FxHashSet;

use crate::{
    cmd::{self, Cmd},
    line_view::{directive::Directive, line::Line, source::Source},
    Result,
};

type PathSet = FxHashSet<Arc<Path>>;

#[derive(Debug, Clone, Default)]
pub struct LineView {
    imported: PathSet,
    title: String,
    lines: Vec<Line<Arc<Cmd>>>,
}

impl LineView {
    pub fn read(path: &Path) -> Result<Self> {
        // clean path
        let path = path.canonicalize()?;

        // setup stack, and source set
        let mut sources = Vec::new();
        let mut imported = FxHashSet::default();

        let mut lines = Vec::new();
        let mut title = None;
        let mut cmd_directory = cmd::Directory::new();

        let root = Source::open(Arc::from(path.as_path()), &mut cmd_directory)?;
        imported.insert(Arc::clone(&root.path));
        sources.push(root);

        while let Some(source) = sources.last_mut() {
            match source_action::SourceAction::perform(
                source,
                &mut imported,
                &mut lines,
                &mut title,
                &mut cmd_directory,
            )? {
                source_action::SourceAction::Noop => {}
                source_action::SourceAction::Pop => {
                    dbg!(sources.pop());
                }
                source_action::SourceAction::Push(source) => sources.push(source),
            }
        }

        let title = title.unwrap_or_else(|| path.display().to_string());

        let cmd_directory = cmd_directory.map_to_arc();
        let lines = lines
            .into_iter()
            .map(|line| line.map_to_arc_cmd(&cmd_directory))
            .collect();

        Ok(Self {
            imported,
            lines,
            title,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
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

    pub fn get(&self, index: usize) -> Option<&Line<Arc<Cmd>>> {
        self.lines.get(index)
    }
}

impl AsRef<LineView> for LineView {
    fn as_ref(&self) -> &LineView {
        self
    }
}

impl<I> std::ops::Index<I> for LineView
where
    Vec<Line<Arc<Cmd>>>: std::ops::Index<I>,
{
    type Output = <Vec<Line<Arc<Cmd>>> as std::ops::Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.lines[index]
    }
}

impl IntoIterator for LineView {
    type Item = Line<Arc<Cmd>>;

    type IntoIter = <Vec<Line<Arc<Cmd>>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

impl<'a> IntoIterator for &'a LineView {
    type Item = &'a Line<Arc<Cmd>>;

    type IntoIter = <&'a Vec<Line<Arc<Cmd>>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.iter()
    }
}

impl<'a> IntoIterator for &'a mut LineView {
    type Item = &'a mut Line<Arc<Cmd>>;

    type IntoIter = <&'a mut Vec<Line<Arc<Cmd>>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.iter_mut()
    }
}
