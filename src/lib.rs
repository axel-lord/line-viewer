mod error;
mod line_view;
mod path_ext;

pub use self::{
    error::Error,
    line_view::{
        line::{Line, Source as LineSource},
        FileReader, LineRead, LineView,
    },
    path_ext::PathExt,
};

pub type Result<T = ()> = std::result::Result<T, Error>;
