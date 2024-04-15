use termion::terminal_size;

pub struct EditorCfg {
    pub screen_size: (u16, u16),
}

impl EditorCfg {
    pub fn new() -> Self {
        EditorCfg {
            screen_size: terminal_size().unwrap(),
        }
    }
}