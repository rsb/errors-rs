//! The Check section of `rules/standards/logging.md`: printing or the
//! `log` crate outside `main`, a level above `info` outside the boundary,
//! and `#[instrument(err)]`.

use super::source::{Source, compact, is_ident, matching, slice, tokens};
use super::{Config, Finding, Rule, finding};

pub fn check(src: &Source, config: &Config, out: &mut Vec<Finding>) {
    if !src.is_main() {
        prints(src, out);
    }
    if src.path != config.boundary && !src.path.starts_with(&config.boundary) {
        levels(src, config, out);
    }
    instrument_err(src, out);
}

/// `println!`, `eprintln!` and the `log` crate outside tests.
fn prints(src: &Source, out: &mut Vec<Finding>) {
    for line in src.lines() {
        if src.is_test_at(line.offset) {
            continue;
        }
        for needle in ["println!", "eprintln!"] {
            if tokens(line.masked, needle).is_empty() {
                continue;
            }
            out.push(finding(
                src,
                line.number,
                Rule::Print,
                format!("{needle} outside main and tests"),
            ));
        }
        if uses_log_crate(line.masked) {
            out.push(finding(
                src,
                line.number,
                Rule::Print,
                "log crate outside main and tests",
            ));
        }
    }
}

/// `use log`, `extern crate log` or a `log::` path that is not part of a
/// longer path such as `tracing_log::` or `crate::log::`.
fn uses_log_crate(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("use log;")
        || trimmed.starts_with("use log::")
        || trimmed.starts_with("extern crate log")
    {
        return true;
    }
    let bytes = line.as_bytes();
    tokens(line, "log::")
        .iter()
        .any(|&at| at < 2 || (bytes[at - 1] != b':' && !is_ident(bytes[at - 1])))
}

/// `warn!` and `error!` anywhere in a file outside the boundary.
fn levels(src: &Source, config: &Config, out: &mut Vec<Finding>) {
    for line in src.lines() {
        for needle in ["warn!", "error!"] {
            if tokens(line.masked, needle).is_empty() {
                continue;
            }
            out.push(finding(
                src,
                line.number,
                Rule::LogLevel,
                format!(
                    "{needle} outside the boundary {}",
                    config.boundary.display()
                ),
            ));
        }
    }
}

/// `#[instrument(err)]` and `#[tracing::instrument(..., err(...), ...)]`,
/// across lines.
fn instrument_err(src: &Source, out: &mut Vec<Finding>) {
    let masked = src.masked.as_str();
    let bytes = masked.as_bytes();
    for at in tokens(masked, "instrument(") {
        if at == 0 || !matches!(bytes[at - 1], b'[' | b':') {
            continue;
        }
        let open = at + "instrument".len();
        let Some(close) = matching(masked, open) else {
            continue;
        };
        let args = compact(slice(masked, open + 1, close));
        let has_err = args
            .split(',')
            .any(|arg| arg == "err" || arg.starts_with("err("));
        if has_err {
            out.push(finding(
                src,
                src.line_of(at),
                Rule::InstrumentErr,
                "#[instrument(err)]",
            ));
        }
    }
}
