// 用于测试的 Rust 字符串字面量
fn main() {
    let message = "Hello, world!";
    let greeting = "Welcome to the application";
    let error_msg = "An error occurred while processing";
    
    println!("{}", message);
}

fn show_message() {
    println!("Displaying message");
}

fn handle_error() {
    println!("Error: operation failed");
}

fn format_string(name: &str) -> String {
    format!("Hello, {}!", name)
}
