// Mixed content Rust file for language filtering tests

// This is an English comment
const ENGLISH_CONST: i32 = 42;

// 这是一个中文注释
const CHINESE_CONST: i32 = 100;

/*
 * Multi-line English comment
 * with multiple lines
 */

/*
 * 多行中文注释
 * 包含多行内容
 */

fn english_function() {
    println!("This is English output");
}

fn chinese_function() {
    println!("这是中文输出");
}

fn mixed_function() {
    // English comment with some Chinese words
    let message = "Hello 世界";
    println!("{}", message);
    
    // Chinese comment with English words
    let greeting = "你好 World";
    println!("{}", greeting);
}

/// English documentation comment
/// @param value - The value to process
fn process_english(value: i32) -> i32 {
    value * 2
}

/// 中文文档注释
/// @param value - 要处理的值
fn process_chinese(value: i32) -> i32 {
    value * 2
}

