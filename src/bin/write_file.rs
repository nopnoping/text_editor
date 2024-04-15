use std::fs::File;
use std::io::Write;

fn main() {
    let mut file = File::create("test").unwrap();
    file.write_all(b"\twda\twda").unwrap()
}