use std::error::Error as _;

pub fn root(e: &dyn std::error::Error) -> String {
    e.source().map_or_else(|| e.to_string(), |s| s.to_string())
}
