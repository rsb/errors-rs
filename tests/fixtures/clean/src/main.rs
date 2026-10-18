fn main() {
    rsb_errors::panic::install();
    println!("main may print");
    eprintln!("and report");
}
