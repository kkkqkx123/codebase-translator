//! Rust String Literals Fixture
//!
//! Tests extraction of string literals from various Rust code patterns.

use std::fmt;

// Error message strings
fn error_examples() {
    panic!("系统遇到致命错误，无法继续执行");
    panic!("Memory allocation failed");
    
    assert!(false, "断言失败：数值不匹配");
    assert_eq!(1, 2, "Expected 1, got 2");
    
    todo!("实现用户认证功能");
    unimplemented!("Database connection not yet implemented");
    unreachable!("This code should never be reached");
}

// Console/log output strings
fn log_examples() {
    println!("应用程序启动成功");
    println!("Application started successfully");
    
    eprintln!("错误：无法打开文件");
    eprintln!("Error: Failed to open configuration file");
    
    print!("Loading data... ");
    println!("完成");
}

// Format strings
fn format_examples() {
    let name = "张三";
    let age = 25;
    
    let msg1 = format!("用户 {} 的年龄是 {}", name, age);
    let msg2 = format!("User {} is {} years old", name, age);
    let msg3 = format!("Processing item {0} of {1}", 1, 10);
    
    // Mixed language format strings
    let msg4 = format!("Hello {}, 欢迎使用系统", name);
}

// String variable definitions
fn string_definitions() {
    let chinese_msg = "这是一个中文消息";
    let english_msg = "This is an English message";
    let mixed_msg = "Hello 世界";
    
    let raw_string = r"Raw string with \n no escapes";
    let raw_chinese = r"原始字符串：无转义";
    
    const ERROR_MSG: &str = "配置错误";
    const SUCCESS_MSG: &str = "Operation completed";
}

// UI/Display strings
struct User {
    name: String,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "用户: {}", self.name)
    }
}

// Error types with messages
#[derive(Debug)]
enum AppError {
    NotFound,
    InvalidInput,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound => write!(f, "找不到请求的资源"),
            AppError::InvalidInput => write!(f, "输入参数无效"),
        }
    }
}

// HTTP/API related strings
fn api_responses() {
    let success = "{ \"code\": 200, \"message\": \"操作成功\" }";
    let error = "{ \"code\": 500, \"message\": \"服务器内部错误\" }";
    let not_found = "{ \"code\": 404, \"message\": \"Resource not found\" }";
}

// SQL query strings
fn sql_queries() {
    let query1 = "SELECT * FROM users WHERE name = '张三'";
    let query2 = "INSERT INTO logs (message) VALUES ('User login')";
    let query3 = "UPDATE settings SET value = '中文配置'";
}

// Regex patterns as strings
fn regex_patterns() {
    let pattern1 = r"\d{4}-\d{2}-\d{2}";
    let pattern2 = r"[\u4e00-\u9fa5]+";  // Chinese characters
    let pattern3 = r"^\w+@\w+\.\w+$";
}

// File path strings
fn file_paths() {
    let path1 = "/home/user/文档/file.txt";
    let path2 = "C:\\Users\\Admin\\Documents\\file.txt";
    let path3 = "./config/设置.json";
}

// Configuration strings
fn config_strings() {
    let db_host = "localhost";
    let db_name = "生产数据库";
    let app_name = "MyApplication";
    let version = "1.0.0";
}

fn main() {
    println!("程序启动");
    println!("Program started");
}
