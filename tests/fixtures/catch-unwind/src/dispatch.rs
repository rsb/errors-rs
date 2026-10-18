use std::panic::{AssertUnwindSafe, catch_unwind};

pub fn execute(command: &str) -> Result<(), String> {
    catch_unwind(AssertUnwindSafe(|| run(command))).unwrap_or_else(|_| Err("internal".into()))
}
