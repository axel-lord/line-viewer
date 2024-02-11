use std::fmt::Debug;

use crate::{Directive, Result};

pub trait LineRead: Debug {
    fn read(&mut self) -> Result<(usize, Directive<'_>)>;
}
