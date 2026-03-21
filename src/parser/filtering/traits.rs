//! Filter traits module
//!
//! This module defines the core trait for all filter implementations.

/// Core filter trait
///
/// All filters in the system implement this trait.
/// Filters are composable and can be chained together.
pub trait Filter: Send + Sync {
    /// Check if content should be translated
    ///
    /// Returns true if the content passes the filter, false otherwise.
    fn should_translate(&self, text: &str) -> bool;

    /// Get filter name for debugging/logging
    fn name(&self) -> &str;
}

/// Filter with context trait
///
/// Extended trait for filters that need additional context
/// beyond just the text content.
pub trait ContextualFilter: Send + Sync {
    /// Context type for the filter
    type Context;

    /// Check if content should be translated with context
    fn should_translate_with_context(&self, text: &str, ctx: &Self::Context) -> bool;

    /// Get filter name for debugging/logging
    fn name(&self) -> &str;
}
