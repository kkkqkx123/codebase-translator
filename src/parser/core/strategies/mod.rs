//! Strategy implementations module
//!
//! This module provides concrete implementations of the `ExtractionStrategy` trait.
//! These implementations are separated from the trait definitions in `abstraction::strategy`
//! to maintain a clean separation between interface and implementation.

pub mod config_based;
pub mod exported_only;
pub mod strategy_impl;

pub use config_based::ConfigBasedStrategy;
pub use exported_only::ExportedOnlyStrategy;
pub use strategy_impl::ExtractionStrategyImpl;
