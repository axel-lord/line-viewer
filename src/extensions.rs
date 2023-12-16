use iced::{
    widget::{Column, Row},
    Element,
};

pub trait ColRowExt<'a, M> {
    fn push_if<IntoElem>(self, condition: bool, elem: impl FnOnce() -> IntoElem) -> Self
    where
        IntoElem: Into<Element<'a, M>>,
        Self: Sized,
    {
        self.push_maybe(condition.then(elem))
    }

    fn push_if_else<IntoElem, IntoElemDefault>(
        self,
        condition: bool,
        elem: impl FnOnce() -> IntoElem,
        default: impl FnOnce() -> IntoElemDefault,
    ) -> Self
    where
        IntoElem: Into<Element<'a, M>>,
        IntoElemDefault: Into<Element<'a, M>>,
        Self: Sized,
    {
        self.push_maybe(condition.then(elem))
            .push_maybe((!condition).then(default))
    }

    fn push_maybe(self, elem: Option<impl Into<Element<'a, M>>>) -> Self;
}

pub trait TapIf {
    fn tap_if<F>(self, cond: bool, action: F) -> Self
    where
        Self: Sized,
        F: FnOnce(Self) -> Self,
    {
        if cond {
            action(self)
        } else {
            self
        }
    }

    fn tap_if_else<T, F, G>(self, cond: bool, action: F, default: G) -> T
    where
        Self: Sized,
        F: FnOnce(Self) -> T,
        G: FnOnce(Self) -> T,
    {
        if cond {
            action(self)
        } else {
            default(self)
        }
    }
}

impl<T> TapIf for T {}

impl<'a, M> ColRowExt<'a, M> for Column<'a, M> {
    fn push_maybe(self, elem: Option<impl Into<Element<'a, M>>>) -> Self {
        if let Some(elem) = elem {
            self.push(elem)
        } else {
            self
        }
    }
}

impl<'a, M> ColRowExt<'a, M> for Row<'a, M> {
    fn push_maybe(self, elem: Option<impl Into<Element<'a, M>>>) -> Self {
        if let Some(elem) = elem {
            self.push(elem)
        } else {
            self
        }
    }
}
