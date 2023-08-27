use crate::line_view::LineView;

#[derive(Clone, Debug)]
pub struct History {
    pos: usize,
    history: Vec<LineView>,
}

impl FromIterator<LineView> for History {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = LineView>,
    {
        let mut history = iter.into_iter().collect::<Vec<_>>();

        if history.is_empty() {
            history.push(LineView::default());
        }

        Self {
            pos: history.len().saturating_sub(1),
            history,
        }
    }
}

impl History {
    pub fn has_past(&self) -> bool {
        self.pos > 0
    }

    pub fn has_future(&self) -> bool {
        self.pos < self.history.len().saturating_sub(1)
    }

    pub fn current(&self) -> LineView {
        self.history[self.pos].clone()
    }

    pub fn undo(&mut self) -> LineView {
        if self.has_past() {
            self.pos -= 1
        }

        self.current()
    }

    pub fn redo(&mut self) -> LineView {
        if self.has_future() {
            self.pos += 1
        }

        self.current()
    }

    pub fn soft_reset(&mut self) -> LineView {
        self.pos = 0;

        self.current()
    }

    #[allow(dead_code)]
    pub fn push(&mut self, content: LineView) -> &mut Self {
        self.history.resize_with(self.pos + 1, || {
            panic!(
                "removing future of history should always shrink it, or leave it be, not grow it"
            )
        });

        self.history.push(content);
        self.pos += 1;

        self
    }
}
