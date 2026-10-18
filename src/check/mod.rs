//! The repository checks named by the Check sections of
//! `rules/standards/errors.md`, `panics.md` and `logging.md`. Each
//! consumer's `cargo xtask errors` calls [`run`] on its workspace root and
//! exits 1 on any finding.
//!
//! The checks are text searches over every `.rs` and `Cargo.toml` file
//! under the root, skipping `target`, hidden directories and
//! [`Config::ignore`]. Comments and string literals are masked before a
//! pattern is looked for, so a doc example or a message is never a hit.
//! There is no parser; what the search misses, review catches.
//!
//! | Rule | Fails on |
//! |---|---|
//! | `banned-crate` | a `Cargo.toml` listing `anyhow`, `eyre`, `color-eyre`, `failure`, `error-chain` or `thiserror` |
//! | `boxed-error` | `Box<dyn ... Error ...>` |
//! | `variant-display` | a snafu variant with a `source` field and no display string, or one without `{source}` |
//! | `variant-location` | a snafu variant with a `source` field and no `#[snafu(implicit)] location: Loc` |
//! | `untyped-snafu` | `context(false)`, `Whatever`, `whatever!` or `snafu::Report` |
//! | `source-call` | `.source()` outside `error.rs` and tests |
//! | `release-profile` | a root `Cargo.toml` whose `[profile.release]` does not state `panic = "unwind"` and `overflow-checks = true` |
//! | `bare-panic` | `panic!()`, `unreachable!()` or `expect("")` outside tests |
//! | `catch-unwind` | a `catch_unwind` outside the dispatcher, an `extern "C" fn` body and tests |
//! | `extern-c` | an `extern "C" fn` whose first statement does not call `catch_unwind` |
//! | `print` | `println!`, `eprintln!` or the `log` crate outside `main` and tests |
//! | `log-level` | `warn!` or `error!` outside the boundary module |
//! | `instrument-err` | `#[instrument(err)]` in any form |
//! | `unreadable` | a file or directory that could not be read |
//!
//! "Tests" are files under a `tests` directory and lines after the first
//! `#[cfg(test)]`; "`main`" is `main.rs` and `src/bin/`; "`error.rs`" is a
//! file of that name or anything under a directory named `error`.
//!
//! ```no_run
//! use std::path::Path;
//! use std::process::ExitCode;
//!
//! use rsb_errors::check::{Config, run};
//!
//! fn main() -> ExitCode {
//!     let config = Config {
//!         dispatcher: "crates/engine/src/lib.rs".into(),
//!         boundary: "crates/engine/src/lib.rs".into(),
//!         ignore: vec!["crates/ffi/vendor".into()],
//!     };
//!     let findings = run(Path::new("."), &config);
//!     for finding in &findings {
//!         eprintln!("{finding}");
//!     }
//!     if findings.is_empty() {
//!         ExitCode::SUCCESS
//!     } else {
//!         ExitCode::FAILURE
//!     }
//! }
//! ```

mod errors;
mod logging;
mod manifest;
mod panics;
mod source;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use source::Source;

/// Where a repository allows what the rules otherwise forbid. Every path
/// is relative to the root passed to [`run`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    /// The dispatcher file: the one place a `catch_unwind` is allowed
    /// outside an `extern "C"` callback and tests.
    pub dispatcher: PathBuf,
    /// The boundary module, a file or a directory: the one place `warn!`
    /// and `error!` are allowed.
    pub boundary: PathBuf,
    /// Files or directories the check does not enter: vendored code, or
    /// planted fixtures.
    pub ignore: Vec<PathBuf>,
}

/// The rule a finding breaks, named as it appears in the finding line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    /// A `Cargo.toml` lists a crate the errors standard bans.
    BannedCrate,
    /// A line has `Box<dyn ... Error ...>`.
    BoxedError,
    /// A snafu variant with a `source` field has no display string, or
    /// one without `{source}`.
    VariantDisplay,
    /// A snafu variant with a `source` field lacks
    /// `#[snafu(implicit)] location: Loc`.
    VariantLocation,
    /// `context(false)`, `Whatever`, `whatever!` or `snafu::Report`.
    UntypedSnafu,
    /// `.source()` outside `error.rs` and tests.
    SourceCall,
    /// The root `Cargo.toml` has no `[profile.release]` stating
    /// `panic = "unwind"` and `overflow-checks = true`.
    ReleaseProfile,
    /// `panic!()`, `unreachable!()` or `expect("")` with no message,
    /// outside tests.
    BarePanic,
    /// A `catch_unwind` outside the dispatcher, an `extern "C" fn` body
    /// and tests.
    CatchUnwind,
    /// An `extern "C" fn` whose first statement does not call
    /// `catch_unwind`.
    ExternC,
    /// `println!`, `eprintln!` or the `log` crate outside `main` and
    /// tests.
    Print,
    /// `warn!` or `error!` outside the boundary module.
    LogLevel,
    /// `#[instrument(err)]` in any form.
    InstrumentErr,
    /// A file or directory the check could not read.
    Unreadable,
}

impl Rule {
    /// The rule's name in a finding line.
    pub fn name(self) -> &'static str {
        match self {
            Rule::BannedCrate => "banned-crate",
            Rule::BoxedError => "boxed-error",
            Rule::VariantDisplay => "variant-display",
            Rule::VariantLocation => "variant-location",
            Rule::UntypedSnafu => "untyped-snafu",
            Rule::SourceCall => "source-call",
            Rule::ReleaseProfile => "release-profile",
            Rule::BarePanic => "bare-panic",
            Rule::CatchUnwind => "catch-unwind",
            Rule::ExternC => "extern-c",
            Rule::Print => "print",
            Rule::LogLevel => "log-level",
            Rule::InstrumentErr => "instrument-err",
            Rule::Unreadable => "unreadable",
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One hit: the file, the line, the rule and what was found.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// Relative to the root passed to [`run`].
    pub path: PathBuf,
    /// 1-based; 0 when the finding is about the whole file.
    pub line: usize,
    /// The rule broken.
    pub rule: Rule,
    /// What was found, in the rule's words.
    pub what: String,
}

impl fmt::Display for Finding {
    /// One line: `path:line: rule: what`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}: {}",
            self.path.display(),
            self.line,
            self.rule,
            self.what
        )
    }
}

/// Every finding under `root`, sorted by path, line and rule. A file or
/// directory that cannot be read is a finding, so the caller always gets
/// a list and exits 1 when it is not empty.
pub fn run(root: &Path, config: &Config) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut files = Vec::new();
    walk(root, root, config, &mut files, &mut out);
    for path in files {
        let text = match fs::read_to_string(root.join(&path)) {
            Ok(text) => text,
            Err(e) => {
                out.push(unreadable(path, &e));
                continue;
            }
        };
        if path.file_name().is_some_and(|name| name == "Cargo.toml") {
            manifest::check(&path, &text, &mut out);
        } else {
            let source = Source::new(path, text);
            errors::check(&source, &mut out);
            panics::check(&source, config, &mut out);
            logging::check(&source, config, &mut out);
        }
    }
    out.sort();
    out
}

/// Every `.rs` and `Cargo.toml` under `dir`, relative to `root`, skipping
/// `target`, hidden directories and the configured ignores.
fn walk(
    root: &Path,
    dir: &Path,
    config: &Config,
    files: &mut Vec<PathBuf>,
    out: &mut Vec<Finding>,
) {
    let relative = |path: &Path| path.strip_prefix(root).unwrap_or(path).to_path_buf();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            out.push(unreadable(relative(dir), &e));
            return;
        }
    };
    for entry in entries {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(e) => {
                out.push(unreadable(relative(dir), &e));
                continue;
            }
        };
        let rel = relative(&path);
        if config.ignore.iter().any(|skip| rel.starts_with(skip)) {
            continue;
        }
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if path.is_dir() {
            if !name.starts_with('.') && name != "target" {
                walk(root, &path, config, files, out);
            }
        } else if name == "Cargo.toml" || path.extension().is_some_and(|ext| ext == "rs") {
            files.push(rel);
        }
    }
}

fn unreadable(path: PathBuf, e: &std::io::Error) -> Finding {
    Finding {
        path,
        line: 0,
        rule: Rule::Unreadable,
        what: e.to_string(),
    }
}

/// A finding in `src` at `line`.
fn finding(src: &Source, line: usize, rule: Rule, what: impl Into<String>) -> Finding {
    Finding {
        path: src.path.clone(),
        line,
        rule,
        what: what.into(),
    }
}
