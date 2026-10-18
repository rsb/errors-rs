# rsb-errors

The runtime pieces of the rsb error and panic standards, shared by every rsb
Rust crate. The standards are `rules/standards/errors.md` and `panics.md`
(repository `rsb/rules`); this crate holds what they need at run time and,
behind the `check` feature, the checks their Check sections name.

- `Loc`: the implicit location snafu stores on every variant. Its `Display`
  is ` (at file:line)` by default and empty after `show_locations(false)`,
  so one display string serves both modes.
- `show_locations`, `locations_shown`: the process-wide switch. Binaries
  wire it to `<APP>_ERROR_LOCATIONS=0`.
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
rsb_errors::show_locations(std::env::var_os("LUMEN_ERROR_LOCATIONS").is_none_or(|v| v != "0"));
// in the dispatcher, around one command:
let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| run(command)));
let outcome = outcome.unwrap_or_else(|_| Err(Error::Internal { what: rsb_errors::panic::take(), .. }));
```

The one place the error is logged is the boundary that produced `outcome`.
See `tests/error_line.rs` for the exact lines a user sees in each mode.

## Why locations are on by default

An error line is read by the owner at a terminal or by an agent with one
line of context. Each layer of the line names the operation that failed,
and the location after it names the `.context` call that wrapped the
layer below:

```
decode /photos/DSCF0001.RAF: open (at crates/libraw/src/adapter.rs:118): cannot open file (at crates/ffi/src/processor.rs:87): No such file or directory (os error 2)
```

Without the locations the same line still reads left to right, but the
reader has to find the code from the operation names, and a name like
`open` is only as precise as the variant it came from. The first version
of this crate printed locations only after `show_locations(true)`, wired
to a `--debug` flag, so the first sight of an error was the short line and
the location cost a second run. The owner decided on 2026-10-18 that the
reader wants the file and line the first time: the default line is the
one with locations, and the cost is a line about twice as long.

The switch stays, inverted, for the places that want a stable string: a
command reply that goes into an agent record, a golden file, a test
fixture. There a line number that shifts with every edit is noise, so the
binary calls `show_locations(false)` when `<APP>_ERROR_LOCATIONS=0`.
`tests/error_line.rs` follows the same split: it asserts one `(at ` per
layer by default, then turns locations off and pins the exact line, so no
test hard-codes a line number.

## Lints apps copy

The `[lints]` tables in `Cargo.toml` and the `clippy.toml` are the ones
`panics.md` asks of every rsb crate. Copy them whole; do not pick.

## Status

0.1.1, not on crates.io (`publish = false`). Consumers take it as a git
dependency pinned by tag:

```toml
rsb-errors = { git = "https://github.com/rsb/errors-rs", tag = "v0.1.1" }
```

`v0.1.0` printed locations only after `show_locations(true)`; `v0.1.1`
prints them by default. No consumer pins `v0.1.0`.

Related repositories: `../rules` holds the standards, `../lumen`, `../libraw`
and `../agent-driver` are the consumers, each with a migration ticket.
