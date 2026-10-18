//! The `Cargo.toml` checks: a banned error crate in any dependency table,
//! and the release profile of the root manifest.

use std::path::Path;

use super::{Finding, Rule};

/// Error crates that replace a typed error enum, or cannot record a
/// location.
pub const BANNED: &[&str] = &[
    "anyhow",
    "eyre",
    "color-eyre",
    "failure",
    "error-chain",
    "thiserror",
];

pub fn check(path: &Path, text: &str, out: &mut Vec<Finding>) {
    banned_crates(path, text, out);
    if path == Path::new("Cargo.toml") {
        release_profile(path, text, out);
    }
}

/// The name of the table a header line opens, or `None` for other lines.
fn header(line: &str) -> Option<&str> {
    let inner = line.strip_prefix('[')?.strip_suffix(']')?;
    Some(
        inner
            .strip_prefix('[')
            .and_then(|i| i.strip_suffix(']'))
            .unwrap_or(inner)
            .trim(),
    )
}

/// A line without its comment, trimmed.
fn code(line: &str) -> &str {
    line.split('#').next().unwrap_or_default().trim()
}

/// The key a line assigns, before any `.`: `anyhow` for
/// `anyhow.workspace = true`.
fn key(line: &str) -> Option<&str> {
    let (key, _) = line.split_once('=')?;
    Some(
        key.trim()
            .trim_matches('"')
            .split('.')
            .next()
            .unwrap_or_default(),
    )
}

/// A banned crate under any `dependencies` table, by key, by
/// `[dependencies.<name>]` header or by `package = "<name>"`.
fn banned_crates(path: &Path, text: &str, out: &mut Vec<Finding>) {
    let mut in_dependencies = false;
    for (i, raw) in text.lines().enumerate() {
        let line = code(raw);
        let hit = |name: &str| {
            BANNED.contains(&name).then(|| Finding {
                path: path.to_path_buf(),
                line: i + 1,
                rule: Rule::BannedCrate,
                what: format!("lists {name}"),
            })
        };
        if let Some(table) = header(line) {
            in_dependencies = table.contains("dependencies");
            if let Some((_, name)) = table.rsplit_once("dependencies.") {
                out.extend(hit(name.trim_matches('"')));
            }
            continue;
        }
        if !in_dependencies {
            continue;
        }
        if let Some(found) = key(line).and_then(hit) {
            out.push(found);
        } else if let Some(found) = renamed_package(line).and_then(hit) {
            out.push(found);
        }
    }
}

/// `<name>` from `package = "<name>"` on the line.
fn renamed_package(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once("package")?;
    let rest = rest.trim_start().strip_prefix('=')?.trim_start();
    rest.strip_prefix('"')?.split('"').next()
}

/// The root manifest states `panic = "unwind"` and
/// `overflow-checks = true` under `[profile.release]`.
fn release_profile(path: &Path, text: &str, out: &mut Vec<Finding>) {
    let mut header_line = None;
    let mut in_profile = false;
    let mut panic_unwinds = false;
    let mut overflow_checks = false;
    for (i, raw) in text.lines().enumerate() {
        let line = code(raw);
        if let Some(table) = header(line) {
            in_profile = table == "profile.release";
            if in_profile {
                header_line = Some(i + 1);
            }
            continue;
        }
        if !in_profile {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            match (key.trim(), value.trim()) {
                ("panic", "\"unwind\"") => panic_unwinds = true,
                ("overflow-checks", "true") => overflow_checks = true,
                _ => {}
            }
        }
    }
    let mut lacking = Vec::new();
    if !panic_unwinds {
        lacking.push("panic = \"unwind\"");
    }
    if !overflow_checks {
        lacking.push("overflow-checks = true");
    }
    let (line, what) = match header_line {
        None => (1, "no [profile.release]".to_owned()),
        Some(_) if lacking.is_empty() => return,
        Some(line) => (
            line,
            format!("[profile.release] lacks {}", lacking.join(" and ")),
        ),
    };
    out.push(Finding {
        path: path.to_path_buf(),
        line,
        rule: Rule::ReleaseProfile,
        what,
    });
}
