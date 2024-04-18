use lazy_static::lazy_static;
use crate::highlight::Highlight;
use crate::util;

pub struct Syntax {
    pub file_type: &'static str,
    pub file_math: Vec<&'static str>,
    single_comment_start: &'static str,
    keyword: Vec<&'static str>,
    multi_comment_start: &'static str,
    multi_comment_end: &'static str,
}

lazy_static! {
    pub static ref HLDB: Vec<Syntax> = vec![
        Syntax {
            file_type: "c",
            file_math: vec![".c", ".h", ".cpp"],
            single_comment_start : "//",
            keyword: vec!["switch", "if", "while", "for", "break", "continue", "return", "else",
                "struct", "union", "typedef", "static", "enum", "class", "case",
                "int|", "long|", "double|", "float|", "char|", "unsigned|", "signed|",
                "void|"],
            multi_comment_start: "/*",
            multi_comment_end: "*/",
        }
    ];
}

impl Syntax {
    pub fn syntax_highlight(&self, line: &Vec<u8>) -> Vec<Highlight> {
        let mut r = Vec::new();

        let mut i = 0;
        let mut prev_sep = true;
        let mut in_string = 0_u8;
        'line: while i < line.len() {
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

            // keyword
            if prev_sep {
                for keyword in self.keyword.iter() {
                    let mut len = keyword.len();
                    let mut k2 = false;
                    if keyword.as_bytes()[len - 1] == '|' as u8 {
                        len -= 1;
                        k2 = true;
                    }

                    if i + len == line.len() || (i + len < line.len() && util::is_separator(line[i + len])) {
                        if &line[i..i + len] == keyword[0..len].as_bytes() {
                            for _ in 0..len {
                                if k2 { r.push(Highlight::Keyword2); } else { r.push(Highlight::Keyword1); }
                            }
                            i += len;
                            continue 'line;
                        }
                    }
                }
            }


            r.push(Highlight::Normal);
            i += 1;
            prev_sep = util::is_separator(c);
        }
        r
    }

    pub fn is_multi_comment_start(&self, line: &Vec<u8>) -> bool {
        if line.len() >= self.multi_comment_start.len() &&
            &line[0..self.multi_comment_start.len()] == self.multi_comment_start.as_bytes() {
            true
        } else {
            false
        }
    }

    pub fn is_multi_comment_end(&self, line: &Vec<u8>) -> bool {
        if line.len() >= self.multi_comment_end.len() &&
            &line[line.len() - self.multi_comment_end.len()..] == self.multi_comment_end.as_bytes() {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_keyword() {
        let s = "*/";
        assert_eq!(true, HLDB[0].is_multi_comment_end(&s.as_bytes().to_vec()));
        // HLDB[0].syntax_highlight(&s.as_bytes().to_vec());
    }
}