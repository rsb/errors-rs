//! The golden-line test every rsb crate carries, written against the crate
//! that defines `Loc`: the default line has no locations, the debug line has
//! one per layer. One test function, because `show_locations` is process-wide
//! and integration test files are separate processes.

use rsb_errors::Loc;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum Inner {
    #[snafu(display("read settings{location}: {source}"))]
    Read {
        source: std::io::Error,
        #[snafu(implicit)]
        location: Loc,
    },
}

#[derive(Debug, Snafu)]
enum Outer {
    #[snafu(display("open photo{location}: {source}"))]
    Open {
        source: Inner,
        #[snafu(implicit)]
        location: Loc,
    },
}

fn deepest() -> Result<(), Outer> {
    let io = Err::<(), _>(std::io::Error::from(std::io::ErrorKind::NotFound));
    io.context(ReadSnafu).context(OpenSnafu)
}

#[test]
fn one_line_by_default_and_a_location_per_layer_when_shown() {
    let e = deepest().unwrap_err();
    assert_eq!(
        e.to_string(),
        "open photo: read settings: entity not found",
        "the approved line"
    );

    rsb_errors::show_locations(true);
    let shown = e.to_string();
    assert_eq!(
        shown.matches(" (at ").count(),
        2,
        "one location per layer: {shown}"
    );
    assert!(
        shown.contains("tests/error_line.rs:"),
        "locations name this file: {shown}"
    );
}

#[test]
fn a_panic_is_reported_with_its_location() {
    rsb_errors::panic::install();
    let caught = std::panic::catch_unwind(|| {
        let empty: Vec<u8> = Vec::new();
        empty[3]
    });
    assert!(caught.is_err(), "the closure panics");
    let report = rsb_errors::panic::take().expect("the hook recorded the panic");
    assert_eq!(
        report.message,
        "index out of bounds: the len is 0 but the index is 3"
    );
    assert!(
        report.to_string().contains("tests/error_line.rs:"),
        "{report}"
    );
    assert_eq!(rsb_errors::panic::take(), None, "take clears the slot");
}
