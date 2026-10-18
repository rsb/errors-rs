use std::panic::catch_unwind;

pub extern "C" fn progress(_data: *mut u8) -> i32 {
    let outcome = catch_unwind(|| 0);
    outcome.unwrap_or(1)
}
