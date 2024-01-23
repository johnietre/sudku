fn main() {
    let v = vec![Some(1), Some(2), None, Some(3)];
    println!("{:?}", v.into_iter().map(|o| {
        println!("{:?}", o);
        o
    }).collect::<Option<Vec<_>>>());
}
