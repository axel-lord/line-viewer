use std::{
    borrow::Cow,
    path::Path,
    sync::{Arc, RwLock},
};

use crate::{
    line_view::{cmd::Cmd, source::Source, PathSet},
    ParsedLine,
};

use super::{directive::Directive, line_map::LineMapNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportKind {
    Source,
    Import,
    Lines,
}

#[derive(Debug, Clone)]
pub struct Import<'line> {
    file: Cow<'line, str>,
    kind: ImportKind,
}

#[derive(Debug)]
pub struct ImportCtx<'ctx> {
    pub is_root: bool,
    pub dir: &'ctx Path,
    pub cmd: &'ctx Arc<RwLock<Cmd>>,
    pub sourced: &'ctx Arc<RwLock<PathSet>>,
    pub imported: &'ctx mut PathSet,
}

impl<'line> Import<'line> {
    pub fn new_import(path: &'line str) -> Self {
        Self {
            file: path.into(),
            kind: ImportKind::Import,
        }
    }
    pub fn new_source(path: &'line str) -> Self {
        Self {
            file: path.into(),
            kind: ImportKind::Source,
        }
    }
    pub fn new_lines(path: &'line str) -> Self {
        Self {
            file: path.into(),
            kind: ImportKind::Lines,
        }
    }

    pub fn perform_import<'ctx>(self, ctx: ImportCtx<'ctx>) -> std::result::Result<Source, Directive<'line>> {
        let Self { file, kind } = self;
        let ImportCtx {
            is_root,
            dir,
            cmd,
            sourced,
            imported,
        } = ctx;
        match kind {
            ImportKind::Source => source(&file, dir, is_root, cmd, sourced),
            ImportKind::Import => import(&file, dir, imported),
            ImportKind::Lines => lines(&file, dir, cmd),
        }
        .ok_or_else(|| Directive::Warning(format!("could not source/import/lines {file}").into()))
    }
}

fn import(line: &str, dir: &Path, imported: &mut PathSet) -> Option<Source> {
    let source = match Source::parse(line, dir) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("{err}");
            return None;
        }
    };

    // prevent cycles
    if imported.contains(&source.path) {
        return None;
    }

    imported.insert(Arc::clone(&source.path));

    Some(source)
}

fn source(
    line: &str,
    dir: &Path,
    is_root: bool,
    cmd: &Arc<RwLock<Cmd>>,
    sourced: &Arc<RwLock<PathSet>>,
) -> Option<Source> {
    let source = match Source::parse(line, dir) {
        Ok(source) => Source {
            // sources gain source context of parent, while imports get their own
            sourced: Arc::clone(sourced),
            // match parent rootness
            is_root,
            // sourced content keep command of parent
            cmd: Arc::clone(cmd),
            ..source
        },
        Err(err) => {
            eprintln!("{err}");
            return None;
        }
    };

    let mut sourced = sourced.write().unwrap();

    // skip if already sourced in this context
    if sourced.contains(&source.path) {
        return None;
    }

    sourced.insert(Arc::clone(&source.path));
    Some(source)
}

fn skip_directives(parsed: ParsedLine<'_>) -> ParsedLine<'_> {
    if matches!(parsed, ParsedLine::Directive(_)) {
        ParsedLine::None
    } else {
        parsed
    }
}

fn lines(line: &str, dir: &Path, cmd: &Arc<RwLock<Cmd>>) -> Option<Source> {
    // lines can be sourced however much is wanted since they cannot create cycles
    match Source::parse(line, dir) {
        Ok(source) => Some(Source {
            // lines inherit command from parent
            cmd: Arc::clone(cmd),
            // the special part about lines
            line_map: Some(LineMapNode::new(skip_directives, None)),
            ..source
        }),
        Err(err) => {
            eprintln!("{err}");
            None
        }
    }
}
