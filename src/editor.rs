use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufRead, BufReader, Stdout, stdout, Write};
use termion::raw::{IntoRawMode, RawTerminal};
use crate::config::EditorCfg;
use crate::key::Keys;

const VERSION: &str = "0.0.1";
const TABLE_STOP: u8 = 8;

macro_rules! ctrl_key {
    ($k:expr) => {($k as u8) & 0x1f};
}

pub struct Editor<'a> {
    stdout: RawTerminal<Stdout>,
    cx: u32,
    cy: u32,
    row_off: u32,
    col_off: u32,
    rows_num: u32,
    row: Vec<Vec<u8>>,
    render: Vec<Vec<u8>>,
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
            row_off: 0,
            col_off: 0,
            rows_num: 0,
            row: Vec::new(),
            render: Vec::new(),
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
    // process key
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

    // move cursor to right location
    fn move_cursor(&mut self, key: Keys) {
        match key {
            // up move
            Keys::ARROW_UP => {
                self.cy = self.cy.saturating_sub(1);
            }
            // down move
            Keys::ARROW_DOWN => {
                self.cy = min(self.cy.wrapping_add(1), max(self.rows_num, self.cfg.screen_row));
            }
            // left move
            Keys::ARROW_LEFT => {
                if self.cx > 0 {
                    self.cx -= 1;
                } else if self.cy > 0 { // move to right at the end of a line
                    self.cy -= 1;
                    if self.cy < self.rows_num {
                        self.cx = self.render[self.cy as usize].len() as u32;
                    }
                }
            }
            Keys::ARROW_RIGHT => {
                if self.cy < self.rows_num {
                    if self.cx < self.render[self.cy as usize].len() as u32 {
                        self.cx = self.cx.wrapping_add(1)
                    } else { // move to left at the next of a line
                        self.cx = 0;
                        self.cy += 1;
                    }
                }
            }
            _ => {}
        }

        // check x
        if self.cy < self.rows_num {
            self.cx = min(self.cx, (self.render[self.cy as usize].len()) as u32)
        } else {
            self.cx = 0;
        }
    }

    fn refresh_screen(&mut self) {
        self.scroll();
        self.stdout.write_all(b"\x1b[?25l").unwrap();
        self.stdout.write_all(b"\x1b[H").unwrap();
        self.draw_rows();
        self.stdout.write_all(
            format!("\x1b[{};{}H", self.cy - self.row_off + 1, self.cx - self.col_off + 1).as_bytes()
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

        if self.cx < self.col_off {
            self.col_off = self.cx;
        }

        if self.cx >= self.col_off + self.cfg.screen_col {
            self.col_off = self.cx - self.cfg.screen_col + 1;
        }
    }

    fn draw_rows(&mut self) {
        for r in 0..self.cfg.screen_row {
            let file_row = r + self.row_off;
            // draw file content
            if file_row < self.rows_num {
                let row = &self.render[file_row as usize];
                if self.col_off < row.len() as u32 {
                    let row = &row[
                        self.col_off as usize
                            ..min(row.len(), (self.col_off + self.cfg.screen_col) as usize)
                        ];
                    self.stdout.write(row).unwrap();
                }
            } else if self.rows_num == 0 && r == self.cfg.screen_row / 3 { // draw hello
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
            } else { // draw empty line
                self.stdout.write_all(b"~").unwrap();
            }

            // clean
            self.stdout.write_all(b"\x1b[K").unwrap();
            // to next line
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
                self.render.push(Vec::new());

                // store raw content
                self.row[self.rows_num as usize].write(line.as_bytes()).unwrap();

                // store render content
                let mut render_vec = Vec::new();
                for c in line.as_bytes() {
                    if *c == '\t' as u8 {
                        render_vec.push(' ' as u8);
                        while render_vec.len() % TABLE_STOP != 0 {
                            render_vec.push(' ' as u8);
                        }
                    } else {
                        render_vec.push(c.clone());
                    }
                }
                self.render[self.rows_num as usize].write(&render_vec).unwrap();

                self.rows_num += 1;
            }
        }
    }
}
