use std::{
    borrow::Cow,
    cell::RefCell,
    path::Path,
    sync::{Arc, RwLock},
};

use crate::{
    line_view::{cmd::Cmd, line, Directive, PathSet, Source},
    Line, Result,
};

use super::{line_map::LineMapNode, source::Watch};

struct Lines<'lines> {
    pub lines: &'lines mut Vec<Line>,
    pub path: &'lines Arc<Path>,
    pub cmd: &'lines Arc<RwLock<Cmd>>,
    pub warning_watcher: &'lines RefCell<Watch>,
    pub position: usize,
}

impl<'lines> Lines<'lines> {
    fn builder(&self) -> line::Builder<line::Source, usize> {
        line::Builder::new()
            .source(self.path.into())
            .position(self.position)
    }

    fn push_warning(&mut self, text: Cow<'_, str>) {
        if let Watch::Watching { occured } = &mut *self.warning_watcher.borrow_mut() {
            occured.push(text.to_string())
        } else {
            self.lines
                .push(self.builder().warning().text(text.into()).build());
        }
    }
    fn push_subtitle(&mut self, text: Cow<'_, str>) {
        self.lines
            .push(self.builder().title().text(text.into()).build());
    }
    fn push_line(&mut self, text: Cow<'_, str>) {
        self.lines.push(
            self.builder()
                .text(text.into())
                .cmd(Arc::clone(self.cmd))
                .build(),
        );
    }
    fn push_empty(&mut self) {
        self.lines.push(self.builder().build());
    }
}

#[derive(Debug)]
pub enum SourceAction {
    Noop,
    Pop,
    Push(Source),
    Extend(Vec<Source>),
}

impl SourceAction {
    pub fn perform(
        source: &mut Source,
        imported: &mut PathSet,
        lines: &mut Vec<Line>,
        title: &mut String,
    ) -> Result<SourceAction> {
        let shallow = source.shallow();
        let Source {
            read,
            ref path,
            cmd,
            ref is_root,
            line_map,
            ref warning_watcher,
            ..
        } = source;

        // makes use of bools easier
        let is_root = *is_root;

        // read line
        let (position, directive) = read.read()?;

        // shared start of builder
        let mut lines = Lines {
            lines,
            path,
            position,
            cmd,
            warning_watcher,
        };

        // apply maps in reverse order
        let directive = if let Some(line_map) = line_map.as_ref() {
            let mut directive = directive;
            for line_map in line_map {
                directive = line_map.map(directive);
            }
            directive
        } else {
            directive
        };

        match dbg!(directive) {
            Directive::Noop | Directive::Comment(..) => {}
            Directive::Close => {
                return Ok(SourceAction::Pop);
            }
            Directive::Clean => {
                *cmd = Arc::default();
            }
            Directive::Prefix(pre) => {
                cmd.write().unwrap().pre(pre);
            }
            Directive::Suffix(suf) => {
                cmd.write().unwrap().suf(suf);
            }
            Directive::Then(_sub_directive) => {
                fn try_block(_directive: Directive<'_>) -> Directive<'_> {
                    todo!()
                }
                compile_error!("dont compile until implemented");
                let prev = line_map.take();
                *line_map = Some(LineMapNode::new(try_block, prev, false));
            }
            Directive::IgnoreWarnings => {
                fn ignore_warnings(directive: Directive<'_>) -> Directive<'_> {
                    match directive {
                        Directive::Warning(..) => Directive::Noop,
                        other => other,
                    }
                }
                let prev = line_map.take();
                *line_map = Some(LineMapNode::new(ignore_warnings, prev, false));
            }
            Directive::IgnoreText => {
                fn ignore_text(directive: Directive<'_>) -> Directive<'_> {
                    match directive {
                        Directive::Text(..) => Directive::Noop,
                        other => other,
                    }
                }
                let prev = line_map.take();
                *line_map = Some(LineMapNode::new(ignore_text, prev, false));
            }
            Directive::EndMap { automatic } => {
                if let Some(line_map_ref) = line_map.as_ref() {
                    if line_map_ref.automatic() == automatic {
                        *line_map = line_map_ref.prev();
                    } else if automatic {
                        let msg = "EndMap directive was issued automatically whilst a manual end directive was required";
                        lines.push_warning(msg.into());
                    } else {
                        let msg = "end directive was given when an automatic EndMap directive was required";
                        lines.push_warning(msg.into());
                    }
                } else if automatic {
                    let msg = "EndMap directive was issued automatically with no LineMap in use";
                    lines.push_warning(msg.into());
                } else {
                    let msg = "end directive used with nothing to end";
                    lines.push_warning(msg.into());
                }
            }
            Directive::Warning(warn) => {
                lines.push_warning(warn);
            }
            Directive::Title(text) => {
                if is_root {
                    *title = text.into();
                }
            }
            Directive::Subtitle(text) => {
                lines.push_subtitle(text);
            }
            Directive::Import(import) => {
                return Ok(SourceAction::Push(
                    match import.perform_import(shallow.shallow(), imported) {
                        Ok(source) => source,
                        Err(directive) => shallow.one_shot(position, directive),
                    },
                ));
            }
            Directive::Empty => lines.push_empty(),
            Directive::Text(text) => lines.push_line(text),

            Directive::Multiple(parses) => {
                return Ok(SourceAction::Push(shallow.multiple(position, parses)));
            }
        };

        Ok(SourceAction::Noop)
    }
}
