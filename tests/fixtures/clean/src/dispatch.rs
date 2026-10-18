use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::Error;
use crate::error::InternalSnafu;

pub fn execute(command: &str) -> Result<(), Error> {
    catch_unwind(AssertUnwindSafe(|| run(command))).unwrap_or_else(|_| {
        let what = rsb_errors::panic::take().map_or_else(
            || "panic with no recorded location".to_owned(),
            |p| p.to_string(),
        );
        InternalSnafu { what }.fail()
    })
}

fn run(_command: &str) -> Result<(), Error> {
    crate::step()
}
