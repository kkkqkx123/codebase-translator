//! Generic extractor trait and types

use std::collections::HashMap;

use tree_sitter::Node;

use crate::core::error::Result;
use crate::core::models::{Position, TranslationUnit};
use crate::parser::strategy::StrategyNodeType;

/// Extraction type categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionType {
    /// Comment extraction
    Comment,
    /// Docstring extraction
    Docstring,
    /// String literal extraction
    StringLiteral,
    /// Error message extraction
    ErrorMessage,
    /// Format string extraction
    FormatString,
    /// Log message extraction
    LogMessage,
    /// Debug message extraction
    DebugMessage,
    /// Custom extraction type
    Custom(&'static str),
}

impl ExtractionType {
    /// Convert to string representation
    pub fn as_str(&self) -> &str {
        match self {
            ExtractionType::Comment => "comment",
            ExtractionType::Docstring => "docstring",
            ExtractionType::StringLiteral => "string",
            ExtractionType::ErrorMessage => "error",
            ExtractionType::FormatString => "format",
            ExtractionType::LogMessage => "log",
            ExtractionType::DebugMessage => "debug",
            ExtractionType::Custom(s) => s,
        }
    }
}

/// Extraction candidate before filtering
#[derive(Debug, Clone)]
pub struct ExtractionCandidate {
    /// Extracted text content
    pub text: String,
    /// Start position
    pub start_pos: Position,
    /// End position
    pub end_pos: Position,
    /// Node type for strategy filtering
    pub node_type: StrategyNodeType,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ExtractionCandidate {
    /// Create a new extraction candidate
    pub fn new(
        text: String,
        start_pos: Position,
        end_pos: Position,
        node_type: StrategyNodeType,
    ) -> Self {
        Self {
            text,
            start_pos,
            end_pos,
            node_type,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Convert to TranslationUnit
    pub fn into_translation_unit(
        self,
        id: String,
        unit_node_type: crate::core::models::NodeType,
    ) -> TranslationUnit {
        TranslationUnit::new(id, unit_node_type, self.text, self.start_pos, self.end_pos)
    }
}

/// Generic extractor trait
pub trait Extractor: Send + Sync {
    /// Get the extraction type
    fn extraction_type(&self) -> ExtractionType;

    /// Extract candidates from the syntax tree
    fn extract(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<ExtractionCandidate>>;
}
