// Rust annotations for testing

// This is a one-line comment
const VALUE: i32 = 42;

/*
 * This is a multi-line comment
 * Multi-line text */

fn test() -> i32 {
    // Another comment inside the function
    VALUE
}

fn main() {
    let result = test();
    println!("Result: {}", result);
}
