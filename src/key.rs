use crate::key::Keys::{ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, NORMAL};

#[allow(non_camel_case_types)]
pub enum Keys {
    ARROW_LEFT,
    ARROW_RIGHT,
    ARROW_UP,
    ARROW_DOWN,
    PAGE_UP,
    PAGE_DOWN,
    NORMAL(u8),
}

// impl Keys {
//     pub fn parse_key(key: u16) -> Self {
//         match key {
//             1000 => ARROW_LEFT,
//             1001 => ARROW_RIGHT,
//             1002 => ARROW_UP,
//             1003 => ARROW_DOWN,
//             _ => NORMAL(key),
//         }
//     }
// }