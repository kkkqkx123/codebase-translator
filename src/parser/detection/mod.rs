//! Language detection module
//!
//! This module provides language detection capabilities for determining
//! the language of text content. It supports multiple scripts and languages.

pub mod language;

pub use language::{default_detector, LanguageDetector, LanguageInfo, Script};
