// 这是一个简单的Rust文件，用于测试翻译功能
// 包含中文注释和文档字符串

/// 计算两个数的和
/// 
/// # Arguments
/// 
/// * `a` - 第一个数字
/// * `b` - 第二个数字
/// 
/// # Examples
/// 
/// ```
/// let result = add(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// 计算两个数的差
/// 
/// # Arguments
/// 
/// * `a` - 被减数
/// * `b` - 减数
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// 乘法运算
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

// 这是一个简单的结构体
pub struct Calculator {
    // 计算器名称
    name: String,
}

impl Calculator {
    /// 创建新的计算器实例
    /// 
    /// # Arguments
    /// 
    /// * `name` - 计算器名称
    pub fn new(name: &str) -> Self {
        Calculator {
            name: name.to_string(),
        }
    }

    /// 获取计算器名称
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

fn main() {
    println!("测试翻译功能");
    let result = add(10, 20);
    println!("结果: {}", result);
}
