use log::info;

pub fn a() {
    println!("hello");
}

pub fn b() {
    eprintln!("oops");
}

pub fn c() {
    log::debug!("deep");
}

#[cfg(test)]
mod tests {
    #[test]
    fn prints() {
        println!("in a test");
    }
}
