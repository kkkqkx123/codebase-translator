// Complex test file with various patterns
// This file demonstrates different types of extractable content

/// Module documentation
/// This module handles complex data processing
/// with multiple functions and patterns
mod complex_module {
    /// Function documentation
    /// This function processes user input
    /// and returns a formatted response
    pub fn process_input(input: &str) -> String {
        format!("Processed: {}", input)
    }

    /// Error handling function
    /// Returns error messages for validation failures
    pub fn validate_input(input: &str) -> Result<(), String> {
        if input.is_empty() {
            Err("Error: input cannot be empty".to_string())
        } else {
            Ok(())
        }
    }
}

// Configuration constants
const MAX_SIZE: usize = 1024;
const DEFAULT_TIMEOUT: u64 = 30;

// Global configuration
const CONFIG_PATH: &str = "/etc/app/config.json";

fn main() {
    // Initialize logging
    log::info!("Application starting");
    log::debug!("Configuration loaded from {}", CONFIG_PATH);
    log::warn!("Using default timeout of {} seconds", DEFAULT_TIMEOUT);

    // Process user input
    let user_input = "test data";
    let result = complex_module::process_input(user_input);
    println!("Result: {}", result);

    // Validate input
    match complex_module::validate_input(user_input) {
        Ok(()) => log::info!("Input validation passed"),
        Err(e) => log::error!("Validation failed: {}", e),
    }

    // Format string with multiple variables
    let status = "success";
    let count = 42;
    let message = format!("Status: {}, Count: {}", status, count);

    // Error handling with custom messages
    let error_msg = "Error: operation failed after 3 attempts";
    let warning_msg = "Warning: deprecated API usage detected";

    // Complex string with special characters
    let special = "Special chars: < > & \" ' \\ /";

    // Long string that might need truncation
    let long_text = "This is a very long text that should be properly formatted in output without breaking layout and should demonstrate the truncation feature";

    println!("{}", message);
    println!("{}", special);
    println!("{}", long_text);
}

