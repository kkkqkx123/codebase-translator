// Test file for string format preservation
// This file tests various Rust string literal types

fn test_regular_strings() {
    let s1 = "Hello, world!";
    let s2 = "Simple text";
    println!("{}", s1);
}

fn test_raw_strings() {
    let raw1 = r"Hello, world!";
    let raw2 = r#"Hello, "world"!"#;
    let raw3 = r##"Hello, #"world"#!"##;
    println!("{}", raw1);
}

fn test_byte_strings() {
    let bytes = b"Hello, world!";
    println!("{:?}", bytes);
}

fn test_format_strings() {
    let name = "Alice";
    let formatted = format!("Hello, {}!", name);
    println!("{}", formatted);
}

fn test_error_messages() {
    panic!("Something went wrong");
    log::error!("Failed to connect");
}

fn test_multiline_strings() {
    let multiline = "Line 1\nLine 2\nLine 3";
    println!("{}", multiline);
}

fn test_strings_with_escapes() {
    let escaped = "Hello\nWorld\t!";
    let quoted = "He said \"Hello\"";
    println!("{}", escaped);
}
