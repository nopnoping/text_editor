use lazy_static::lazy_static;
use crate::highlight::Highlight;
use crate::util;

pub struct Syntax {
    pub file_type: &'static str,
    pub file_math: Vec<&'static str>,
}

lazy_static! {
pub static ref HLDB: Vec<Syntax> = vec![
    Syntax {
        file_type: "c",
        file_math: vec![".c", ".h", ".cpp"],
    }
];
}

impl Syntax {
    pub fn syntax_highlight(&self, line: &Vec<u8>) -> Vec<Highlight> {
        let mut r = Vec::new();

        let mut i = 0;
        let mut prev_sep = true;
        while i < line.len() {
            let c = line[i];

            if (c.is_ascii_digit() && (prev_sep || r[i - 1] == Highlight::Number))
                || (c == '.' as u8 && i > 0 && r[i - 1] == Highlight::Number) {
                r.push(Highlight::Number);
            } else {
                r.push(Highlight::Normal);
            }

            i += 1;
            prev_sep = util::is_separator(c);
        }
        r
    }
}