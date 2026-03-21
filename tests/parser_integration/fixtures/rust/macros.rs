// Rust macros for testing
macro_rules! log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

macro_rules! error {
    ($msg:expr) => {
        eprintln!("Error: {}", $msg);
    };
}

fn main() {
    log!("Application started");
    log!("Processing data: {}", 42);
    error!("Something went wrong");
    
    println!("Normal output");
}

