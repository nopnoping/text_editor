use std::io;
use std::io::Read;

macro_rules! ctrl_key {
    ($k:expr) => {($k as u8) & 0x1f};
}

#[allow(non_camel_case_types)]
pub enum Keys {
    BACKSPACE,
    ARROW_LEFT,
    ARROW_RIGHT,
    ARROW_UP,
    ARROW_DOWN,
    DEL_KEY,
    HOME_KEY,
    END_KEY,
    PAGE_UP,
    PAGE_DOWN,
    QUIT,
    CTL_H,
    CTL_L,
    CTL_S,
    CTL_F,
    ENTER,
    ESC,
    NORMAL(u8),
}

impl Keys {
    pub fn read_key() -> Keys {
        let mut c = [0; 4];
        let size = io::stdin().lock().read(&mut c).unwrap();
        let r = c[0] as char;

        if r == '\x1b' {
            if size < 3 { return Keys::ESC; }
            let r1 = c[1] as char;
            let r2 = c[2] as char;

            if r1 == '[' {
                if r2 >= '0' && r2 <= '9' && size >= 4 {
                    io::stdin().lock().read(&mut c).unwrap();
                    let r3 = c[3] as char;
                    if r3 == '~' {
                        return match r2 {
                            '1' => Keys::HOME_KEY,
                            '3' => Keys::DEL_KEY,
                            '4' => Keys::END_KEY,
                            '5' => Keys::PAGE_UP,
                            '6' => Keys::PAGE_DOWN,
                            '7' => Keys::HOME_KEY,
                            '8' => Keys::END_KEY,
                            _ => Keys::ESC,
                        };
                    }
                } else {
                    return match r2 {
                        'A' => Keys::ARROW_UP,
                        'B' => Keys::ARROW_DOWN,
                        'C' => Keys::ARROW_RIGHT,
                        'D' => Keys::ARROW_LEFT,
                        'H' => Keys::HOME_KEY,
                        'F' => Keys::END_KEY,
                        _ => Keys::ESC,
                    };
                }
            } else if r1 == 'O' {
                return match r2 {
                    'H' => Keys::HOME_KEY,
                    'F' => Keys::END_KEY,
                    _ => Keys::ESC,
                };
            }

            return Keys::ESC;
        }

        if r == ctrl_key!('q') as char {
            return Keys::QUIT;
        }

        if r == ctrl_key!('h') as char {
            return Keys::CTL_H;
        }

        if r == ctrl_key!('l') as char {
            return Keys::CTL_L;
        }
        if r == ctrl_key!('s') as char {
            return Keys::CTL_S;
        }
        if r == ctrl_key!('f') as char {
            return Keys::CTL_F;
        }

        if r == '\r' {
            return Keys::ENTER;
        }

        if c[0] == 127 {
            return Keys::BACKSPACE;
        }

        return Keys::NORMAL(c[0]);
    }
}