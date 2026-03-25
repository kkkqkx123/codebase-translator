fn main() {
    let name = "test_space";
    let message = format!("图空间 '{}' 不存在", name);
    println!("{}", message);

    let error = format!("配置文件 '{}' 未找到", "config.toml");
    eprintln!("{}", error);

    let info = format!("成功连接到数据库 '{}'", "postgres");
    log::info!("{}", info);

    // Test with multiple placeholders
    let result = format!("用户 '{}' 在 {} 时访问了资源", "admin", "2024-01-01");
    println!("{}", result);

    // Test format_args!
    let msg = format_args!("处理文件 '{}' 大小为 {} 字节", "data.txt", 1024);
    println!("{}", msg);

    // Test print! macros
    print!("正在处理: {}", name);
    println!("完成: {}", name);
    eprint!("错误: {}", name);
    eprintln!("警告: {}", name);
}