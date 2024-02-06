use std::fmt::Debug;

use crate::Result;

#[derive(Debug, Clone, Default)]
pub enum ParsedLine<'s> {
    #[default]
    Empty,
    End,
    Text(&'s str),
    Warning(String),
}

pub trait LineRead: Debug {
    fn read(&mut self) -> Result<(usize, ParsedLine<'_>)>;
}
