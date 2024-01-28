use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use crate::line_view::{cmd::Cmd, source::Source, PathSet};

pub fn include(line: &str, dir: &Path, included: &mut PathSet) -> Option<Source> {
    let source = match Source::parse(line, dir) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("{err}");
            return None;
        }
    };

    // prevent cycles
    if included.contains(&source.path) {
        return None;
    }

    included.insert(Arc::clone(&source.path));

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
            // sources gain source context of parent, while includes get their own
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

pub fn lines(line: &str, dir: &Path, cmd: &Arc<RwLock<Cmd>>) -> Option<Source> {
    // lines can be sourced however much is wanted since they cannot create cycles
    match Source::parse(line, dir) {
        Ok(source) => Some(Source {
            // lines inherit command from parent
            cmd: Arc::clone(cmd),
            // the special part about lines
            skip_directives: true,
            ..source
        }),
        Err(err) => {
            eprintln!("{err}");
            None
        }
    }
}
