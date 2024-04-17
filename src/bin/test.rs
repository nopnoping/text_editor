fn main() {
    let mut vec = vec![1, 2, 3];
    vec.clear();

    vec.push(4);
    vec.push(5);
    vec.push(6);

    println!("{:?}", vec[0]);
}
