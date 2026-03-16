//! Mixed Language Content Fixture
//!
//! Contains both Chinese and English content to test language filtering.

/// This is an English-only doc comment
/// It should NOT be extracted when source is zh and target is en
fn english_only_function() {
    let msg = "This is an English string";
    println!("Hello World");
}

/// 这是一个纯中文的文档注释
/// 当源语言为zh，目标语言为en时，应该被提取
fn chinese_only_function() {
    let msg = "这是一个中文字符串";
    println!("你好世界");
}

/// Mixed content: 你好 World
/// This function has both languages
/// 包含两种语言的内容
fn mixed_content_function() {
    let msg1 = "Hello 世界";
    let msg2 = "欢迎使用 Rust programming language";
    println!("混合内容 mixed content");
}

// Pure English comment
fn another_english_function() {
    let x = "test value";
    let y = "another value";
}

// 纯中文注释
fn another_chinese_function() {
    let x = "测试值";
    let y = "另一个值";
}

fn error_examples() {
    // English errors - should NOT be extracted
    panic!("System error occurred");
    assert!(false, "Assertion failed");
    
    // Chinese errors - should be extracted
    panic!("系统发生错误");
    assert!(false, "断言失败");
    
    // Mixed errors - should be extracted (contains Chinese)
    panic!("Error: 操作失败");
}

fn log_examples() {
    // English logs - should NOT be extracted
    println!("Application started");
    log::info!("Processing request");
    
    // Chinese logs - should be extracted
    println!("应用程序已启动");
    log::info!("正在处理请求");
    
    // Mixed logs - should be extracted
    println!("User 张三 logged in");
}

const ENGLISH_CONST: &str = "English constant value";
const CHINESE_CONST: &str = "中文常量值";
const MIXED_CONST: &str = "Mixed 混合 value";

enum Message {
    Success,
    Error,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // English - should NOT be extracted
            Message::Success => write!(f, "Operation successful"),
            // Chinese - should be extracted
            Message::Error => write!(f, "操作失败"),
        }
    }
}
