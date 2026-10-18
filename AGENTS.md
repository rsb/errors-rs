# AGENTS.md

Read `README.md` first. Sessions run from this repository. Standards shared by
the rsb apps live in `../rules`; read its `README.md`. This crate implements
the runtime half of `../rules/standards/errors.md` and `panics.md`, and its
`check` feature implements their Check sections for every consumer.

Commits follow Conventional Commits. Never add `Co-Authored-By` or any other AI
attribution to commits or pull requests. Outside ticket work, do not commit or
push unless asked. Dates and times are America/Los_Angeles.

Checks. Add `--message-format short` to check, test and clippy. The `check`
feature is off by default, so every command carries `--all-features`. CI runs
the last three on every push.

- Typecheck: `cargo check --workspace --all-targets --all-features`
- One test file: `cargo test -p rsb-errors --all-features --test <file>`
- Full suite: `cargo test --workspace --all-features`
- Lint: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Format: `cargo fmt --check`

The rules these enforce are in `engineering/`.
