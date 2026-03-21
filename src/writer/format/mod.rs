//! Text formatting utilities
//!
//! This module provides utility functions for text formatting,
//! including comment prefix extraction and text replacement.

pub mod prefix;
pub mod replacement;

pub use prefix::{extract_comment_prefix, CommentPrefix};
pub use replacement::{byte_to_char_pos, replace_in_raw_match};
