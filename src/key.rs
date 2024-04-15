use std::io;
use std::io::Read;

#[allow(non_camel_case_types)]
pub enum Keys {
    ARROW_LEFT,
    ARROW_RIGHT,
    ARROW_UP,
    ARROW_DOWN,
    DEL_KEY,
    HOME_KEY,
    END_KEY,
    PAGE_UP,
    PAGE_DOWN,
    NORMAL(u8),
}

impl Keys {
    pub fn read_key() -> Keys {
        let mut c = [0; 1];
        io::stdin().lock().read(&mut c).unwrap();
        let r = c[0] as char;

        if r == '\x1b' {
            io::stdin().lock().read(&mut c).unwrap();
            let r1 = c[0] as char;
            io::stdin().lock().read(&mut c).unwrap();
            let r2 = c[0] as char;

            if r1 == '[' {
                if r2 >= '0' && r2 <= '9' {
                    io::stdin().lock().read(&mut c).unwrap();
                    let r3 = c[0] as char;
                    if r3 == '~' {
                        return match r2 {
                            '1' => Keys::HOME_KEY,
                            '3' => Keys::DEL_KEY,
                            '4' => Keys::END_KEY,
                            '5' => Keys::PAGE_UP,
                            '6' => Keys::PAGE_DOWN,
                            '7' => Keys::HOME_KEY,
                            '8' => Keys::END_KEY,
                            _ => Keys::NORMAL('\x1b' as u8),
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
                        _ => Keys::NORMAL('\x1b' as u8),
                    };
                }
            } else if r1 == 'O' {
                return match r2 {
                    'H' => Keys::HOME_KEY,
                    'F' => Keys::END_KEY,
                    _ => Keys::NORMAL('\x1b' as u8),
                };
            }

            return Keys::NORMAL('\x1b' as u8);
        }

        return Keys::NORMAL(c[0]);
    }
}