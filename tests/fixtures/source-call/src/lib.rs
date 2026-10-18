use std::error::Error as _;

pub fn cause(e: &crate::Error) -> Option<String> {
    e.source().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn source_is_reachable() {
        let _ = crate::Error::Open.source();
    }
}
