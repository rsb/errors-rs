# Errors

This crate follows `../../rules/standards/errors.md` and is the home of its
runtime types: `Loc`, `show_locations` and `locations_shown`.

**Check.** `cargo test --all-features --test error_line` holds the golden
lines for both display modes. `cargo clippy` enforces the must-use lints.
The repository-wide checks the standard names live in `src/check.rs` behind
the `check` feature, with one planted-hit test per rule, and are run by each
consumer's `cargo xtask errors`, never by this crate on itself.
