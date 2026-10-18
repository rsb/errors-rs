//! A clean crate: every exemption and no hit. A doc example is masked:
//!
//! ```
//! panic!();
//! whatever!("in a doc comment");
//! ```

pub mod boundary;
pub mod dispatch;
pub mod error;
pub mod ffi;

pub use error::Error;

/// A string is masked too.
pub const NOTE: &str = "Box<dyn Error>, println!(), catch_unwind( and .source() in a string";
pub const RAW: &str = r#"panic!() "quoted" whatever! expect("")"#;
pub const QUOTE: char = '"';
pub const ESCAPED: char = '\'';

/* A block comment with panic!() and /* a nested one with error!() */ too. */
pub fn step() -> Result<(), Error> {
    let empty: Vec<u8> = Vec::new();
    assert!(empty.is_empty(), "a fresh vector is empty");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;

    #[test]
    fn exempt() {
        println!("tests may print");
        let _ = crate::step().err().and_then(|e| e.source().map(|s| s.to_string()));
        std::panic::catch_unwind(|| panic!()).unwrap_err();
    }
}
