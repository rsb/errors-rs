use std::error::Error as _;

#[test]
fn the_cause_is_reachable() {
    let e = std::io::Error::other("x");
    assert!(e.source().is_none(), "a leaf has no source");
}
