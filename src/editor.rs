use std::cmp::{min};
use std::fmt::{Arguments, format};
use std::fs::File;
use std::io::{BufRead, BufReader, Stdout, stdout, Write};
use std::iter::repeat;
use memchr::memmem;
use termion::raw::{IntoRawMode, RawTerminal};
use crate::config::EditorCfg;
use crate::highlight::Highlight;
use crate::key::Keys;
use crate::syntax::Syntax;
use crate::{syntax, util};
use crate::util::get_current_time_secs;

const VERSION: &str = "0.0.1";
const TABLE_STOP: u8 = 4;
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
    hl: Vec<Vec<Highlight>>,

    dirty: bool,
    quit_time: u8,
    status_msg: String,
    status_msg_time: u64,

    last_match: i64,
    find_direction: i8,
    saved_match_hl: Vec<Highlight>,

    syntax: Option<&'static Syntax>,

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
            hl: Vec::new(),

            dirty: false,
            quit_time: QUIT_TIMES,
            status_msg: String::from(""),
            status_msg_time: 0,

            last_match: -1,
            find_direction: 1,
            saved_match_hl: Vec::new(),

            syntax: None,
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

        self.set_status_msg(format_args!("HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find"));
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
            Keys::CTL_F => self.find_world(),
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
                self.cy = min(self.cy.wrapping_add(1), self.rows_num - 1);
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
                    } else if self.cy < self.rows_num - 1 { // move to left at the next of a line
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

    /* promotion read for status bar*/
    fn promotion_read<F>(&mut self, s: String, callback: F) -> String
        where
            F: Fn(&mut Self, &Keys, &str),
    {
        let mut user_input = String::new();
        loop {
            self.set_status_msg(format_args!("{}", &s.replace("{}", &user_input)));
            self.refresh_screen();

            let key = Keys::read_key();
            match key {
                Keys::BACKSPACE | Keys::DEL_KEY | Keys::CTL_H => {
                    if user_input.len() != 0 {
                        user_input.remove(user_input.len() - 1);
                    }
                }
                Keys::ESC => {
                    self.set_status_msg(format_args!(""));
                    callback(self, &key, &user_input);
                    return String::new();
                }
                Keys::ENTER => {
                    self.set_status_msg(format_args!(""));
                    callback(self, &key, &user_input);
                    break;
                }
                Keys::NORMAL(c) if !c.is_ascii_control() && c < 128 => user_input.push(c as char),
                _ => {}
            }
            callback(self, &key, &user_input);
        }
        user_input
    }

    /* screen refresh */
    fn refresh_screen(&mut self) {
        self.scroll();

        self.stdout.write_all(b"\x1b[?25l").unwrap();
        self.stdout.write_all(b"\x1b[H").unwrap();

        self.draw_rows();
        self.draw_status_bar();
        self.draw_status_msg();

        if self.rows_num == 0 {
            self.stdout.write_all(b"\x1b[H").unwrap();
        } else {
            self.stdout.write_all(
                format!("\x1b[{};{}H", self.cy - self.row_off + 1, self.rx - self.col_off + 5).as_bytes()
            ).unwrap();
        }
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

            if file_row < self.rows_num {
                self.draw_file(file_row);
            } else if self.rows_num == 0
                && (r == self.cfg.screen_row / 3 || r == self.cfg.screen_row / 3 + 1) {
                self.draw_hello(r);
            } else {
                self.stdout.write_all(b"~").unwrap();
            }

            // clean
            self.stdout.write_all(b"\x1b[K").unwrap();
            // to next line
            self.stdout.write_all(b"\r\n").unwrap();
        }
    }

    fn draw_file(&mut self, file_row: u32) {
        // line number
        self.stdout.write_all(format!("{:^4}", file_row + 1).as_bytes()).unwrap();
        // file row content
        let row = &self.render[file_row as usize];
        if self.col_off < row.len() as u32 {
            // syntax highlighting
            let start = self.col_off as usize;
            let end = min(row.len(), (self.col_off + self.cfg.screen_col - 4) as usize);
            let r = self.highlight_line(row, &self.hl[file_row as usize], start, end);
            self.stdout.write(r.as_bytes()).unwrap();
        }
    }

    fn draw_hello(&mut self, r: u32) {
        let mut welcome = format!("My editor -- version:{}", VERSION);
        if r == self.cfg.screen_row / 3 + 1 {
            welcome = "HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find".to_string();
        }

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
    }

    fn draw_status_bar(&mut self) {
        self.stdout.write_all(b"\x1b[7m").unwrap();

        let status = format!("{:20} - {} lines {}", self.cfg.get_file_name(), self.rows_num, self.get_dirty_status());
        let line = match self.syntax {
            None => format!("no ft | {}/{}", self.cy + 1, self.rows_num),
            Some(syntax) => format!("{} | {}/{}", syntax.file_type, self.cy + 1, self.rows_num),
        };
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
            if let Ok(file) = File::open(&self.cfg.file_name) {
                let reader = BufReader::new(file);
                self.select_syntax();

                for line in reader.lines() {
                    let line = line.unwrap().replace("\r", "").replace("\n", "");
                    self.insert_new_row(self.rows_num as usize, line.as_bytes().to_vec());
                }

                self.dirty = false;
            } else {
                self.set_status_msg(format_args!("File not exist. Open a empty file. Please use Ctrl-s to Save file!!"))
            }
        }
    }

    fn insert_char(&mut self, c: u8) {
        if self.cy > self.rows_num {
            panic!("err cy");
        }
        if self.cy == self.rows_num {
            self.insert_new_row(self.rows_num as usize, Vec::new());
        }

        if let Ok(_) = self.insert_u8_to_row(self.cy as usize, self.cx as usize, c) {
            self.dirty = true;
            self.cx = self.cx.wrapping_add(1);
        }
    }

    fn delete_char(&mut self) {
        if self.cy >= self.rows_num { return; }
        if self.cx == 0 && self.cy == 0 { return; }

        if self.cx > 0 {
            if let Ok(_) = self.delete_u8_of_row(self.cy as usize, self.cx as usize - 1) {
                self.dirty = true;
                self.cx = self.cx.saturating_sub(1);
            }
        } else { // delete at the beginning of line
            let d = self.delete_row(self.cy as usize);
            let old_len = self.insert_line_to_row(self.cy as usize - 1, d);

            self.dirty = true;
            self.cy = self.cy.saturating_sub(1);
            self.cx = old_len as u32;
        }
    }

    fn insert_new_line(&mut self) {
        if self.cx == 0 {
            self.insert_new_row(self.cy as usize, Vec::new());
        } else {
            let d = self.delete_content_to_end(self.cy as usize, self.cx as usize);
            self.insert_new_row(self.cy as usize + 1, d);
        }

        self.dirty = true;
        self.cx = 0;
        self.cy = self.cy.wrapping_add(1);
    }

    fn save_file(&mut self) {
        if self.cfg.file_name == "" {
            self.cfg.file_name = self.promotion_read(String::from("Save as: {} (ESC to cancel)"), |_, _, _| {});
            if self.cfg.file_name == "" {
                self.set_status_msg(format_args!("Save aborted"));
                return;
            }
            self.select_syntax();
            self.re_build_row_highlight();
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

    fn find_world(&mut self) {
        let save_cx = self.cx;
        let save_cy = self.cy;
        let save_col_off = self.col_off;
        let save_row_off = self.row_off;

        let world = self.promotion_read(String::from("Search: {} (Use ESC/Arrows/Enter)"), Editor::find_world_callback);
        if world == "" {
            self.cx = save_cx;
            self.cy = save_cy;
            self.col_off = save_col_off;
            self.row_off = save_row_off;
        }
    }

    fn find_world_callback(&mut self, keys: &Keys, world: &str) {
        if self.last_match != -1 {
            Highlight::copy_highlight(&mut self.hl[self.last_match as usize], &self.saved_match_hl);
        }

        match keys {
            Keys::ARROW_UP => self.find_direction = -1,
            Keys::ARROW_DOWN => self.find_direction = 1,
            Keys::ENTER | Keys::ESC => {
                self.last_match = -1;
                self.find_direction = 1;
                return;
            }
            _ => {
                self.last_match = -1;
                self.find_direction = 1;
            }
        }

        let mut cur = self.last_match;
        for _ in 0..self.rows_num {
            cur += self.find_direction as i64;
            if cur >= self.rows_num as i64 { cur = 0 };
            if cur < 0 { cur = self.rows_num as i64 - 1; }
            if let Option::Some(index) = memmem::find(&self.row[cur as usize], world.as_bytes()) {
                self.last_match = cur;
                Highlight::copy_highlight(&mut self.saved_match_hl, &self.hl[cur as usize]);

                self.cy = cur as u32;
                self.cx = index as u32;
                self.row_off = self.rows_num;
                self.row_cx_to_rx();
                for i in 0..world.len() {
                    self.hl[cur as usize][self.rx as usize + i] = Highlight::Match;
                }
                break;
            }
        }
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

    fn select_syntax(&mut self) {
        if self.cfg.file_name == "" { return; }

        let ext = util::get_file_type(&self.cfg.file_name);
        for s in syntax::HLDB.iter() {
            for t in s.file_math.iter() {
                if *t == ext {
                    self.syntax = Some(s);
                    break;
                }
            }
        }
    }

    fn highlight_line(&self, line: &Vec<u8>, hl: &Vec<Highlight>, start: usize, end: usize) -> String {
        let mut hl_str = String::new();

        for i in start..end {
            if line[i].is_ascii_control() {
                hl_str.push_str("\x1b[7m");
                if line[i] <= 26 {
                    hl_str.push('@');
                } else {
                    hl_str.push('?');
                }
                hl_str.push_str("\x1b[m");
                continue;
            }
            if i == start || hl[i] != hl[i - 1] {
                hl_str.push_str(hl[i].to_color());
            }
            hl_str.push(line[i] as char);
        }

        hl_str.push_str(Highlight::Normal.to_color());
        hl_str
    }

    fn build_row_highlight(&self, line: &Vec<u8>) -> Vec<Highlight> {
        match self.syntax {
            None => {
                let mut r = Vec::new();
                for _ in 0..line.len() { r.push(Highlight::Normal) }
                r
            }
            Some(syntax) => syntax.syntax_highlight(line),
        }
    }

    fn re_build_row_highlight(&mut self) {
        for i in 0..self.rows_num {
            self.hl[i as usize] = self.build_row_highlight(&self.render[i as usize]);
        }
    }

    /* row modify helper */
    fn insert_new_row(&mut self, index: usize, line: Vec<u8>) {
        self.row.insert(index, line);
        self.render.insert(index, self.get_render_vec(&self.row[index]));
        self.hl.insert(index, self.build_row_highlight(&self.render[index]));
        self.rows_num += 1;
    }

    fn insert_u8_to_row(&mut self, y: usize, x: usize, c: u8) -> Result<(), &'static str> {
        if x > self.row[y].len() { return Err("err cx"); }
        self.row[y].insert(x, c);
        self.update_render_and_hl(y);
        Ok(())
    }

    fn insert_line_to_row(&mut self, y: usize, line: Vec<u8>) -> usize {
        let old_len = self.row[y].len();
        for c in line {
            self.row[y].push(c);
        }
        self.update_render_and_hl(y);
        old_len
    }

    fn delete_row(&mut self, index: usize) -> Vec<u8> {
        let r = self.row.remove(index);
        self.render.remove(index);
        self.hl.remove(index);
        self.rows_num -= 1;
        r
    }

    fn delete_u8_of_row(&mut self, y: usize, x: usize) -> Result<(), &'static str> {
        if x >= self.row[y].len() { return Err("err x"); }
        self.row[y].remove(x);
        self.update_render_and_hl(y);
        Ok(())
    }

    fn delete_content_to_end(&mut self, index: usize, start: usize) -> Vec<u8> {
        let len = self.row[index].len();
        let r = self.row[index].drain(start..len).collect();
        self.update_render_and_hl(index);
        r
    }

    fn update_render_and_hl(&mut self, y: usize) {
        self.render[y] = self.get_render_vec(&self.row[y]);
        self.hl[y] = self.build_row_highlight(&self.render[y]);
    }
}
