use lazy_static::lazy_static;
use crate::highlight::Highlight;
use crate::util;

pub struct Syntax {
    pub file_type: &'static str,
    pub file_math: Vec<&'static str>,
    single_comment_start: &'static str,
}

lazy_static! {
    pub static ref HLDB: Vec<Syntax> = vec![
        Syntax {
            file_type: "c",
            file_math: vec![".c", ".h", ".cpp"],
            single_comment_start : "//",
        }
    ];
}

impl Syntax {
    pub fn syntax_highlight(&self, line: &Vec<u8>) -> Vec<Highlight> {
        let mut r = Vec::new();

        let mut i = 0;
        let mut prev_sep = true;
        let mut in_string = 0_u8;
        while i < line.len() {
            let c = line[i];
            // single comment
            if in_string == 0 && i + 1 < line.len() && &line[i..i + 2] == self.single_comment_start.as_bytes() {
                while i < line.len() {
                    r.push(Highlight::Comment);
                    i += 1;
                }
                break;
            }

            // string highlight
            if in_string > 0 {
                r.push(Highlight::String);
                if c == '\\' as u8 && i < line.len() - 1 {
                    r.push(Highlight::String);
                    i += 2;
                    continue;
                }
                if c == in_string { in_string = 0; }
                i += 1;
                prev_sep = true;
                continue;
            } else {
                if c == '"' as u8 || c == '\'' as u8 {
                    in_string = c;
                    r.push(Highlight::String);
                    i += 1;
                    continue;
                }
            }


            // digit highlight
            if (c.is_ascii_digit() && (prev_sep || r[i - 1] == Highlight::Number))
                || (c == '.' as u8 && i > 0 && r[i - 1] == Highlight::Number) {
                r.push(Highlight::Number);
                i += 1;
                prev_sep = false;
                continue;
            }

            r.push(Highlight::Normal);
            i += 1;
            prev_sep = util::is_separator(c);
        }
        r
    }
}