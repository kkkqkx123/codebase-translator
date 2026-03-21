// Rust documentation comments for testing

/// This is a module-level documentation note
/// This is the structure document annotation
/// Field Documentation
        pub value: i32,
    }
    
    impl Example {
        /// Create a new example instance
        pub fn new(value: i32) -> Self {
            Example { value }
        }
        
        /// return value
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
