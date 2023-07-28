use std::env;

fn main() {
    for argument in env::args() {
        println!("{argument}");
    }
    println!("Hello, world!");
}
