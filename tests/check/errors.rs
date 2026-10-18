//! The Check section of `errors.md`.

use crate::findings;

#[test]
fn a_crate_listing_a_banned_error_crate_fails() {
    assert_eq!(
        findings("banned-crate"),
        [
            "Cargo.toml:7: banned-crate: lists anyhow",
            "crates/raw/Cargo.toml:7: banned-crate: lists thiserror",
            "crates/raw/Cargo.toml:9: banned-crate: lists eyre",
            "crates/raw/Cargo.toml:13: banned-crate: lists color-eyre",
        ],
        "every dependency table, by key, header or package rename"
    );
}

#[test]
fn a_boxed_dyn_error_fails_with_its_line() {
    assert_eq!(
        findings("boxed-error"),
        [
            "src/lib.rs:2: boxed-error: fn bad() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }",
            "src/lib.rs:3: boxed-error: fn worse() -> Result<(), Box< dyn Error + Send >> { Ok(()) }",
        ],
        "a borrowed dyn Error, a comment and a string are not hits"
    );
}

#[test]
fn a_variant_with_a_source_displays_it() {
    assert_eq!(
        findings("variant-display"),
        [
            "src/error.rs:13: variant-display: Decode has a source but its display lacks {source}",
            "src/error.rs:19: variant-display: Unpack has a source but no display string",
        ],
        "a display without {{source}}, and no display at all"
    );
}

#[test]
fn a_variant_with_a_source_records_its_location() {
    assert_eq!(
        findings("variant-location"),
        [
            "src/error.rs:13: variant-location: Decode has a source but no #[snafu(implicit)] location: Loc",
            "src/error.rs:15: variant-location: Unpack has a source but no #[snafu(implicit)] location: Loc",
            "src/error.rs:20: variant-location: Process has a source but no #[snafu(implicit)] location: Loc",
        ],
        "no location, a location that is not implicit, and snafu's own Location"
    );
}

#[test]
fn the_untyped_snafu_forms_fail() {
    assert_eq!(
        findings("untyped-snafu"),
        [
            "src/lib.rs:1: untyped-snafu: Whatever",
            "src/lib.rs:1: untyped-snafu: snafu::Report",
            "src/lib.rs:6: untyped-snafu: context(false)",
            "src/lib.rs:15: untyped-snafu: Whatever",
            "src/lib.rs:16: untyped-snafu: whatever!",
            "src/lib.rs:19: untyped-snafu: snafu::Report",
            "src/lib.rs:20: untyped-snafu: snafu::Report",
        ],
        "context(false), Whatever, whatever! and Report, imported or by path"
    );
}

#[test]
fn source_is_walked_only_in_error_rs_and_tests() {
    assert_eq!(
        findings("source-call"),
        ["src/lib.rs:4: source-call: .source() outside error.rs and tests"],
        "error.rs, a #[cfg(test)] module and tests/ are exempt"
    );
}
