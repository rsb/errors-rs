use snafu::{Report, ResultExt, Whatever};
use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(context(false))]
    #[snafu(display("io{location}: {source}"))]
    Io {
        source: std::io::Error,
        #[snafu(implicit)]
        location: rsb_errors::Loc,
    },
}

fn load() -> Result<(), Whatever> {
    whatever!("no typed error here")
}

fn main() -> snafu::Report<Error> {
    snafu::Report::capture(|| Ok(()))
}
