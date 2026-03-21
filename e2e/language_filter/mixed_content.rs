// 用于语言过滤测试的混合内容 Rust 文件

// 这是一条英语评论
const ENGLISH_CONST: i32 = 42;

// 这是一个中文注释
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
    // 带有一些中文词汇的英文评论
    let message = "Hello 世界";
    // 带有一些中文词汇的英文评论
    
    // 带英文单词的中文评论
    let greeting = "你好 World";
    // 带英文单词的中文评论
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
