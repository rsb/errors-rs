//! The Check section of `rules/standards/panics.md` for source files: a
//! panic with no message, a `catch_unwind` outside the dispatcher, and an
//! `extern "C"` callback with no guard. The release profile is in
//! `manifest.rs`.

use super::source::{Source, compact, from, matching, slice};
use super::{Config, Finding, Rule, finding};

pub fn check(src: &Source, config: &Config, out: &mut Vec<Finding>) {
    bare_panics(src, out);
    let callbacks = extern_c(src, out);
    stray_catches(src, config, &callbacks, out);
}

/// `panic!()`, `unreachable!()` and `expect("")` outside tests. The
/// masking keeps a string's quotes and replaces its content, so only an
/// empty message still reads `expect("")`.
fn bare_panics(src: &Source, out: &mut Vec<Finding>) {
    for line in src.lines() {
        if src.is_test_at(line.offset) {
            continue;
        }
        let squeezed = compact(line.masked);
        for needle in ["panic!()", "unreachable!()", "expect(\"\")"] {
            if squeezed.contains(needle) {
                out.push(finding(
                    src,
                    line.number,
                    Rule::BarePanic,
                    format!("{needle} with no message"),
                ));
            }
        }
    }
}

/// Every `extern "C" fn` with a body: a finding when its first statement
/// does not call `catch_unwind`, and the body's range either way so the
/// `catch_unwind` inside it is allowed.
fn extern_c(src: &Source, out: &mut Vec<Finding>) -> Vec<(usize, usize)> {
    let masked = src.masked.as_str();
    let mut bodies = Vec::new();
    for (at, needle) in masked.match_indices("extern \"") {
        let quote = at + needle.len() - 1;
        let Some(close) = from(masked, quote + 1).find('"').map(|i| quote + 1 + i) else {
            continue;
        };
        let abi = slice(&src.text, quote + 1, close);
        if abi != "C" && abi != "C-unwind" {
            continue;
        }
        let rest = from(masked, close + 1);
        let signature = rest.trim_start();
        if !signature.starts_with("fn ") {
            continue;
        }
        let name = signature
            .trim_start_matches("fn ")
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()
            .unwrap_or_default();
        let Some(body_open) = signature
            .find(['{', ';'])
            .map(|i| close + 1 + (rest.len() - signature.len()) + i)
        else {
            continue;
        };
        if masked.as_bytes().get(body_open) != Some(&b'{') {
            continue;
        }
        let Some(body_close) = matching(masked, body_open) else {
            continue;
        };
        let first_statement = slice(masked, body_open + 1, body_close)
            .split([';', '{'])
            .next()
            .unwrap_or_default();
        if !first_statement.contains("catch_unwind(") {
            out.push(finding(
                src,
                src.line_of(at),
                Rule::ExternC,
                format!("extern \"C\" fn {name} does not begin with catch_unwind"),
            ));
        }
        bodies.push((body_open, body_close));
    }
    bodies
}

/// A `catch_unwind` call outside the dispatcher file, the callbacks and
/// tests.
fn stray_catches(
    src: &Source,
    config: &Config,
    callbacks: &[(usize, usize)],
    out: &mut Vec<Finding>,
) {
    if src.path == config.dispatcher {
        return;
    }
    for at in src.tokens("catch_unwind(") {
        if src.is_test_at(at)
            || callbacks
                .iter()
                .any(|&(open, close)| at > open && at < close)
        {
            continue;
        }
        out.push(finding(
            src,
            src.line_of(at),
            Rule::CatchUnwind,
            format!(
                "catch_unwind outside the dispatcher {}",
                config.dispatcher.display()
            ),
        ));
    }
}
