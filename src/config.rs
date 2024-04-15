use termion::terminal_size;

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

    pub fn move_cursor(&mut self, key: char) {
        match key {
            'k' => {
                self.cy = self.cy.wrapping_sub(1);
            }
            'j' => {
                self.cy = self.cy.wrapping_add(1);
            }
            'h' => {
                self.cx = self.cx.wrapping_sub(1);
            }
            'l' => {
                self.cx = self.cx.wrapping_add(1);
            }
            _ => {}
        }
    }
}