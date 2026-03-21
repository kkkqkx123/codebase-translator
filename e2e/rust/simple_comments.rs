// 用于测试的 Rust 简单注释

// 这是一个单行注释
const VALUE: i32 = 42;

/*
这是一个多行注释
多行文本
*/

fn test() -> i32 {
    // 函数内部的另一个注释
    VALUE
}

fn main() {
    let result = test();
    println!("Result: {}", result);
}
