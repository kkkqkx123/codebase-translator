// 用于测试的 Rust 文档注释

/// 这是模块级文档注释
/// 这是结构文档注释
/// 字段文档
        pub value: i32,
    }
    
    impl Example {
        /// 创建一个新的示例实例
        pub fn new(value: i32) -> Self {
            Example { value }
        }
        
        /// 返回值
        pub fn get_value(&self) -> i32 {
            self.value
        }
    }
}

/// 功能文档
/// 
/// # 参数
/// 
/// * `name` - 要问候的名字
/// 
/// # 返回
/// 
/// 问候信息
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
