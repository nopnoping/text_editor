#[derive(PartialEq)]
#[derive(Clone)]
pub enum Highlight {
    Normal,
    Comment,
    MComment,
    Keyword1,
    Keyword2,
    String,
    Number,
    Match,
}

impl Highlight {
    pub fn to_color(&self) -> &'static str {
        match self {
            Highlight::Number => "\x1b[31m",
            Highlight::Comment | Highlight::MComment => "\x1b[36m",
            Highlight::Normal => "\x1b[39m",
            Highlight::Match => "\x1b[34m",
            Highlight::String => "\x1b[35m",
            Highlight::Keyword1 => "\x1b[33m",
            Highlight::Keyword2 => "\x1b[32m",
        }
    }

    pub fn copy_highlight(des: &mut Vec<Highlight>, src: &Vec<Highlight>) {
        des.clear();
        for h in src {
            des.push(h.clone());
        }
    }
}