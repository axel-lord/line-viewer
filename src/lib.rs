mod error;
mod line_view;
mod path_ext;

pub use self::{
    error::Error,
    line_view::{
        directive::Directive,
        file_reader::FileReader,
        line::{Line, Source as LineSource},
        line_read::LineRead,
        source::Source,
        source_action::SourceAction,
        LineView,
    },
    path_ext::PathExt,
};

pub type Result<T = ()> = std::result::Result<T, Error>;
