use std::{borrow::Cow, fmt::Debug};

use crate::Result;

pub trait LineRead: Debug {
    fn read(&mut self) -> Result<(usize, ParsedLine<'_>)>;
}

#[derive(Debug, Clone, Default)]
pub enum ParsedLine<'s> {
    #[default]
    None,
    Empty,
    End,
    Comment(&'s str),
    Text(&'s str),
    Warning(Cow<'s, str>),
    Directive(&'s str),
}

impl<'s> ParsedLine<'s> {
    pub fn parse_line(text: &'s str) -> Self {
        let text = text.trim_end();
        if text.is_empty() {
            Self::Empty
        } else if let Some(directive) = text.strip_prefix("#-") {
            Self::Directive(directive.trim_end())
        } else if text.starts_with("##") {
            Self::Text(&text[1..])
        } else if let Some(text) = text.strip_prefix('#') {
            Self::Comment(text.trim())
        } else {
            Self::Text(text)
        }
    }

    pub fn parse_directive(_directive: &'s str) -> Self {
        todo!()
    }
}
