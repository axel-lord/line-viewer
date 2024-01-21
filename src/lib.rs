mod line_view;
mod error;

pub use line_view::{LineView, Cmd as Cmd, Handle as LineHandle};
pub use error::Error;

pub type Result<T = ()> = std::result::Result<T, Error>;
