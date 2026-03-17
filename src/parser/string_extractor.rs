//! String literal extractor
//!
//! Extracts string literals from code based on AST patterns and configurable rules.
//! Uses a conservative approach: only extracts strings from explicitly defined contexts.
//!
//! # Deprecated
//!
//! This module is currently not used in the codebase and is marked as deprecated.
//! It may be removed in a future version.

use std::collections::HashSet;
use std::sync::Arc;

use regex::Regex;
use tree_sitter::{Node, Tree};

use crate::core::error::{Result, TranslateError};
use crate::core::models::{Position, TranslationUnit};
use crate::parser::core::query_executor::QueryExecutor;
use crate::parser::core::StringProcessor;
use crate::parser::strategy::{ExtractionStrategy, ExtractionStrategyImpl, StrategyNodeType};

/// Category of string literal extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StringCategory {
    /// Error handling: panic, Error, throw, etc.
    ErrorHandling,
    /// Output/logging: print, console, logging, etc.
    Output,
    /// Variable assignments
    Variables,
    /// Object properties
    Properties,
    /// Other/uncategorized
    Other,
}

impl StringCategory {
    /// Get category name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ErrorHandling => "error_handling",
            Self::Output => "output",
            Self::Variables => "variables",
            Self::Properties => "properties",
            Self::Other => "other",
        }
    }
}

/// Configuration for string extraction
#[derive(Debug, Clone)]
pub struct StringExtractorConfig {
    /// Enabled categories
    pub enabled_categories: HashSet<StringCategory>,
    /// Patterns by category
    pub patterns: CategoryPatterns,
    /// Variable name patterns (regex) to extract
    pub variable_patterns: Vec<String>,
    /// Object property names to extract
    pub property_patterns: Vec<String>,
    /// Custom regex patterns for special cases
    pub custom_regex_patterns: Vec<(String, Regex, usize, StringCategory)>, // (name, regex, group, category)
}

/// Patterns organized by category
#[derive(Debug, Clone, Default)]
pub struct CategoryPatterns {
    pub error_handling: Vec<String>,
    pub output: Vec<String>,
}

impl StringExtractorConfig {
    /// Create config from project config and language
    pub fn from_project_config(
        categories: &crate::config::project::StringLiteralCategories,
        language: &str,
    ) -> Result<Self> {
        use crate::config::project::get_default_patterns_for_language;

        let mut enabled_categories = HashSet::new();
        let mut patterns = CategoryPatterns::default();

        // Error handling
        if categories.error_handling {
            enabled_categories.insert(StringCategory::ErrorHandling);
            let error_patterns = get_default_patterns_for_language(language, "error_handling");
            patterns.error_handling = error_patterns;
        }

        // Output
        if categories.output {
            enabled_categories.insert(StringCategory::Output);
            let output_patterns = get_default_patterns_for_language(language, "output");
            patterns.output = output_patterns;
        }

        // Variables
        if categories.variables {
            enabled_categories.insert(StringCategory::Variables);
        }

        // Properties
        if categories.properties {
            enabled_categories.insert(StringCategory::Properties);
        }

        Ok(Self {
            enabled_categories,
            patterns,
            variable_patterns: Vec::new(),
            property_patterns: Vec::new(),
            custom_regex_patterns: Vec::new(),
        })
    }

    /// Check if a category is enabled
    pub fn is_category_enabled(&self, category: StringCategory) -> bool {
        self.enabled_categories.contains(&category)
    }

    /// Get all enabled function patterns
    pub fn get_all_function_patterns(&self) -> Vec<String> {
        let mut all = Vec::new();
        all.extend(self.patterns.error_handling.clone());
        all.extend(self.patterns.output.clone());
        all
    }
}

impl Default for StringExtractorConfig {
    fn default() -> Self {
        Self {
            enabled_categories: HashSet::new(),
            patterns: CategoryPatterns::default(),
            variable_patterns: Vec::new(),
            property_patterns: Vec::new(),
            custom_regex_patterns: Vec::new(),
        }
    }
}

/// Extracted string metadata
#[derive(Debug, Clone)]
pub struct ExtractedString {
    /// The string content
    pub content: String,
    /// Start position in the source
    pub start_pos: Position,
    /// End position in the source
    pub end_pos: Position,
    /// Context where the string was found
    pub context: ExtractionContext,
    /// Category of the extracted string
    pub category: StringCategory,
    /// Node type for classification
    pub node_type: StrategyNodeType,
}

/// Context where the string was extracted from
#[derive(Debug, Clone)]
pub enum ExtractionContext {
    /// Function call argument
    FunctionCall { name: String },
    /// Variable assignment
    VariableAssignment { name: String },
    /// Object property value
    ObjectProperty { key: String },
    /// Custom pattern match
    CustomPattern { name: String },
}

/// String literal extractor
pub struct StringExtractor {
    config: StringExtractorConfig,
    strategy: Arc<ExtractionStrategyImpl>,
    string_processor: StringProcessor,
    compiled_var_patterns: Vec<Regex>,
}

impl StringExtractor {
    /// Create a new string extractor
    pub fn new(
        config: StringExtractorConfig,
        strategy: Arc<ExtractionStrategyImpl>,
    ) -> Result<Self> {
        // Compile variable patterns
        let compiled_var_patterns: Result<Vec<Regex>> = config
            .variable_patterns
            .iter()
            .map(|p| {
                Regex::new(p).map_err(|e| {
                    TranslateError::Parse(format!("Invalid variable pattern '{}': {}", p, e))
                })
            })
            .collect();

        Ok(Self {
            config,
            strategy,
            string_processor: StringProcessor::new(),
            compiled_var_patterns: compiled_var_patterns?,
        })
    }

    /// Extract strings from a tree-sitter tree
    pub fn extract(
        &self,
        tree: &Tree,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let mut units = Vec::new();
        let root = tree.root_node();

        // 1. Extract function call strings
        units.extend(self.extract_function_call_strings(&root, content, file_path)?);

        // 2. Extract variable assignment strings
        units.extend(self.extract_variable_strings(&root, content, file_path)?);

        // 3. Extract object property strings
        units.extend(self.extract_property_strings(&root, content, file_path)?);

        // 4. Apply custom regex patterns
        units.extend(self.apply_custom_patterns(content, file_path)?);

        // 5. Deduplicate by position
        let unique_units = self.deduplicate_units(units);

        Ok(unique_units)
    }

    /// Extract strings from function calls
    fn extract_function_call_strings(
        &self,
        root: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let all_patterns = self.config.get_all_function_patterns();
        if all_patterns.is_empty() {
            return Ok(Vec::new());
        }

        // Build query for function calls with string arguments
        let query = r#"
            (call_expression
              function: [
                (identifier) @func
                (member_expression
                  object: (identifier) @obj
                  property: (property_identifier) @prop)
              ]
              arguments: (arguments
                [(string) @str
                 (template_string) @template]))
        "#;

        let executor = QueryExecutor::from_string(&tree_sitter_javascript::LANGUAGE.into(), query)?;
        let matches = executor.execute(root, content)?;

        let mut units = Vec::new();
        let mut current_func: Option<String> = None;
        let mut current_obj: Option<String> = None;

        for m in matches {
            match m.capture_name.as_str() {
                "func" => {
                    current_func = Some(m.text.to_string());
                }
                "obj" => {
                    current_obj = Some(m.text.to_string());
                }
                "prop" => {
                    if let Some(ref obj) = current_obj {
                        current_func = Some(format!("{}.{}", obj, m.text));
                    }
                }
                "str" | "template" => {
                    if let Some(ref func_name) = current_func {
                        // Check if function matches any pattern and get category
                        if let Some(category) = self.get_function_category(func_name) {
                            // Check if this category is enabled
                            if self.config.is_category_enabled(category) {
                                let text = self.string_processor.clean_string_literal(m.text);

                                if let Some(unit) = self.create_unit(
                                    &text,
                                    m.start_pos,
                                    m.end_pos,
                                    file_path,
                                    StrategyNodeType::StringLiteral,
                                    category,
                                )? {
                                    units.push(unit);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(units)
    }

    /// Get category for a function name
    fn get_function_category(&self, func_name: &str) -> Option<StringCategory> {
        // Check each category
        if self
            .config
            .patterns
            .error_handling
            .iter()
            .any(|p| func_name == p || func_name.ends_with(&format!(".{}", p)))
        {
            return Some(StringCategory::ErrorHandling);
        }

        if self
            .config
            .patterns
            .output
            .iter()
            .any(|p| func_name == p || func_name.ends_with(&format!(".{}", p)))
        {
            return Some(StringCategory::Output);
        }

        None
    }

    /// Extract strings from variable assignments
    fn extract_variable_strings(
        &self,
        root: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        if !self.config.is_category_enabled(StringCategory::Variables)
            || self.compiled_var_patterns.is_empty()
        {
            return Ok(Vec::new());
        }

        // Query for variable declarations with string initializers
        let query = r#"
            (variable_declarator
              name: (identifier) @var_name
              value: [(string) @str
                      (template_string) @template])
        "#;

        let executor = QueryExecutor::from_string(&tree_sitter_javascript::LANGUAGE.into(), query)?;
        let matches = executor.execute(root, content)?;

        let mut units = Vec::new();
        let mut current_var: Option<String> = None;

        for m in matches {
            match m.capture_name.as_str() {
                "var_name" => {
                    current_var = Some(m.text.to_string());
                }
                "str" | "template" => {
                    if let Some(ref var_name) = current_var {
                        // Check if variable name matches any pattern
                        if self.matches_variable_pattern(var_name) {
                            let text = self.string_processor.clean_string_literal(m.text);

                            if let Some(unit) = self.create_unit(
                                &text,
                                m.start_pos,
                                m.end_pos,
                                file_path,
                                StrategyNodeType::StringLiteral,
                                StringCategory::Variables,
                            )? {
                                units.push(unit);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(units)
    }

    /// Extract strings from object properties
    fn extract_property_strings(
        &self,
        root: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        if !self.config.is_category_enabled(StringCategory::Properties)
            || self.config.property_patterns.is_empty()
        {
            return Ok(Vec::new());
        }

        // Query for object properties with string values
        let query = r#"
            (pair
              key: (property_identifier) @key
              value: [(string) @str
                      (template_string) @template])
        "#;

        let executor = QueryExecutor::from_string(&tree_sitter_javascript::LANGUAGE.into(), query)?;
        let matches = executor.execute(root, content)?;

        let mut units = Vec::new();
        let mut current_key: Option<String> = None;

        for m in matches {
            match m.capture_name.as_str() {
                "key" => {
                    current_key = Some(m.text.to_string());
                }
                "str" | "template" => {
                    if let Some(ref key) = current_key {
                        // Check if property name matches any pattern
                        if self
                            .config
                            .property_patterns
                            .iter()
                            .any(|p| key.contains(p))
                        {
                            let text = self.string_processor.clean_string_literal(m.text);

                            if let Some(unit) = self.create_unit(
                                &text,
                                m.start_pos,
                                m.end_pos,
                                file_path,
                                StrategyNodeType::StringLiteral,
                                StringCategory::Properties,
                            )? {
                                units.push(unit);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(units)
    }

    /// Apply custom regex patterns
    fn apply_custom_patterns(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let mut units = Vec::new();

        for (_name, regex, group, category) in &self.config.custom_regex_patterns {
            // Check if this category is enabled
            if !self.config.is_category_enabled(*category) {
                continue;
            }

            for cap in regex.captures_iter(content) {
                if let Some(matched) = cap.get(*group) {
                    let text = matched.as_str().to_string();
                    let start_pos = Position {
                        offset: matched.start(),
                        line: 0, // Would need to calculate from offset
                        column: 0,
                    };
                    let end_pos = Position {
                        offset: matched.end(),
                        line: 0,
                        column: 0,
                    };

                    if let Some(unit) = self.create_unit(
                        &text,
                        start_pos,
                        end_pos,
                        file_path,
                        StrategyNodeType::StringLiteral,
                        *category,
                    )? {
                        units.push(unit);
                    }
                }
            }
        }

        Ok(units)
    }

    /// Check if variable name matches any configured pattern
    fn matches_variable_pattern(&self, var_name: &str) -> bool {
        self.compiled_var_patterns
            .iter()
            .any(|re| re.is_match(var_name))
    }

    /// Create a translation unit if content passes filters
    fn create_unit(
        &self,
        content: &str,
        start_pos: Position,
        end_pos: Position,
        file_path: &str,
        node_type: StrategyNodeType,
        _category: StringCategory,
    ) -> Result<Option<TranslationUnit>> {
        // Apply strategy
        let ctx = crate::parser::strategy::ExtractionContext::new(content);
        if !self.strategy.should_extract(node_type, &ctx) {
            return Ok(None);
        }

        // Skip if only symbols
        if self.string_processor.is_only_symbols(content) {
            return Ok(None);
        }

        let id = format!("{}_{}_{}", file_path, start_pos.offset, end_pos.offset);
        let final_node_type = self.strategy.get_node_type(node_type);

        Ok(Some(TranslationUnit::new(
            id,
            final_node_type,
            content.to_string(),
            start_pos,
            end_pos,
        )))
    }

    /// Deduplicate units by position
    fn deduplicate_units(&self, units: Vec<TranslationUnit>) -> Vec<TranslationUnit> {
        let mut seen = HashSet::new();
        units
            .into_iter()
            .filter(|u| {
                let key = (u.start_pos.offset, u.end_pos.offset);
                seen.insert(key)
            })
            .collect()
    }
}
