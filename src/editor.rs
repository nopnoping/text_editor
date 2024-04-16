use std::cmp::{min};
use std::fmt::{Arguments, format};
use std::fs::File;
use std::io::{BufRead, BufReader, Stdout, stdout, Write};
use std::iter::repeat;
use termion::raw::{IntoRawMode, RawTerminal};
use crate::config::EditorCfg;
use crate::key::Keys;
use crate::util::get_current_time_secs;

const VERSION: &str = "0.0.1";
const TABLE_STOP: u8 = 8;
const QUIT_TIMES: u8 = 1;

pub struct Editor {
    stdout: RawTerminal<Stdout>,

    cx: u32,
    rx: u32,
    cy: u32,
    row_off: u32,
    col_off: u32,

    rows_num: u32,
    row: Vec<Vec<u8>>,
    render: Vec<Vec<u8>>,

    dirty: bool,
    quit_time: u8,
    status_msg: String,
    status_msg_time: u64,

    cfg: EditorCfg,
}

/* pub func */
impl Editor {
    pub fn new(cfg: EditorCfg) -> Self {
        // raw mode
        let mut stdout = stdout().into_raw_mode().unwrap();
        stdout.flush().unwrap();
        // construct
        Editor {
            stdout,
            cfg,

            rx: 0,
            cx: 0,
            cy: 0,
            row_off: 0,
            col_off: 0,

            rows_num: 0,
            row: Vec::new(),
            render: Vec::new(),

            dirty: false,
            quit_time: QUIT_TIMES,
            status_msg: String::from(""),
            status_msg_time: 0,
        }
    }

    pub fn run(&mut self) {
        // let default_panic = std::panic::take_hook();
        // std::panic::set_hook(Box::new(move |info| {
        //     let mut stdout = stdout().into_raw_mode().unwrap();
        //     stdout.flush().unwrap();
        //     stdout.suspend_raw_mode().unwrap();
        //     default_panic(info);
        // }));

        self.set_status_msg(format_args!("HELP: Ctrl-S = save | Ctrl-Q = quit"));
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
impl Editor {
    /* process key */
    fn process_key_press(&mut self) -> bool {
        let key = Keys::read_key();
        match key {
            Keys::QUIT => {
                if self.dirty && self.quit_time > 0 {
                    let q = self.quit_time;
                    self.set_status_msg(format_args!("WARNING!!! File has unsaved changes. \
                    Press Ctrl-Q {} more times to quit.", q));
                    self.quit_time = self.quit_time.saturating_sub(1);
                    return true;
                }
                print!("\x1b[2J\x1b[H");
                return false;
            }
            Keys::ENTER => self.insert_new_line(),
            Keys::CTL_S => self.save_file(),
            Keys::PAGE_UP => {
                self.cy = self.row_off;
                let mut times = self.cfg.screen_row;
                while times > 0 {
                    self.move_cursor(Keys::ARROW_UP);
                    times -= 1;
                }
            }
            Keys::PAGE_DOWN => {
                self.cy = self.row_off + self.cfg.screen_row - 1;
                if self.cy > self.rows_num {
                    self.cy = self.rows_num;
                }
                let mut times = self.cfg.screen_row;
                while times > 0 {
                    self.move_cursor(Keys::ARROW_DOWN);
                    times -= 1;
                }
            }
            Keys::HOME_KEY => self.cx = 0,
            Keys::END_KEY => {
                if self.cy < self.rows_num {
                    self.cx = self.row[self.cy as usize].len() as u32;
                }
            }
            Keys::BACKSPACE | Keys::CTL_H | Keys::DEL_KEY => self.delete_char(),
            Keys::ARROW_UP | Keys::ARROW_DOWN | Keys::ARROW_LEFT | Keys::ARROW_RIGHT => self.move_cursor(key),
            Keys::CTL_L | Keys::ESC => {}
            Keys::NORMAL(k) => self.insert_char(k),
        };
        self.quit_time = QUIT_TIMES;
        return true;
    }

    fn move_cursor(&mut self, key: Keys) {
        match key {
            // up move
            Keys::ARROW_UP => {
                self.cy = self.cy.saturating_sub(1);
            }
            // down move
            Keys::ARROW_DOWN => {
                self.cy = min(self.cy.wrapping_add(1), self.rows_num);
            }
            // left move
            Keys::ARROW_LEFT => {
                if self.cx > 0 {
                    self.cx -= 1;
                } else if self.cy > 0 { // move to right at the end of a line
                    self.cy -= 1;
                    if self.cy < self.rows_num {
                        self.cx = self.row[self.cy as usize].len() as u32;
                    }
                }
            }
            Keys::ARROW_RIGHT => {
                if self.cy < self.rows_num {
                    if self.cx < self.row[self.cy as usize].len() as u32 {
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
            self.cx = min(self.cx, (self.row[self.cy as usize].len()) as u32)
        } else {
            self.cx = 0;
        }
    }

    fn insert_char(&mut self, c: u8) {
        if self.cy > self.rows_num {
            panic!("err cy");
        }
        if self.cy == self.rows_num {
            self.row.push(Vec::new());
            self.render.push(Vec::new());
            self.rows_num = self.rows_num.wrapping_add(1);
        }

        let row = &mut self.row[self.cy as usize];
        if self.cx <= row.len() as u32 {
            row.insert(self.cx as usize, c);

            let row = &self.row[self.cy as usize];
            self.render[self.cy as usize] = self.get_render_vec(row);

            self.dirty = true;
            self.cx = self.cx.wrapping_add(1);
        }
    }

    fn delete_char(&mut self) {
        if self.cy >= self.rows_num { return; }
        if self.cx == 0 && self.cy == 0 { return; }

        if self.cx > 0 {
            let row = &mut self.row[self.cy as usize];
            if self.cx > row.len() as u32 { panic!("err cx"); }
            row.remove((self.cx - 1) as usize);

            let row = &self.row[self.cy as usize];
            self.render[self.cy as usize] = self.get_render_vec(row);

            self.dirty = true;
            self.cx = self.cx.saturating_sub(1);
        } else { // delete at the beginning of line
            let row2 = self.row[self.cy as usize].to_vec();
            let row1 = &mut self.row[(self.cy - 1) as usize];
            let row1_len = row1.len() as u32;
            row1.write_all(&row2).unwrap();

            self.row.remove(self.cy as usize);
            self.render.remove(self.cy as usize);

            let row1 = &self.row[(self.cy - 1) as usize];
            self.render[(self.cy - 1) as usize] = self.get_render_vec(row1);

            self.dirty = true;
            self.cy = self.cy.saturating_sub(1);
            self.cx = row1_len;
            self.rows_num = self.rows_num.saturating_sub(1);
        }
    }

    fn insert_new_line(&mut self) {
        if self.cx == 0 {
            self.row.insert(self.cy as usize, Vec::new());
            self.render.insert(self.cy as usize, Vec::new());
        } else {
            let row1 = &mut self.row[self.cy as usize];
            let mut row2 = Vec::new();
            while row1.len() > self.cx as usize {
                row2.push(row1.remove(self.cx as usize));
            }

            let row1 = &self.row[self.cy as usize];
            self.render[self.cy as usize] = self.get_render_vec(row1);

            self.row.insert((self.cy + 1) as usize, row2.to_vec());
            self.render.insert((self.cy + 1) as usize, self.get_render_vec(&row2));
        }

        self.dirty = true;
        self.cx = 0;
        self.cy = self.cy.wrapping_add(1);
        self.rows_num = self.rows_num.wrapping_add(1);
    }
    /* screen refresh */
    fn refresh_screen(&mut self) {
        self.scroll();

        self.stdout.write_all(b"\x1b[?25l").unwrap();
        self.stdout.write_all(b"\x1b[H").unwrap();

        self.draw_rows();
        self.draw_status_bar();
        self.draw_status_msg();

        self.stdout.write_all(
            format!("\x1b[{};{}H", self.cy - self.row_off + 1, self.rx - self.col_off + 1).as_bytes()
        ).unwrap();
        self.stdout.write_all(b"\x1b[?25h").unwrap();

        self.stdout.flush().unwrap();
    }

    fn scroll(&mut self) {
        self.row_cx_to_rx();

        if self.cy < self.row_off {
            self.row_off = self.cy;
        }

        if self.cy >= self.row_off + self.cfg.screen_row {
            self.row_off = self.cy - self.cfg.screen_row + 1;
        }

        if self.rx < self.col_off {
            self.col_off = self.rx;
        }

        if self.rx >= self.col_off + self.cfg.screen_col {
            self.col_off = self.rx - self.cfg.screen_col + 1;
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
            self.stdout.write_all(b"\r\n").unwrap();
        }
    }

    fn draw_status_bar(&mut self) {
        self.stdout.write_all(b"\x1b[7m").unwrap();

        let status = format!("{:20} - {} lines {}", self.cfg.get_file_name(), self.rows_num, self.get_dirty_status());
        let line = format!("{}/{}", self.cy + 1, self.rows_num);
        let spaces: String = repeat(' ').take(self.cfg.screen_col as usize - status.len() - line.len()).collect();

        self.stdout.write_all(status.as_bytes()).unwrap();
        self.stdout.write_all(spaces.as_bytes()).unwrap();
        self.stdout.write_all(line.as_bytes()).unwrap();

        self.stdout.write_all(b"\x1b[m\r\n").unwrap();
    }

    fn draw_status_msg(&mut self) {
        self.stdout.write_all(b"\x1b[K").unwrap();
        if self.status_msg.len() > 0 && get_current_time_secs() - self.status_msg_time < 5 {
            self.stdout.write_all(self.status_msg.as_bytes()).unwrap();
        }
    }

    fn set_status_msg(&mut self, args: Arguments<'_>) {
        self.status_msg = format(args);
        self.status_msg_time = get_current_time_secs();
    }

    /* file */
    fn edit_or_open(&mut self) {
        if self.cfg.file_name != "" {
            let file = File::open(&self.cfg.file_name).unwrap();
            let reader = BufReader::new(file);

            for line in reader.lines() {
                // store raw content
                let line = line.unwrap().replace("\r", "").replace("\n", "");
                self.row.push(line.as_bytes().to_vec());
                // store render content
                self.render.push(self.get_render_vec(&self.row[self.rows_num as usize]));

                self.rows_num += 1;
            }
            self.dirty = false;
        }
    }

    fn save_file(&mut self) {
        if self.cfg.file_name == "" {
            self.cfg.file_name = self.read_file_name();
            if self.cfg.file_name == "" {
                self.set_status_msg(format_args!("Save aborted"));
                return;
            }
        } else {}
        let mut file = File::create(&self.cfg.file_name).unwrap();
        let mut bytes: u32 = 0;
        for i in 0..self.rows_num {
            file.write_all(&self.row[i as usize]).unwrap();
            file.write_all(b"\n").unwrap();
            bytes += self.row[i as usize].len() as u32 + 1;
        }
        file.flush().unwrap();
        self.dirty = false;
        self.set_status_msg(format_args!("{} bytes written to disk", bytes));
    }

    fn read_file_name(&mut self) -> String {
        let mut file_name = String::new();
        loop {
            self.set_status_msg(format_args!("Save as:{} (ESC to cancel)", &file_name));
            self.refresh_screen();

            let key = Keys::read_key();
            match key {
                Keys::BACKSPACE | Keys::DEL_KEY | Keys::CTL_H => {
                    if file_name.len() != 0 {
                        file_name.remove(file_name.len() - 1);
                    }
                }
                Keys::ESC => {
                    self.set_status_msg(format_args!(""));
                    return String::new();
                }
                Keys::ENTER => {
                    self.set_status_msg(format_args!(""));
                    break;
                }
                Keys::NORMAL(c) => {
                    if !c.is_ascii_control() && c < 128 {
                        file_name.push(c as char);
                    }
                }
                _ => {}
            }
        }
        file_name
    }

    /* helper */
    fn get_render_vec(&self, line: &Vec<u8>) -> Vec<u8> {
        let mut render_vec = Vec::new();
        for c in line {
            if *c == '\t' as u8 {
                render_vec.push(' ' as u8);
                while render_vec.len() % TABLE_STOP as usize != 0 {
                    render_vec.push(' ' as u8);
                }
            } else {
                render_vec.push(c.clone());
            }
        }
        render_vec
    }

    fn row_cx_to_rx(&mut self) {
        self.rx = 0;
        if self.cy < self.rows_num {
            for i in 0..self.cx {
                if self.row[self.cy as usize][i as usize] == '\t' as u8 {
                    self.rx += TABLE_STOP as u32 - (self.rx % TABLE_STOP as u32)
                } else {
                    self.rx += 1;
                }
            }
        }
    }

    fn get_dirty_status(&self) -> &'static str {
        if self.dirty {
            "(modified)"
        } else {
            ""
        }
    }
}
