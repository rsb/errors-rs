use std::panic::{AssertUnwindSafe, catch_unwind};

pub fn guarded(f: impl FnOnce()) {
    let _ = catch_unwind(AssertUnwindSafe(f));
}
