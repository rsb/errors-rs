# Errors

This crate follows `../../rules/standards/errors.md` and is the home of its
runtime types: `Loc`, `show_locations` and `locations_shown`. `Loc` prints
` (at file:line)` by default; `show_locations(false)` is the off switch.
The reason is in `README.md`.

**Check.** `cargo test --all-features --test error_line` holds the golden
lines for both display modes. `cargo clippy` enforces the must-use lints.
The repository-wide checks the standard names live in `src/check/` behind
the `check` feature as `check::run(root: &Path, config: &Config) ->
Vec<Finding>`, with one planted-hit fixture per rule under `tests/fixtures/`
and one clean fixture. Each consumer's `cargo xtask errors` calls it;
`tests/check` also runs it on this repository, ignoring the fixtures.
