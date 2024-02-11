use std::{any, fmt::Debug, ops::Deref, rc::Rc};

use crate::Directive;

pub trait LineMap {
    fn map<'l>(&self, line: Directive<'l>) -> Directive<'l>;
    fn name(&self) -> &str;
}

impl<F> LineMap for F
where
    F: Fn(Directive) -> Directive,
{
    fn map<'line>(&self, line: Directive<'line>) -> Directive<'line> {
        self(line)
    }
    fn name(&self) -> &str {
        any::type_name::<F>()
    }
}

struct LMNodeI<LM: ?Sized> {
    pub prev: Option<Rc<LMNodeI<dyn LineMap>>>,
    pub line_map: LM,
}

#[derive(Clone)]
pub struct LineMapNode {
    this: Rc<LMNodeI<dyn LineMap>>,
}

impl LineMapNode {
    pub fn new<LM>(line_map: LM, prev: Option<Self>) -> Self
    where
        LM: LineMap + 'static,
    {
        let this = Rc::new(LMNodeI {
            prev: prev.map(|p| p.this),
            line_map,
        });

        Self { this }
    }

    pub fn prev(&self) -> Option<Self> {
        self.this
            .prev
            .as_ref()
            .map(|p| LineMapNode { this: Rc::clone(p) })
    }
}

#[derive(Debug)]
pub struct Iter(Option<LineMapNode>);
impl Iterator for Iter {
    type Item = LineMapNode;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.0.take() {
            self.0 = next.prev();
            Some(next)
        } else {
            None
        }
    }
}

impl IntoIterator for LineMapNode {
    type Item = LineMapNode;

    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter(Some(self))
    }
}

impl IntoIterator for &LineMapNode {
    type Item = LineMapNode;

    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.clone().into_iter()
    }
}

impl Deref for LineMapNode {
    type Target = dyn LineMap;

    fn deref(&self) -> &Self::Target {
        &self.this.line_map
    }
}

impl AsRef<dyn LineMap + 'static> for LineMapNode {
    fn as_ref(&self) -> &(dyn LineMap + 'static) {
        &self.this.line_map
    }
}

impl Debug for LineMapNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineMapNode")
            .field("line_map", &self.name())
            .field("prev", &self.prev())
            .finish()
    }
}
