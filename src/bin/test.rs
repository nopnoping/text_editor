fn main() {
    let t = Test::A;
    let r: u8 = t as u8;
    let a = 0_u8;
    let t = r as Test;
}

enum Test {
    A = 0,
    B,
    C,
}