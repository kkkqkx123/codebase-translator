//! Translation applier modules
//!
//! This module provides functionality for applying translations to file content,
//! separated into single-line and multi-line appliers for better maintainability.

pub mod line;
pub mod multiline;

pub use line::LineApplier;
pub use multiline::MultilineApplier;
