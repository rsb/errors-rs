//! The Check section of `panics.md`.

use crate::findings;

#[test]
fn a_release_profile_states_unwind_and_overflow_checks() {
    assert_eq!(
        findings("release-profile"),
        ["Cargo.toml:9: release-profile: [profile.release] lacks overflow-checks = true"],
        "the key it lacks, at the table header"
    );
    assert_eq!(
        findings("no-release-profile"),
        ["Cargo.toml:1: release-profile: no [profile.release]"],
        "a root manifest with no release profile"
    );
}

#[test]
fn a_panic_with_no_message_fails_outside_tests() {
    assert_eq!(
        findings("bare-panic"),
        [
            "src/lib.rs:2: bare-panic: panic!() with no message",
            "src/lib.rs:6: bare-panic: unreachable!() with no message",
            "src/lib.rs:10: bare-panic: expect(\"\") with no message",
        ],
        "a message or a #[cfg(test)] module makes it a non-hit"
    );
}

#[test]
fn catch_unwind_lives_in_the_dispatcher_and_callbacks() {
    assert_eq!(
        findings("catch-unwind"),
        ["src/lib.rs:4: catch-unwind: catch_unwind outside the dispatcher src/dispatch.rs"],
        "the dispatcher, an extern \"C\" body and tests are exempt"
    );
}

#[test]
fn an_extern_c_fn_begins_with_catch_unwind() {
    assert_eq!(
        findings("extern-c"),
        ["src/ffi.rs:10: extern-c: extern \"C\" fn bad does not begin with catch_unwind"],
        "a guarded callback and an extern block of declarations are not hits"
    );
}
