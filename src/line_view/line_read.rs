use std::fmt::Debug;

use crate::{Directive, Result};

pub trait LineRead: Debug {
    fn read(&mut self) -> Result<(usize, Directive<'_>)>;
}

#[derive(Debug)]
struct Fused<T> {
    line_read: T,
    empty: Option<usize>,
}

impl<T> From<T> for Fused<T> where T: LineRead {
    fn from(value: T) -> Self {
        Fused { line_read: value, empty: None }
    }
}

impl<T> LineRead for Fused<T> where T: LineRead {
    fn read(&mut self) -> Result<(usize, Directive<'_>)> {
        let Self { line_read, empty } = self;
        if let Some(size) = *empty {
            Ok((size, Directive::Empty))
        } else {
            match line_read.read() {
                Ok((size, Directive::Close)) => {
                    *empty = Some(size);
                    Ok((size, Directive::Close))
                }
                other => other,
            }
        }
    }
}

struct LineReaderInner<LR: ?Sized> {
    stack: Vec<(usize, Directive<'static>)>,
    debug: fn() -> &'static str,
    line_read: LR,
}

impl<T: ?Sized> Debug for LineReaderInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (self.debug)())
    }
}

pub struct LineReader {
    this: Box<LineReaderInner<dyn LineRead>>,
}

impl LineReader {
    pub fn new<LR>(line_read: LR) -> Self
    where
        LR: 'static + LineRead,
    {
        let line_read = Fused::from(line_read);
        let this = Box::new(LineReaderInner {
            stack: Vec::new(),
            debug: || std::any::type_name::<LR>(),
            line_read,
        });

        Self { this }
    }

    pub fn push(&mut self, line_nr: usize, directive: Directive<'static>) -> &mut Self {
        self.this.stack.push((line_nr, directive));
        self
    }
}

impl Debug for LineReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynLineRead")
            .field("line_read", &self.this)
            .finish_non_exhaustive()
    }
}

impl LineRead for LineReader {
    fn read(&mut self) -> Result<(usize, Directive<'_>)> {
        if let Some(res) = self.this.stack.pop() {
            Ok(res)
        } else {
            self.this.line_read.read()
        }
    }
}
