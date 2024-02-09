use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use crate::{
    line_view::{cmd::Cmd, source::Source, PathSet},
    ParsedLine,
};

use super::line_map::LineMapNode;

pub fn import(line: &str, dir: &Path, imported: &mut PathSet) -> Option<Source> {
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

pub fn source(
    line: &str,
    dir: &Path,
    cmd: &Arc<RwLock<Cmd>>,
    sourced: &Arc<RwLock<PathSet>>,
) -> Option<Source> {
    let source = match Source::parse(line, dir) {
        Ok(source) => Source {
            // sources gain source context of parent, while imports get their own
            sourced: Arc::clone(sourced),
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

pub fn lines(line: &str, dir: &Path, cmd: &Arc<RwLock<Cmd>>) -> Option<Source> {
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
