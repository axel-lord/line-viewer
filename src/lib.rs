mod error;
mod line_view;
mod path_ext;

pub use self::{
    error::Error,
    line_view::{LineView, line::{Line, Source as LineSource}},
    path_ext::PathExt,
};

pub type Result<T = ()> = std::result::Result<T, Error>;
