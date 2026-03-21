// Rust simple comments for testing

// This is a single-line comment
const VALUE: i32 = 42;

/*
This is a multi-line comment
with multiple lines of text
*/

fn test() -> i32 {
    // Another comment inside function
    VALUE
}

fn main() {
    let result = test();
    println!("Result: {}", result);
}
