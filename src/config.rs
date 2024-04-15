use std::cmp::min;
use termion::terminal_size;
use crate::key::Keys;


pub struct EditorCfg {
    pub screen_row: u16,
    pub screen_col: u16,
    pub cx: u16,
    pub cy: u16,
}

impl EditorCfg {
    pub fn new() -> Self {
        let size = terminal_size().unwrap();
        EditorCfg {
            screen_col: size.0,
            screen_row: size.1,
            cx: 0,
            cy: 0,
        }
    }

    pub fn move_cursor(&mut self, key: Keys) {
        match key {
            Keys::ARROW_UP => {
                self.cy = self.cy.saturating_sub(1);
            }
            Keys::ARROW_DOWN => {
                self.cy = min(self.cy.wrapping_add(1), self.screen_row);
            }
            Keys::ARROW_LEFT => {
                self.cx = self.cx.saturating_sub(1);
            }
            Keys::ARROW_RIGHT => {
                self.cx = min(self.cx.wrapping_add(1), self.screen_col);
            }
            _ => {}
        }
    }
}