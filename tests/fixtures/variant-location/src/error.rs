use rsb_errors::Loc;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("open raw{location}: {source}"))]
    Open {
        source: std::io::Error,
        #[snafu(implicit)]
        location: rsb_errors::Loc,
    },
    #[snafu(display("decode: {source}"))]
    Decode { source: std::io::Error },
    #[snafu(display("unpack: {source}"))]
    Unpack {
        source: std::io::Error,
        location: Loc,
    },
    #[snafu(display("process: {source}"))]
    Process {
        source: std::io::Error,
        #[snafu(implicit)]
        location: snafu::Location,
    },
}
