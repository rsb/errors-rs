//! One planted-hit fixture per rule under `tests/fixtures/`, each a small
//! repository that is clean except for the hit, so a test asserts the
//! exact finding lines. `clean` carries every exemption and no hit, and
//! `own` runs the check on this repository.

mod clean;
mod errors;
mod logging;
mod panics;

use std::path::Path;

use rsb_errors::check::{Config, run};

/// The dispatcher and boundary every fixture uses.
pub fn config() -> Config {
    Config {
        dispatcher: "src/dispatch.rs".into(),
        boundary: "src/boundary.rs".into(),
        ignore: Vec::new(),
    }
}

/// The finding lines for one fixture tree.
pub fn findings(fixture: &str) -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture);
    run(&root, &config())
        .iter()
        .map(ToString::to_string)
        .collect()
}
