fn main() {
    let t = Test::A('b');

    match t {
        Test::A(t) if t == 'a' => {
            print!("hello");
        }
        _ => { print!("www"); }
    }
}

enum Test {
    A(char)
}