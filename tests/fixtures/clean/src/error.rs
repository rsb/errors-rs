use std::error::Error as _;

use rsb_errors::Loc;
use snafu::prelude::*;

/// The crate's one error enum.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// A doc comment inside the enum is masked.
    #[snafu(display("open raw{location}: {source}"))]
    OpenRaw {
        source: std::io::Error,
        #[snafu(implicit)]
        location: Loc,
    },
    #[snafu(display("decode {path}{location}: {source}"))]
    Decode {
        path: std::path::PathBuf,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<Error>,
        #[snafu(implicit)]
        location: rsb_errors::Loc,
    },
    #[snafu(display("too small: {width}"))]
    TooSmall { width: u32 },
    #[snafu(display("internal error: {what}{location}"))]
    Internal {
        what: String,
        #[snafu(implicit)]
        location: Loc,
    },
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Error::OpenRaw { .. } => "open_raw",
            Error::Decode { .. } => "decode",
            Error::TooSmall { .. } => "too_small",
            Error::Internal { .. } => "internal",
        }
    }

    pub fn root(&self) -> String {
        self.source().map_or_else(|| self.to_string(), |s| s.to_string())
    }
}
