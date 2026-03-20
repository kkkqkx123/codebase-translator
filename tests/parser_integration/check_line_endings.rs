//! Check line endings in fixture file

use std::fs;
use std::path::PathBuf;

const FIXTURES_DIR: &str = "tests/main_integration/fixtures";

#[test]
fn check_simple_rust_line_endings() {
    let fixture_path = PathBuf::from(FIXTURES_DIR).join("simple_rust.rs");
    let content = fs::read_to_string(&fixture_path).expect("Failed to read fixture");
    
    // Check for CRLF
    let crlf_count = content.matches("\r\n").count();
    let lf_count = content.matches('\n').count() - crlf_count;
    
    println!("CRLF count: {}", crlf_count);
    println!("LF count: {}", lf_count);
    
    // Check specific line around multiply function (line 30-31)
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > 30 {
        println!("Line 30: {:?}", lines[29]);
        println!("Line 31: {:?}", lines[30]);
    }
    
    // Check bytes around position 503-520 (where /// 乘法运算 is)
    if content.len() > 520 {
        let bytes: &[u8] = content.as_bytes();
        println!("Bytes 500-525: {:?}", &bytes[500..525]);
    }
}
