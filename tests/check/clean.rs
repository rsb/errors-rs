//! No finding where there is nothing to find.

use std::path::Path;

use rsb_errors::check::{Config, run};

use crate::findings;

#[test]
fn a_clean_repository_has_no_finding() {
    assert_eq!(
        findings("clean"),
        Vec::<String>::new(),
        "every exemption, a doc example, a string, a raw string and a char literal"
    );
}

#[test]
fn an_unreadable_file_is_a_finding() {
    assert_eq!(
        findings("unreadable"),
        ["src/lib.rs:0: unreadable: stream did not contain valid UTF-8"],
        "line 0 names the whole file"
    );
}

#[test]
fn this_repository_has_no_finding() {
    let config = Config {
        dispatcher: "src/lib.rs".into(),
        boundary: "src/lib.rs".into(),
        ignore: vec!["tests/fixtures".into()],
    };
    let found: Vec<String> = run(Path::new(env!("CARGO_MANIFEST_DIR")), &config)
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        found,
        Vec::<String>::new(),
        "this crate meets its own standards"
    );
}
