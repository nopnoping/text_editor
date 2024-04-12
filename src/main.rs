use std::io;
use std::io::prelude::*;
use std::io::{Read, stdout, Write};
use termion::raw::IntoRawMode;

fn main() {
    let stdin = io::stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut stdin = stdin.lock();

    stdout.flush().unwrap();

    let mut c = [0;1];
    loop {
        io::
        stdin.read(&mut c).unwrap();
        if c[0] == 'q' as u8 {
            break;
        }
        if c[0].is_ascii_control() {
            print!("{}\r\n", c[0])
        } else {
            print!("{} ({})\r\n", c[0], c[0] as char)
        }
    }

}
