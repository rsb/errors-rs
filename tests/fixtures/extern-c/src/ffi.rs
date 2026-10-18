use std::panic::catch_unwind;

pub extern "C" fn good(data: *mut u8) -> i32 {
    match catch_unwind(|| step(data)) {
        Ok(code) => code,
        Err(_) => 1,
    }
}

pub unsafe extern "C" fn bad(data: *mut u8) -> i32 {
    let code = step(data);
    code
}

unsafe extern "C" {
    fn libraw_open(path: *const u8) -> i32;
}

fn step(_data: *mut u8) -> i32 {
    0
}
