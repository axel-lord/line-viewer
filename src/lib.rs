mod error;
mod line_view;

pub use self::{
    error::Error,
    line_view::{cmd::Cmd, handle::Handle as LineHandle, LineView},
};

pub type Result<T = ()> = std::result::Result<T, Error>;
