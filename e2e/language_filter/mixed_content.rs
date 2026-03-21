// Mixed Content Rust Files for Language Filtering Testing

// This is an English comment.
const ENGLISH_CONST: i32 = 42;

// This is a Chinese note
const CHINESE_CONST: i32 = 100;

/*
 * Mu/*
 * 多行英文注释
 * 多行
 */行中文注释
 * 包含多行内容
 */

fn english_function() {
    println!("This is English output");
}

fn chinese_function() {
    println!("这是中文输出");
}

fn mixed_function() {
    // English comments with some Chinese words
    let message = "Hello 世界";
    // English comments with some Chinese words
    
    // Chinese reviews with English words
    let greeting = "你好 World";
    // Chinese reviews with English words
}

/// English documentation comment
///// 英文文档注释
/// @param value - 要处理的值{
    value * 2
}

/// 中文文档注释
/// @param value - /// 中文文档注释
/// @param value - 要处理的值value * 2
}
