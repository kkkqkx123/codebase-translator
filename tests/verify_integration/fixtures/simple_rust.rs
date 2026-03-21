// Simple Rust file with comments and strings
// This is a line comment
// Another line comment

/// This is a doc comment
/// It spans multiple lines
/// with more documentation
fn main() {
    println!("Hello, world!");
    let x = 42;
    
    let message = "This is a string literal";
    let error_msg = "Error: something went wrong";
    
    log::info!("Processing data");
    log::warn!("Warning: deprecated feature");
    
    let formatted = format!("Value: {}", x);
    
    match x {
        42 => println!("Found the answer"),
        _ => println!("Unknown value"),
    }
}

// Another comment at the end
fn helper_function() -> String {
    let result = "Helper result";
    result.to_string()
}

