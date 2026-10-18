pub fn a() {
    panic!()
}

pub fn b() {
    unreachable!()
}

pub fn c(v: Option<u8>) -> u8 {
    v.expect("")
}

pub fn d(v: Option<u8>) -> u8 {
    v.expect("decoder set width before unpack")
}

pub fn e() {
    panic!("RGBA rows are whole pixels")
}

#[cfg(test)]
mod tests {
    #[test]
    fn allowed_here() {
        panic!()
    }
}
