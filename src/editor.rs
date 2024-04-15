use std::io;
use std::io::{Read, Stdout, stdout, Write};
use termion::raw::{IntoRawMode, RawTerminal};
use crate::config::EditorCfg;

macro_rules! ctrl_key {
    ($k:expr) => {($k as u8) & 0x1f};
}

pub struct Editor {
    stdout: RawTerminal<Stdout>,
    cfg: EditorCfg,
}

impl Editor {
    pub fn new(cfg: EditorCfg) -> Self {
        let mut stdout = stdout().into_raw_mode().unwrap();
        stdout.flush().unwrap();
        Editor {
            stdout,
            cfg,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if !self.process_key_press() {
                break;
            }
        }
    }
}

impl Editor {
    fn read_key(&self) -> u8 {
        let mut c = [0; 1];
        io::stdin().lock().read(&mut c).unwrap();
        return c[0];
    }

    fn process_key_press(&mut self) -> bool {
        let key = self.read_key();
        match key {
            k if k == ctrl_key!('q') => {
                self.stdout.write(b"\x1b[2J").unwrap();
                self.stdout.write(b"\x1b[H").unwrap();
                false
            }
            _ => true
        }
    }

    fn refresh_screen(&mut self) {
        self.stdout.write_all(b"\x1b[2J").unwrap();
        self.stdout.write_all(b"\x1b[H").unwrap();
        self.draw_rows();
        self.stdout.write_all(b"\x1b[H").unwrap();
        self.stdout.flush().unwrap();
    }

    fn draw_rows(&mut self) {
        for r in 0..self.cfg.screen_size.1 {
            self.stdout.write_all(b"~").unwrap();
            if r < self.cfg.screen_size.1 - 1 {
                self.stdout.write_all(b"\r\n").unwrap();
            }
        }
    }
}
