//! Comment prefix extraction
//!
//! This module provides functionality for extracting and formatting
//! comment prefixes from source code lines.

/// Extract comment prefix from a line (e.g., "// ", "/// ", "* ", "/* ")
pub fn extract_comment_prefix(line: &str) -> String {
    let trimmed = line.trim_start();
    let leading_whitespace = &line[..(line.len() - trimmed.len())];

    if trimmed.starts_with("///") {
        format!("{}/// ", leading_whitespace)
    } else if trimmed.starts_with("//!") {
        format!("{}//! ", leading_whitespace)
    } else if trimmed.starts_with("//") {
        format!("{}// ", leading_whitespace)
    } else if trimmed.starts_with("/**") {
        format!("{}/** ", leading_whitespace)
    } else if trimmed.starts_with("/*") {
        format!("{}/* ", leading_whitespace)
    } else if trimmed.starts_with('*') {
        format!("{}* ", leading_whitespace)
    } else if trimmed.starts_with('#') {
        format!("{}# ", leading_whitespace)
    } else {
        leading_whitespace.to_string()
    }
}

/// Represents different types of comment prefixes
#[derive(Debug, Clone, PartialEq)]
pub enum CommentPrefix {
    /// Line comment: //
    Line,
    /// Documentation comment: ///
    Doc,
    /// Module documentation: //!
    ModuleDoc,
    /// Block comment start: /*
    Block,
    /// Block documentation: /**
    BlockDoc,
    /// Block comment middle: *
    Star,
    /// Hash comment: # (e.g., Python, YAML)
    Hash,
    /// No recognized prefix
    None,
}

impl CommentPrefix {
    /// Detect the comment prefix type from a line
    pub fn detect(line: &str) -> (Self, String) {
        let trimmed = line.trim_start();
        let leading_whitespace = &line[..(line.len() - trimmed.len())];

        if trimmed.starts_with("///") {
            (Self::Doc, format!("{}/// ", leading_whitespace))
        } else if trimmed.starts_with("//!") {
            (Self::ModuleDoc, format!("{}//! ", leading_whitespace))
        } else if trimmed.starts_with("//") {
            (Self::Line, format!("{}// ", leading_whitespace))
        } else if trimmed.starts_with("/**") {
            (Self::BlockDoc, format!("{}/** ", leading_whitespace))
        } else if trimmed.starts_with("/*") {
            (Self::Block, format!("{}/* ", leading_whitespace))
        } else if trimmed.starts_with('*') {
            (Self::Star, format!("{}* ", leading_whitespace))
        } else if trimmed.starts_with('#') {
            (Self::Hash, format!("{}# ", leading_whitespace))
        } else {
            (Self::None, leading_whitespace.to_string())
        }
    }

    /// Check if this is a block comment marker
    pub fn is_block_marker(&self) -> bool {
        matches!(self, Self::Block | Self::BlockDoc)
    }

    /// Check if this is a documentation comment
    pub fn is_doc_comment(&self) -> bool {
        matches!(self, Self::Doc | Self::ModuleDoc | Self::BlockDoc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_comment_prefix() {
        assert_eq!(extract_comment_prefix("    // comment"), "    // ");
        assert_eq!(extract_comment_prefix("/// doc"), "/// ");
        assert_eq!(extract_comment_prefix("//! module doc"), "//! ");
        assert_eq!(extract_comment_prefix("  /* block"), "  /* ");
        assert_eq!(extract_comment_prefix(" * middle"), " * ");
        assert_eq!(extract_comment_prefix("  # python"), "  # ");
        assert_eq!(extract_comment_prefix("no comment"), "");
    }

    #[test]
    fn test_comment_prefix_detect() {
        let (prefix, formatted) = CommentPrefix::detect("    // comment");
        assert_eq!(prefix, CommentPrefix::Line);
        assert_eq!(formatted, "    // ");

        let (prefix, formatted) = CommentPrefix::detect("/// doc");
        assert_eq!(prefix, CommentPrefix::Doc);
        assert_eq!(formatted, "/// ");

        let (prefix, _) = CommentPrefix::detect("/* block");
        assert_eq!(prefix, CommentPrefix::Block);
        assert!(prefix.is_block_marker());

        let (prefix, _) = CommentPrefix::detect("/// doc");
        assert!(prefix.is_doc_comment());
    }
}
