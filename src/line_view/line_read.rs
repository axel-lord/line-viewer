use std::{borrow::Cow, fmt::Debug};

use crate::{line_view::directive::Directive, Result};

pub trait LineRead: Debug {
    fn read(&mut self) -> Result<(usize, ParsedLine<'_>)>;
}

#[derive(Debug, Clone, Default)]
pub enum ParsedLine<'s> {
    #[default]
    None,
    Empty,
    Close,
    Comment(Cow<'s, str>),
    Text(Cow<'s, str>),
    Warning(Cow<'s, str>),
    Multiple(Vec<ParsedLine<'static>>), // too complicated for me to prove lifetimes
    Directive(Directive<'s>),
}

impl<'s> ParsedLine<'s> {
    pub fn parse_line(text: &'s str) -> Self {
        let text = text.trim_end();
        if text.is_empty() {
            Self::Empty
        } else if let Some(directive) = text.strip_prefix("#-") {
            Self::Directive(Directive::parse_str(directive.trim_end()))
        } else if text.starts_with("##") {
            Self::Text(Cow::Borrowed(&text[1..]))
        } else if let Some(text) = text.strip_prefix('#') {
            Self::Comment(text.trim_start().into())
        } else {
            Self::Text(text.into())
        }
    }
}
