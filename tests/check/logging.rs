//! The Check section of `logging.md`.

use crate::findings;

#[test]
fn printing_and_the_log_crate_stay_in_main_and_tests() {
    assert_eq!(
        findings("print"),
        [
            "src/lib.rs:1: print: log crate outside main and tests",
            "src/lib.rs:4: print: println! outside main and tests",
            "src/lib.rs:8: print: eprintln! outside main and tests",
            "src/lib.rs:12: print: log crate outside main and tests",
        ],
        "main.rs, src/bin and tests are exempt"
    );
}

#[test]
fn warn_and_error_stay_in_the_boundary() {
    assert_eq!(
        findings("log-level"),
        [
            "src/lib.rs:4: log-level: warn! outside the boundary src/boundary.rs",
            "src/lib.rs:9: log-level: error! outside the boundary src/boundary.rs",
        ],
        "the boundary module is exempt"
    );
}

#[test]
fn instrument_err_fails_in_every_form() {
    assert_eq!(
        findings("instrument-err"),
        [
            "src/lib.rs:3: instrument-err: #[instrument(err)]",
            "src/lib.rs:6: instrument-err: #[instrument(err)]",
            "src/lib.rs:9: instrument-err: #[instrument(err)]",
            "src/lib.rs:15: instrument-err: #[instrument(err)]",
        ],
        "bare, by path with other arguments, err(Debug), and across lines"
    );
}
