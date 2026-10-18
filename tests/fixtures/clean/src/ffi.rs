use std::panic::catch_unwind;

pub extern "C" fn progress(_data: *mut u8, _stage: i32) -> i32 {
    match catch_unwind(|| 0) {
        Ok(code) => code,
        Err(_) => 1,
    }
}

unsafe extern "C" {
    pub fn libraw_init(flags: u32) -> *mut u8;
}
