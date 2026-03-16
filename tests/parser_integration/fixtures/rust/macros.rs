//! Macros fixture for testing
//!
//! This file contains various Rust macros for testing string extraction.

fn error_examples() {
    // Error macros
    panic!("This is a panic message");
    panic!("Formatted panic: {}", 42);
    
    assert!(true, "Assertion message");
    assert_eq!(1, 1, "Equality assertion");
    assert_ne!(1, 2, "Inequality assertion");
    
    todo!("Not yet implemented");
    unimplemented!("This is unimplemented");
    unreachable!("This should never be reached");
}

fn format_examples() {
    // Format macros
    let s = format!("Hello, {}!", "world");
    let s = format!("Value: {}", 42);
    let s = format!("Multiple: {} and {}", 1, 2);
    
    print!("Print without newline");
    println!("Print with newline");
    println!("Print with value: {}", 123);
    
    eprint!("Error print");
    eprintln!("Error print line: {}", "error");
}

fn log_examples() {
    // Log macros (also format macros)
    println!("Simple log message");
    println!("Formatted log: {}", "data");
    eprintln!("Error log message");
}

fn debug_examples() {
    // Debug macro
    dbg!("Debug message");
    dbg!(42);
    
    let value = 100;
    dbg!(&value);
}

fn write_examples() {
    use std::io::Write;
    
    let mut buffer = Vec::new();
    write!(&mut buffer, "Write to buffer: {}", "data").unwrap();
    writeln!(&mut buffer, "Write line: {}", 42).unwrap();
}

fn raw_strings() {
    // Raw strings
    let s1 = r"This is a raw string";
    let s2 = r#"This is a raw string with "quotes""#;
    let s3 = r##"This is a raw string with #"quotes"#"##;
    let s4 = r###"This is a raw string with ##"quotes"##"###;
    
    // Regular strings with escapes
    let s5 = "Line 1\nLine 2\tTabbed";
    let s6 = "Quote: \"hello\"";
    let s7 = "Backslash: \\";
}

fn format_args() {
    // Various format arguments
    println!("{}", "positional");
    println!("{0} and {1}", "first", "second");
    println!("{name}", name = "value");
    println!("{:.2}", 3.14159);
    println!("{:>10}", "right");
    println!("{:<10}", "left");
    println!("{:^10}", "center");
}
