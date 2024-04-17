#[derive(PartialEq)]
#[derive(Clone)]
pub enum Highlight {
    Normal,
    Number,
    Match,
}

impl Highlight {
    pub fn to_color(&self) -> &'static str {
        match self {
            Highlight::Number => "\x1b[31m",
            Highlight::Normal => "\x1b[39m",
            Highlight::Match => "\x1b[34m",
        }
    }

    pub fn copy_highlight(des: &mut Vec<Highlight>, src: &Vec<Highlight>) {
        des.clear();
        for h in src {
            des.push(h.clone());
        }
    }
}