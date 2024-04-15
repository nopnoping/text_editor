use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufRead, BufReader, Stdout, stdout, Write};
use termion::raw::{IntoRawMode, RawTerminal};
use crate::config::EditorCfg;
use crate::key::Keys;

const VERSION: &str = "0.0.1";

macro_rules! ctrl_key {
    ($k:expr) => {($k as u8) & 0x1f};
}

pub struct Editor<'a> {
    stdout: RawTerminal<Stdout>,
    cx: u32,
    cy: u32,
    rows_num: u32,
    row_off: u32,
    row: Vec<Vec<u8>>,
    cfg: EditorCfg<'a>,
}

/* pub func */
impl<'a> Editor<'a> {
    pub fn new(cfg: EditorCfg<'a>) -> Self {
        let mut stdout = stdout().into_raw_mode().unwrap();
        stdout.flush().unwrap();
        Editor {
            stdout,
            cfg,
            cx: 0,
            cy: 0,
            rows_num: 0,
            row_off: 0,
            row: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        self.edit_or_open();
        loop {
            self.refresh_screen();
            if !self.process_key_press() {
                break;
            }
        }
    }
}

/* private func */
impl Editor<'_> {
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
                    self.move_cursor(Keys::ARROW_UP);
                    times -= 1;
                }
            }
            Keys::PAGE_DOWN => {
                let mut times = self.cfg.screen_row;
                while times > 0 {
                    self.move_cursor(Keys::ARROW_DOWN);
                    times -= 1;
                }
            }
            Keys::HOME_KEY => self.cx = 0,
            Keys::END_KEY => self.cx = self.cfg.screen_col - 1,
            Keys::ARROW_UP | Keys::ARROW_DOWN | Keys::ARROW_LEFT | Keys::ARROW_RIGHT => self.move_cursor(key),
            _ => {}
        };
        return true;
    }

    fn move_cursor(&mut self, key: Keys) {
        match key {
            Keys::ARROW_UP => {
                self.cy = self.cy.saturating_sub(1);
            }
            Keys::ARROW_DOWN => {
                self.cy = min(self.cy.wrapping_add(1), max(self.rows_num, self.cfg.screen_row));
            }
            Keys::ARROW_LEFT => {
                self.cx = self.cx.saturating_sub(1);
            }
            Keys::ARROW_RIGHT => {
                self.cx = min(self.cx.wrapping_add(1), self.cfg.screen_col);
            }
            _ => {}
        }
    }

    fn refresh_screen(&mut self) {
        self.scroll();
        self.stdout.write_all(b"\x1b[?25l").unwrap();
        self.stdout.write_all(b"\x1b[H").unwrap();
        self.draw_rows();
        self.stdout.write_all(
            format!("\x1b[{};{}H", self.cy - self.row_off + 1, self.cx + 1).as_bytes()
        ).unwrap();
        self.stdout.write_all(b"\x1b[?25h").unwrap();
        self.stdout.flush().unwrap();
    }

    fn scroll(&mut self) {
        if self.cy < self.row_off {
            self.row_off = self.cy;
        }

        if self.cy >= self.row_off + self.cfg.screen_row {
            self.row_off = self.cy - self.cfg.screen_row + 1;
        }
    }

    fn draw_rows(&mut self) {
        for r in 0..self.cfg.screen_row {
            let file_row = r as u32 + self.row_off;
            if file_row < self.rows_num {
                let row = &self.row[file_row as usize];
                let row = &row[..min(row.len(), self.cfg.screen_col as usize)];
                self.stdout.write(row).unwrap();
            } else if self.rows_num == 0 && r == self.cfg.screen_row / 3 {
                let welcome = format!("My editor -- version:{}", VERSION);
                let welcome = &welcome[..min(welcome.len(), self.cfg.screen_col as usize)];
                let mut padding = (self.cfg.screen_col.wrapping_sub(welcome.len() as u32)) / 2;
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

    fn edit_or_open(&mut self) {
        if self.cfg.file_name != "" {
            let file = File::open(&self.cfg.file_name).unwrap();
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line.unwrap().replace("\r\n", "");
                self.row.push(Vec::new());
                self.row[self.rows_num as usize].write(line.as_bytes()).unwrap();
                self.row[self.rows_num as usize].write(b"\0").unwrap();
                self.rows_num += 1;
            }
        }
    }
}
