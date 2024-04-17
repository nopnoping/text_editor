use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_time_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub fn is_separator(c: u8) -> bool {
    if c == 32 || c == '\0' as u8 || ",.()+-/*=~%<>[];".contains(c as char) {
        true
    } else {
        false
    }
}

pub fn get_file_type(file_name: &str) -> &str {
    match file_name.rfind('.') {
        None => "",
        Some(i) => &file_name[i..],
    }
}