use crate::{LineRead, ParsedLine, Result};
use std::{
    fmt::Debug,
    io::{BufRead, BufReader, Read},
};

#[derive(Debug)]
pub struct FileReader<R>(BufReader<R>, usize, String);

impl<R> FileReader<R>
where
    R: Read,
{
    pub fn new(read: R) -> Self {
        Self(BufReader::new(read), 0, String::new())
    }
}

impl<R> LineRead for FileReader<R>
where
    R: Debug + Read,
{
    fn read(&mut self) -> Result<(usize, ParsedLine<'_>)> {
        let Self(read, pos, buf) = self;

        let pos = {
            *pos += 1;
            *pos - 1
        };

        buf.clear();
        if read.read_line(buf)? == 0 {
            return Ok((pos, ParsedLine::Close));
        }

        Ok((pos, ParsedLine::parse_line(buf)))
    }
}
