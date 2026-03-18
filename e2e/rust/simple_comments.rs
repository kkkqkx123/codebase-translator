//! Simple comments fixture for testing
//!
//! This file contains various types of Rust comments for testing extraction.

/// This is an outer doc comment for the module
/// It spans multiple lines

/// Function with documentation
/// # Examples
/// ```
/// let result = add(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    // This is a line comment
    a + b
}

// Standalone line comment

/* Block comment
   spanning multiple
   lines */

fn main() {
    // Comment inside function
    let x = 5; // Inline comment
    
    /* Another block
       comment */
    
    println!("Hello, world!");
}

// TODO: This should be filtered
// FIXME: This too
// NOTE: And this
// XXX: Also this
// HACK: Finally this

// Normal comment that should be translated
