use std::sync::Arc;

use crate::{
    line_view::{import, line, Directive, PathSet, Source},
    Line, ParsedLine, Result,
};

#[derive(Debug)]
pub enum SourceAction {
    Noop,
    Pop,
    Push(Source),
}

impl SourceAction {
    pub fn perform(
        Source {
            read,
            ref path,
            cmd,
            sourced,
            ref dir,
            ref is_root,
            ref line_map,
        }: &mut Source,
        imported: &mut PathSet,
        lines: &mut Vec<Line>,
        title: &mut String,
    ) -> Result<SourceAction> {
        // makes use of bools easier
        let is_root = *is_root;

        // read line
        let (position, parsed_line) = read.read()?;

        // shared start of builder
        let builder = || line::Builder::new().source(path.into()).position(position);
        let push_warning = |lines: &mut Vec<Line>, text| {
            lines.push(builder().warning().text(text).build());
        };
        let push_subtitle = |lines: &mut Vec<Line>, text| {
            lines.push(builder().title().text(text).build());
        };
        let push_line = |lines: &mut Vec<Line>, text| {
            lines.push(builder().text(text).cmd(Arc::clone(cmd)).build());
        };

        // apply maps in reverse order
        let parsed_line = if let Some(line_map) = line_map.as_ref() {
            let mut parsed_line = parsed_line;
            for line_map in line_map {
                parsed_line = line_map.map(parsed_line);
            }
            parsed_line
        } else {
            parsed_line
        };

        match dbg!(parsed_line) {
            ParsedLine::None | ParsedLine::Comment(_) => {}
            ParsedLine::Empty => {
                lines.push(builder().build());
            }
            ParsedLine::End => {
                return Ok(SourceAction::Pop);
            }
            ParsedLine::Warning(s) => {
                push_warning(lines, s.to_string());
            }
            ParsedLine::Directive(directive) => match directive {
                Directive::Clean => {
                    *cmd = Arc::default();
                }
                Directive::Prefix(pre) => {
                    cmd.write().unwrap().pre(pre);
                }
                Directive::Suffix(suf) => {
                    cmd.write().unwrap().suf(suf);
                }
                Directive::Warning(warn) => {
                    push_warning(lines, warn.into());
                }
                Directive::Title(text) => {
                    if is_root {
                        *title = text.into();
                    }
                }
                Directive::Subtitle(text) => {
                    push_subtitle(lines, text.into());
                }
                Directive::Import(import) => {
                    return Ok(SourceAction::Push(
                        match import.perform_import(import::ImportCtx {
                            is_root,
                            dir,
                            cmd,
                            sourced,
                            imported,
                        }) {
                            Ok(source) => source,
                            Err(_directive) => {
                                Source::one_shot(path.to_path_buf(), position, ParsedLine::None)
                            }
                        },
                    ));
                }
            },
            ParsedLine::Text(line) => {
                push_line(lines, line.into());
            }
        };

        Ok(SourceAction::Noop)
    }
}
