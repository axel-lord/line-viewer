mod error;
mod line_view;

pub use self::{
    error::Error,
    line_view::{LineView, line::Line},
};

pub type Result<T = ()> = std::result::Result<T, Error>;
