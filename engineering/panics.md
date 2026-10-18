# Panics

This crate follows `../../rules/standards/panics.md` and is the home of its
runtime module `panic` (`install`, `take`, `Report`).

**Check.** The `[lints]` tables in `Cargo.toml` and `clippy.toml` are the
standard's tables verbatim; `cargo clippy -- -D warnings` enforces them.
`cargo test --all-features --test error_line` proves that an installed hook
records a panic's message and location and that `take` clears it.
