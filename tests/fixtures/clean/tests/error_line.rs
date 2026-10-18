#[test]
fn a_panic_is_reported() {
    rsb_errors::panic::install();
    let caught = std::panic::catch_unwind(|| panic!());
    assert!(caught.is_err(), "the closure panics");
    let report = rsb_errors::panic::take().expect("");
    println!("{report}");
}
