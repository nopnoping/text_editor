use std::io;
use std::io::prelude::*;
use std::io::{Read, stdout, Write};
use termion::raw::IntoRawMode;

macro_rules! ctrl_key {
    ($k:expr) => {($k as u8) & 0x1f};
}


fn editor_read_key() -> u8 {
    let mut c = [0; 1];
    io::stdin().lock().read(&mut c).unwrap();
    return c[0];
}

fn editor_process_key_press() -> bool {
    let key = editor_read_key();
    match key {
        k if k == ctrl_key!('q') => false,
        _ => true
    }
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    stdout.flush().unwrap();

    while editor_process_key_press() {}
}
