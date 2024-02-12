use std::{
    borrow::Cow,
    cell::RefCell,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::{
    line_view::{cmd::Cmd, line, Directive, PathSet, Source},
    DirectiveSource as _, Line, Result,
};

use super::{
    line_map::{DirectiveMapper, DirectiveMapperChain},
    source::Watch,
};

struct Lines<'lines> {
    pub lines: &'lines mut Vec<Line>,
    pub path: &'lines Arc<Path>,
    pub cmd: &'lines Arc<RwLock<Cmd>>,
    pub warning_watcher: &'lines RefCell<Watch>,
    pub position: usize,
}

struct Then {
    warnings: Vec<String>,
}

impl From<Vec<String>> for Then {
    fn from(value: Vec<String>) -> Self {
        Self { warnings: value }
    }
}

impl DirectiveMapper for Then {
    fn map<'l>(&self, line: Directive<'l>, depth: usize) -> Directive<'l> {
        match (self.warnings.is_empty(), line) {
            // regerdless of if there are any warnings else is encountered
            // since the else should share end block and warnings we need to
            // automatically add an end to this block, start the watch again
            // add the warnings back to be watched and then readd the else
            // directive. else becomes end, watch, warning..., else
            (_, Directive::Else) => Directive::Multiple(
                [Directive::EndMap { automatic: false }, Directive::Watch]
                    .into_iter()
                    .chain(
                        self.warnings
                            .iter()
                            .map(|warning| Directive::Warning(warning.clone().into())),
                    )
                    .chain(std::iter::once(Directive::Else))
                    .collect(),
            ),

            // there are no warnings
            (true, other) => other,

            // there are warnings but close is encountered, close needs to
            // be forwarded sice it is used to pop the source
            (false, Directive::Close) => Directive::Close,

            // there are warnings but an end is encountered and
            // the depth is 0 meaning we are the top map, has
            // to be forwarded to ensure this closes
            (false, directive @ Directive::EndMap { .. }) if depth == 0 => directive,

            // there are warnings, other directives become noop
            (false, _) => Directive::Noop,
        }
    }

    fn name(&self) -> &str {
        "Then"
    }
}

struct Else {
    warnings: Vec<String>,
}

impl From<Vec<String>> for Else {
    fn from(value: Vec<String>) -> Self {
        Self { warnings: value }
    }
}

impl DirectiveMapper for Else {
    fn map<'l>(&self, line: Directive<'l>, depth: usize) -> Directive<'l> {
        match (self.warnings.is_empty(), line) {
            // has warnings and asked to display them
            (false, Directive::DisplayWarnings) => Directive::Multiple(
                self.warnings
                    .iter()
                    .map(|warning| Directive::Warning(warning.clone().into()))
                    .collect(),
            ),

            // has warnings and any other directive
            (false, other) => other,

            // no warnings and close, forward to avoid close being ignored
            // TODO: disconnect manual close drirective from Directive::Close
            // to prevent this happening from manual close directives
            (true, Directive::Close) => Directive::Close,

            // no warnings and end, forward if and only if depth is 0 (we are top map)
            // to ensure this map will be removed
            (true, directive @ Directive::EndMap { .. }) if depth == 0 => directive,

            // no warnings and any other directive, everything becomes noop
            (true, _) => Directive::Noop,
        }
    }

    fn name(&self) -> &str {
        "Else"
    }
}

fn directive_debug(line: Directive<'_>) -> Directive<'_> {
    eprintln!("{line:#?}");
    line
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
        title: &mut Option<String>,
    ) -> Result<SourceAction> {
        let shallow = source.shallow();
        let Source {
            read,
            ref path,
            cmd,
            line_map,
            ref warning_watcher,
            ..
        } = source;

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
            line_map.apply(directive)
        } else {
            directive
        };

        match directive {
            Directive::Noop | Directive::Comment(..) => {}
            Directive::Close => {
                return Ok(SourceAction::Pop);
            }
            Directive::Clean => {
                *cmd = Arc::default();
            }
            Directive::Exe(exe) => {
                cmd.write().unwrap().exe(PathBuf::from(exe.as_ref()));
            }
            Directive::Arg(arg) => {
                cmd.write().unwrap().arg(arg.into());
            }
            Directive::Watch => {
                let is_sleeping = warning_watcher.borrow().is_sleeping();
                if is_sleeping {
                    warning_watcher.borrow_mut().watch();
                } else {
                    lines.push_warning(
                        "watch called multiple times before else or then block".into(),
                    );
                }
            }
            Directive::Then => {
                if let Watch::Watching { occured } =
                    std::mem::take(&mut *warning_watcher.borrow_mut())
                {
                    let prev = line_map.take();
                    *line_map = Some(DirectiveMapperChain::new(Then::from(occured), prev, false));
                } else {
                    lines.push_warning(
                        "then blocks need to be placed somewhere after a watch directive".into(),
                    );
                }
            }
            Directive::Else => {
                if let Watch::Watching { occured } =
                    std::mem::take(&mut *warning_watcher.borrow_mut())
                {
                    let prev = line_map.take();
                    *line_map = Some(DirectiveMapperChain::new(Else::from(occured), prev, false));
                } else {
                    lines.push_warning(
                        "else blocks need to be placed somewhere after a watch directive".into(),
                    );
                }
            }
            Directive::DisplayWarnings => {
                lines.push_warning("warnings can only be displayed in else blocks".into());
            }
            Directive::IgnoreWarnings => {
                fn ignore_warnings(directive: Directive<'_>) -> Directive<'_> {
                    match directive {
                        Directive::Warning(..) => Directive::Noop,
                        other => other,
                    }
                }
                let prev = line_map.take();
                *line_map = Some(DirectiveMapperChain::new(ignore_warnings, prev, false));
            }
            Directive::IgnoreText => {
                fn ignore_text(directive: Directive<'_>) -> Directive<'_> {
                    match directive {
                        Directive::Text(..) => Directive::Noop,
                        other => other,
                    }
                }
                let prev = line_map.take();
                *line_map = Some(DirectiveMapperChain::new(ignore_text, prev, false));
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
                if title.is_none() {
                    *title = Some(text.into());
                }
            }
            Directive::Subtitle(text) => {
                lines.push_subtitle(text);
            }
            Directive::Import(import) => match import.perform_import(shallow.shallow(), imported) {
                Ok(source) => {
                    return Ok(SourceAction::Push(source));
                }
                Err(directive) => {
                    read.push(position, directive);
                }
            },
            Directive::Empty => lines.push_empty(),
            Directive::Text(text) => lines.push_line(text),

            Directive::Multiple(parses) => {
                for directive in parses.into_iter().rev() {
                    read.push(position, directive);
                }
            }
            Directive::Debug => {
                let prev = line_map.take();
                *line_map = Some(DirectiveMapperChain::new(directive_debug, prev, false));
            }
        };

        Ok(SourceAction::Noop)
    }
}
