//! Parser coordinator types

/// Indicates which type of parser will handle a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserType {
    /// A tree-sitter parser at the given index
    TreeSitter(usize),
    /// The regex-based fallback parser
    Regex,
}
