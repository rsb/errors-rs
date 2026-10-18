# rsb-errors

The runtime pieces of the rsb error and panic standards, shared by every rsb
Rust crate. The standards are `rules/standards/errors.md` and `panics.md`
(repository `rsb/rules`); this crate holds what they need at run time and,
behind the `check` feature, the checks their Check sections name.

- `Loc`: the implicit location snafu stores on every variant. Its `Display`
  is empty by default and ` (at file:line)` once `show_locations(true)` has
  been called, so one display string serves both modes.
- `show_locations`, `locations_shown`: the process-wide toggle. Binaries
  wire it to `--debug` and `<APP>_ERROR_LOCATIONS`.
- `panic::install`, `panic::take`: a panic hook that records message, file,
  line and column of the last panic, so the dispatcher's `catch_unwind` can
  turn it into an `Internal` error with the location.
- `check::run(root: &Path, config: &Config) -> Vec<Finding>` (feature
  `check`): the checks the three standards name, over every `.rs` and
  `Cargo.toml` under `root`. `Config` names the dispatcher file, the
  boundary module and paths to ignore; each `Finding` displays as one line
  `path:line: rule: what`. Each repository's `cargo xtask errors` calls it
  and exits 1 on any finding.

## What a binary writes

Four lines, and nothing else from this crate:

```rust
rsb_errors::panic::install();
rsb_errors::show_locations(args.debug || std::env::var_os("LUMEN_ERROR_LOCATIONS").is_some());
// in the dispatcher, around one command:
let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| run(command)));
let outcome = outcome.unwrap_or_else(|_| Err(Error::Internal { what: rsb_errors::panic::take(), .. }));
```

The one place the error is logged is the boundary that produced `outcome`.
See `tests/error_line.rs` for the exact lines a user sees in each mode.

## Lints apps copy

The `[lints]` tables in `Cargo.toml` and the `clippy.toml` are the ones
`panics.md` asks of every rsb crate. Copy them whole; do not pick.

## Status

0.1.0, not on crates.io (`publish = false`). Consumers take it as a git
dependency pinned by tag:

```toml
rsb-errors = { git = "https://github.com/rsb/errors-rs", tag = "v0.1.0" }
```

Related repositories: `../rules` holds the standards, `../lumen`, `../libraw`
and `../agent-driver` are the consumers, each with a migration ticket.
