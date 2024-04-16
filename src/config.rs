use termion::terminal_size;

pub struct EditorCfg {
    pub screen_row: u32,
    pub screen_col: u32,
    pub file_name: String,
}

impl EditorCfg {
    pub fn new(file_name: String) -> Self {
        let size = terminal_size().unwrap();
        EditorCfg {
            screen_col: size.0 as u32,
            screen_row: (size.1 - 2) as u32,
            file_name,
        }
    }

    pub fn get_file_name(&self) -> &str {
        if self.file_name == "" {
            "[No Name]"
        } else {
            &self.file_name
        }
    }
}