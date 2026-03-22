//! Strategy implementations module
//!
//! This module provides concrete implementations of the `ExtractionStrategy` trait.

pub mod config_based;
pub mod exported_only;

pub use config_based::ConfigBasedStrategy;
pub use exported_only::ExportedOnlyStrategy;
