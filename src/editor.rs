use std::cmp::min;
use std::io::{Stdout, stdout, Write};
use termion::raw::{IntoRawMode, RawTerminal};
use crate::config::EditorCfg;
use crate::key::Keys;

const VERSION: &str = "0.0.1";

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
    fn process_key_press(&mut self) -> bool {
        let key = Keys::read_key();
        match key {
            Keys::NORMAL(k) if k == ctrl_key!('q') => {
                print!("\x1b[2J\x1b[H");
                return false;
            }
            Keys::PAGE_UP => {
                let mut times = self.cfg.screen_row;
                while times > 0 {
                    self.cfg.move_cursor(Keys::ARROW_UP);
                    times -= 1;
                }
            }
            Keys::PAGE_DOWN => {
                let mut times = self.cfg.screen_row;
                while times > 0 {
                    self.cfg.move_cursor(Keys::ARROW_DOWN);
                    times -= 1;
                }
            }
            Keys::HOME_KEY => self.cfg.cx = 0,
            Keys::END_KEY => self.cfg.cx = self.cfg.screen_col - 1,
            Keys::ARROW_UP | Keys::ARROW_DOWN | Keys::ARROW_LEFT | Keys::ARROW_RIGHT => self.cfg.move_cursor(key),
            _ => {}
        };
        return true;
    }

    fn refresh_screen(&mut self) {
        self.stdout.write_all(b"\x1b[?25l").unwrap();
        self.stdout.write_all(b"\x1b[H").unwrap();
        self.draw_rows();
        self.stdout.write_all(
            format!("\x1b[{};{}H", self.cfg.cy + 1, self.cfg.cx + 1).as_bytes()
        ).unwrap();
        self.stdout.write_all(b"\x1b[?25h").unwrap();
        self.stdout.flush().unwrap();
    }

    fn draw_rows(&mut self) {
        for r in 0..self.cfg.screen_row {
            if r == self.cfg.screen_row / 3 {
                let welcome = format!("My editor -- version:{}", VERSION);
                let welcome = &welcome[..min(welcome.len(), self.cfg.screen_col as usize)];
                let mut padding = (self.cfg.screen_col.wrapping_sub(welcome.len() as u16)) / 2;
                if padding > 0 {
                    self.stdout.write_all(b"~").unwrap();
                    padding -= 1;
                }
                while padding > 0 {
                    self.stdout.write_all(b" ").unwrap();
                    padding -= 1;
                }
                self.stdout.write_all(welcome.as_bytes()).unwrap();
            } else {
                self.stdout.write_all(b"~").unwrap();
            }
            self.stdout.write_all(b"\x1b[K").unwrap();
            if r < self.cfg.screen_row - 1 {
                self.stdout.write_all(b"\r\n").unwrap();
            }
        }
    }
}
