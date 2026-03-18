//! Doc comments fixture for testing
//!
//! This file tests various forms of Rust documentation comments.

//! Inner doc comment for the crate root
//! More inner documentation

/// Outer doc comment for module
/// Continues here

/// Struct with documentation
/// 
/// # Fields
/// 
/// * `name` - The name field
/// * `age` - The age field
pub struct Person {
    /// The person's name
    name: String,
    /// The person's age
    age: u32,
}

impl Person {
    /// Creates a new Person
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name to use
    /// * `age` - The age to use
    /// 
    /// # Examples
    /// 
    /// ```
    /// let person = Person::new("Alice", 30);
    /// ```
    pub fn new(name: &str, age: u32) -> Self {
        Self {
            name: name.to_string(),
            age,
        }
    }
    
    /// Gets the person's name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /** Block doc comment
        This is a block-style documentation comment
        It can span multiple lines
    */
    pub fn age(&self) -> u32 {
        self.age
    }
}

/*! Inner block doc comment
   For the module
*/

/// Trait with documentation
trait Greetable {
    /// Returns a greeting
    fn greet(&self) -> String;
}

impl Greetable for Person {
    /// Greets the person
    fn greet(&self) -> String {
        format!("Hello, my name is {}", self.name)
    }
}

/// Enum with documentation
#[derive(Debug)]
pub enum Status {
    /// Active status
    Active,
    /// Inactive status
    Inactive,
    /// Pending status
    Pending,
}

/// Macro with documentation
/// 
/// # Usage
/// 
/// ```
/// say_hello!();
/// ```
#[macro_export]
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
}
