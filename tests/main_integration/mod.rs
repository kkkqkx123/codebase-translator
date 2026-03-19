//! Main Integration Tests
//!
//! End-to-end integration tests for the main module.
//! These tests verify the complete translation workflow including:
//! - Configuration loading
//! - File scanning
//! - Text extraction
//! - Translation
//! - File writing
//! - Cache management
//! - Backup creation
//! - Logging

pub mod e2e_tests;
pub mod logger_tests;
pub mod project_logging_tests;
