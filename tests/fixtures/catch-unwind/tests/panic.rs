#[test]
fn a_panic_is_caught() {
    let caught = std::panic::catch_unwind(|| panic!("boom"));
    assert!(caught.is_err(), "the closure panics");
}
