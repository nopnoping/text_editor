pub enum Highlight {
    Normal,
    Number,
}

impl Highlight {
    pub fn to_color(&self) -> &'static str {
        match self {
            Highlight::Number => "\x1b[31m",
            Highlight::Normal => "\x1b[37m",
        }
    }
}