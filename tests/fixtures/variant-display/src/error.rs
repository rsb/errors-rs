use rsb_errors::Loc;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("open raw{location}: {source}"))]
    Open {
        source: std::io::Error,
        #[snafu(implicit)]
        location: Loc,
    },
    #[snafu(display("decode {path}{location}"))]
    Decode {
        path: String,
        source: std::io::Error,
        #[snafu(implicit)]
        location: Loc,
    },
    Unpack {
        source: std::io::Error,
        #[snafu(implicit)]
        location: Loc,
    },
    #[snafu(display("too small: {width}"))]
    TooSmall { width: u32 },
}
