use std::fmt::Debug;

use crate::Result;

pub trait LineRead: Debug {
    fn read(&mut self) -> Result<(usize, ParsedLine<'_>)>;
}

#[derive(Debug, Clone, Default)]
pub enum ParsedLine<'s> {
    #[default]
    Empty,
    End,
    Text(&'s str),
    Warning(String),
}

impl<'s> ParsedLine<'s> {
    pub fn parse(text: &'s str) -> Self {
        let text = text.trim_end();
        if text.is_empty() {
            Self::Empty
        } else {
            Self::Text(text)
        }
    }
}
