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

    pub fn get_color_highlight(line: &Vec<u8>) -> Vec<Highlight> {
        let mut r = Vec::new();
        for c in line {
            if c.is_ascii_digit() { r.push(Highlight::Number); } else { r.push(Highlight::Normal); }
        }
        r
    }

    pub fn highlight_line(line: &Vec<u8>, hl: &Vec<Highlight>, start: usize, end: usize) -> String {
        let mut hl_str = String::new();

        for i in start..end {
            if i == start || hl[i] != hl[i - 1] {
                hl_str.push_str(hl[i].to_color());
            }
            hl_str.push(line[i] as char);
        }

        hl_str.push_str(Highlight::Normal.to_color());
        hl_str
    }

    pub fn copy_highlight(des: &mut Vec<Highlight>, src: &Vec<Highlight>) {
        des.clear();
        for h in src {
            des.push(h.clone());
        }
    }
}