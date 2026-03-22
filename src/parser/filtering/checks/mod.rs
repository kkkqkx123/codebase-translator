//! Filter checks module
//!
//! This module provides layered filter implementations.
//! Each check handles a specific aspect of filtering, ordered by complexity.

pub mod content;
pub mod length;
pub mod pattern;

// Language detection module with tiered detection strategies
pub mod language;

pub use content::ContentFilter;
pub use language::LanguageFilter;
pub use length::LengthFilter;
pub use pattern::PatternFilter;

// Re-export language detection types for advanced usage
pub use language::{DetectionStrategy, LanguageDetector, LanguageInfo, QuickDetector, SampledDetector, Script};
