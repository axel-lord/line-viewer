use std::{collections::VecDeque, fmt::Debug};

use crate::{Directive, Result};

pub trait LineRead: Debug {
    fn read(&mut self) -> Result<(usize, Directive<'_>)>;
}

struct DynLineReadInt<LR: ?Sized> {
    queue: VecDeque<(usize, Directive<'static>)>,
    debug: fn() -> &'static str,
    line_read: LR,
}

impl<T: ?Sized> Debug for DynLineReadInt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (self.debug)())
    }
}

pub struct DynLineRead {
    this: Box<DynLineReadInt<dyn LineRead>>,
}

impl DynLineRead {
    pub fn new<LR>(line_read: LR) -> Self
    where
        LR: 'static + LineRead,
    {
        let this = Box::new(DynLineReadInt {
            queue: VecDeque::new(),
            debug: || {
                std::any::type_name::<LR>()
            },
            line_read,
        });

        Self { this }
    }

    pub fn enqueue(&mut self, line_nr: usize, directive: Directive<'static>) -> &mut Self {
        self.this.queue.push_back((line_nr, directive));
        self
    }
}

impl Debug for DynLineRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynLineRead").field("line_read", &self.this).finish_non_exhaustive()
    }
}

impl LineRead for DynLineRead {
    fn read(&mut self) -> Result<(usize, Directive<'_>)> {
        if let Some(res) = self.this.queue.pop_front() {
            Ok(res)
        } else {
            self.this.line_read.read()
        }
    }
}
