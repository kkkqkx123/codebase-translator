//! Content filtering module
//!
//! This module provides a layered filtering system for determining which content
//! should be translated. The filtering is organized into multiple checks, each
//! handling a specific aspect:
//!
//! # Filter Checks
//!
//! 1. **Length Check** (`checks::length`) - O(1) constant-time checks
//!    - Empty text detection
//!    - Maximum length validation
//!
//! 2. **Language Check** (`checks::language`) - O(k) quick detection
//!    - Source language matching
//!    - CJK character detection
//!
//! 3. **Pattern Check** (`checks::pattern`) - O(n) regex matching
//!    - Keyword exclusion
//!    - Pattern exclusion/inclusion
//!    - Placeholder detection
//!    - Code pattern detection
//!
//! 4. **Content Check** (`checks::content`) - O(len) deep analysis
//!    - Symbol-only text detection
//!
//! # Usage
//!
//! ```rust,ignore
//! use codebase_translate::parser::filtering::{CompositeFilter, FilterConfig};
//!
//! let config = FilterConfig::default();
//! let filter = CompositeFilter::new(config).unwrap();
//!
//! assert!(filter.should_translate("Hello world"));
//! assert!(!filter.should_translate("TODO: fix this"));
//! ```

pub mod composite;
pub mod config;
pub mod checks;
pub mod traits;

// Re-export main types
pub use composite::CompositeFilter as ContentFilter;
pub use composite::{
    default_filter, from_project_config, from_project_config_with_translator,
};
pub use config::FilterConfig;
pub use traits::Filter;

// Re-export checks for advanced usage
pub use checks::{LengthFilter, ContentFilter as ContentCheckFilter, LanguageFilter, PatternFilter};
