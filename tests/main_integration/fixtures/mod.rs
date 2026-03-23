//! 通用基础设施模块
//!
//! 这个模块包含了所有通用的基础设施代码，包括：
//! - 基础工具和ID生成
//! - 内存管理
//! - 线程管理

pub mod id;

// Re-export commonly used types and functions for easy use by other modules
pub use id::*;
