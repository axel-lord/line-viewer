use std::{borrow::Cow, char};

use crate::line_view::import::Import;

#[derive(Debug, Clone)]
pub enum Directive<'l> {
    Clean,
    Empty,
    Prefix(Cow<'l, str>),
    Suffix(Cow<'l, str>),
    Warning(Cow<'l, str>),
    Title(Cow<'l, str>),
    Subtitle(Cow<'l, str>),
    Text(Cow<'l, str>),
    Import(Import<'l>),
}

impl<'l> Directive<'l> {
    fn parse_str_result(text: &'l str) -> Result<Self, Cow<'l, str>> {
        let mut split = text.trim_start().splitn(2, char::is_whitespace);

        let Some(directive) = split.next() else {
            return Err(format!("could not parse directive \"{text}\"").into());
        };
        let payload = split.next();

        let require_payload = move |directive| {
            payload
                .map(|payload| {
                    let payload = payload.trim();
                    payload
                        .strip_prefix('"')
                        .and_then(|payload| payload.strip_suffix('"'))
                        .unwrap_or(payload)
                })
                .ok_or_else(|| Cow::Owned(format!("directive {directive} requires an argument")))
        };

        Ok(match directive {
            "pre" => Self::Prefix(require_payload("pre")?.into()),

            "suf" => Self::Suffix(require_payload("suf")?.into()),

            "clean" => Self::Clean,

            "title" => Self::Title(require_payload("title")?.into()),

            "subtitle" => Self::Subtitle(require_payload("subtitle")?.into()),

            "import" => Self::Import(Import::new_import(require_payload("import")?)),

            "lines" => Self::Import(Import::new_lines(require_payload("lines")?)),

            "source" => Self::Import(Import::new_source(require_payload("source")?)),

            "warning" => Self::Warning(require_payload("warning")?.into()),

            "text" => Self::Text(require_payload("text")?.into()),

            "empty" => Self::Empty,

            other => {
                return Err(format!("{other} is not a directive").into());
            }
        })
    }
    pub fn parse_str(text: &'l str) -> Self {
        match Self::parse_str_result(text) {
            Err(warn) => Self::Warning(warn),
            Ok(directive) => directive,
        }
    }
}
