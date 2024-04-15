use termion::terminal_size;

pub struct EditorCfg<'a> {
    pub screen_row: u16,
    pub screen_col: u16,
    pub file_name: &'a str,
}

impl<'a> EditorCfg<'a> {
    pub fn new(file_name: &'a str) -> Self {
        let size = terminal_size().unwrap();
        EditorCfg {
            screen_col: size.0,
            screen_row: size.1,
            file_name,
        }
    }
}