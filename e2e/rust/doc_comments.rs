// Rust doc comments for testing

/// This is a module-level documentation comment
mod example {
    /// This is a struct documentation comment
    pub struct Example {
        /// Field documentation
        pub value: i32,
    }
    
    impl Example {
        /// Creates a new Example instance
        pub fn new(value: i32) -> Self {
            Example { value }
        }
        
        /// Returns the value
        pub fn get_value(&self) -> i32 {
            self.value
        }
    }
}

/// Function documentation
/// 
/// # Arguments
/// 
/// * `name` - A name to greet
/// 
/// # Returns
/// 
/// A greeting message
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
