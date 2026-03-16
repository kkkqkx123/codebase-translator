# 策略模式和过滤器实现指南

## 1. 概述

本文档描述如何在 `src/parser` 模块中实现策略模式和过滤器系统，这些组件负责决定哪些内容应该被提取和翻译。

## 2. 策略模式设计

### 2.1 目标

策略模式允许根据配置灵活地控制提取行为，支持：

- 基于配置的提取规则
- 可组合的策略
- 运行时策略切换
- 自定义策略扩展

### 2.2 核心概念

#### 2.2.1 策略节点类型

```rust
/// Strategy node type for extraction decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrategyNodeType {
    /// Regular comment
    Comment,
    /// Documentation string
    DocString,
    /// Error message
    ErrorMessage,
    /// Format string
    FormatString,
    /// Log message
    LogMessage,
    /// Markdown paragraph
    MarkdownParagraph,
    /// Markdown heading
    MarkdownHeading,
    /// Markdown list item
    MarkdownListItem,
    /// Markdown table cell
    MarkdownTableCell,
}

impl StrategyNodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::DocString => "docstring",
            Self::ErrorMessage => "error_message",
            Self::FormatString => "format_string",
            Self::LogMessage => "log_message",
            Self::MarkdownParagraph => "markdown_paragraph",
            Self::MarkdownHeading => "markdown_heading",
            Self::MarkdownListItem => "markdown_list_item",
            Self::MarkdownTableCell => "markdown_table_cell",
        }
    }
}
```

#### 2.2.2 提取上下文

```rust
/// Context for extraction decisions
#[derive(Debug, Clone)]
pub struct ExtractionContext {
    /// Content to extract
    pub content: String,
    /// Function name (if applicable)
    pub function_name: Option<String>,
    /// Whether the item is exported/public
    pub is_exported: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ExtractionContext {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            function_name: None,
            is_exported: false,
            metadata: HashMap::new(),
        }
    }

    pub fn with_function_name(mut self, name: impl Into<String>) -> Self {
        self.function_name = Some(name.into());
        self
    }

    pub fn with_exported(mut self, exported: bool) -> Self {
        self.is_exported = exported;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}
```

### 2.3 策略 Trait

```rust
use crate::core::models::NodeType;

/// Extraction strategy trait
pub trait ExtractionStrategy: Send + Sync {
    /// Determine if a node should be extracted
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool;

    /// Get the corresponding NodeType for a StrategyNodeType
    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType;

    /// Get strategy name
    fn name(&self) -> &str;
}
```

### 2.4 配置驱动的策略

```rust
use serde::{Deserialize, Serialize};

/// Extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Extract comments
    #[serde(default = "default_true")]
    pub comments: bool,

    /// Extract docstrings
    #[serde(default = "default_true")]
    pub docstrings: bool,

    /// Extract error messages
    #[serde(default = "default_true")]
    pub error_messages: bool,

    /// Extract format strings
    #[serde(default = "default_false")]
    pub format_strings: bool,

    /// Extract log messages
    #[serde(default = "default_true")]
    pub log_messages: bool,

    /// Custom extraction patterns
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            comments: true,
            docstrings: true,
            error_messages: true,
            format_strings: false,
            log_messages: true,
            custom_patterns: Vec::new(),
        }
    }
}

/// Config-based extraction strategy
pub struct ConfigBasedStrategy {
    config: ExtractionConfig,
}

impl ConfigBasedStrategy {
    pub fn new(config: ExtractionConfig) -> Self {
        Self { config }
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ExtractionConfig = toml::from_str(&content)?;
        Ok(Self::new(config))
    }
}

impl ExtractionStrategy for ConfigBasedStrategy {
    fn should_extract(&self, node_type: StrategyNodeType, _ctx: &ExtractionContext) -> bool {
        match node_type {
            StrategyNodeType::Comment => self.config.comments,
            StrategyNodeType::DocString => self.config.docstrings,
            StrategyNodeType::ErrorMessage => self.config.error_messages,
            StrategyNodeType::FormatString => self.config.format_strings,
            StrategyNodeType::LogMessage => self.config.log_messages,
            StrategyNodeType::MarkdownParagraph
            | StrategyNodeType::MarkdownHeading
            | StrategyNodeType::MarkdownListItem
            | StrategyNodeType::MarkdownTableCell => true,
        }
    }

    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType {
        match node_type {
            StrategyNodeType::Comment => NodeType::Comment,
            StrategyNodeType::DocString => NodeType::DocString,
            StrategyNodeType::ErrorMessage => NodeType::ErrorMessage,
            StrategyNodeType::FormatString => NodeType::FormatString,
            StrategyNodeType::LogMessage => NodeType::LogMessage,
            StrategyNodeType::MarkdownParagraph => NodeType::Comment,
            StrategyNodeType::MarkdownHeading => NodeType::Comment,
            StrategyNodeType::MarkdownListItem => NodeType::Comment,
            StrategyNodeType::MarkdownTableCell => NodeType::Comment,
        }
    }

    fn name(&self) -> &str {
        "config_based"
    }
}
```

### 2.5 组合策略

```rust
/// Combined strategy that applies multiple strategies
pub struct CombinedStrategy {
    strategies: Vec<Box<dyn ExtractionStrategy>>,
    combine_mode: CombineMode,
}

/// How to combine multiple strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombineMode {
    /// All strategies must agree (AND)
    All,
    /// At least one strategy must agree (OR)
    Any,
    /// First strategy that decides wins
    First,
}

impl CombinedStrategy {
    pub fn new(strategies: Vec<Box<dyn ExtractionStrategy>>, mode: CombineMode) -> Self {
        Self {
            strategies,
            combine_mode: mode,
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn ExtractionStrategy>) {
        self.strategies.push(strategy);
    }
}

impl ExtractionStrategy for CombinedStrategy {
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool {
        if self.strategies.is_empty() {
            return true;
        }

        match self.combine_mode {
            CombineMode::All => self.strategies.iter()
                .all(|s| s.should_extract(node_type, ctx)),
            CombineMode::Any => self.strategies.iter()
                .any(|s| s.should_extract(node_type, ctx)),
            CombineMode::First => self.strategies.first()
                .map(|s| s.should_extract(node_type, ctx))
                .unwrap_or(true),
        }
    }

    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType {
        self.strategies.first()
            .map(|s| s.get_node_type(node_type))
            .unwrap_or(match node_type {
                StrategyNodeType::Comment => NodeType::Comment,
                _ => NodeType::DocString,
            })
    }

    fn name(&self) -> &str {
        "combined"
    }
}
```

### 2.6 自定义策略示例

```rust
/// Strategy that only extracts exported items
pub struct ExportedOnlyStrategy {
    base: Box<dyn ExtractionStrategy>,
}

impl ExportedOnlyStrategy {
    pub fn new(base: Box<dyn ExtractionStrategy>) -> Self {
        Self { base }
    }
}

impl ExtractionStrategy for ExportedOnlyStrategy {
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool {
        if !ctx.is_exported {
            return false;
        }
        self.base.should_extract(node_type, ctx)
    }

    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType {
        self.base.get_node_type(node_type)
    }

    fn name(&self) -> &str {
        "exported_only"
    }
}

/// Strategy that filters by function name patterns
pub struct FunctionNameFilterStrategy {
    base: Box<dyn ExtractionStrategy>,
    allowed_patterns: Vec<regex::Regex>,
    denied_patterns: Vec<regex::Regex>,
}

impl FunctionNameFilterStrategy {
    pub fn new(
        base: Box<dyn ExtractionStrategy>,
        allowed: Vec<String>,
        denied: Vec<String>,
    ) -> Result<Self, regex::Error> {
        let allowed_patterns = allowed.into_iter()
            .map(|p| regex::Regex::new(&p))
            .collect::<Result<Vec<_>, _>>()?;

        let denied_patterns = denied.into_iter()
            .map(|p| regex::Regex::new(&p))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            base,
            allowed_patterns,
            denied_patterns,
        })
    }
}

impl ExtractionStrategy for FunctionNameFilterStrategy {
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool {
        if let Some(func_name) = &ctx.function_name {
            // Check denied patterns first
            for pattern in &self.denied_patterns {
                if pattern.is_match(func_name) {
                    return false;
                }
            }

            // Check allowed patterns
            if !self.allowed_patterns.is_empty() {
                let allowed = self.allowed_patterns.iter()
                    .any(|p| p.is_match(func_name));
                if !allowed {
                    return false;
                }
            }
        }

        self.base.should_extract(node_type, ctx)
    }

    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType {
        self.base.get_node_type(node_type)
    }

    fn name(&self) -> &str {
        "function_name_filter"
    }
}
```

## 3. 过滤器系统

### 3.1 过滤器配置

```rust
use serde::{Deserialize, Serialize};

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Keywords to exclude
    #[serde(default)]
    pub exclude_keywords: Vec<String>,

    /// Regex patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Regex patterns to include (if set, only matching content is included)
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// Minimum content length
    #[serde(default = "default_min_length")]
    pub min_length: usize,

    /// Maximum content length
    #[serde(default = "default_max_length")]
    pub max_length: usize,

    /// Allow placeholders (e.g., %s, {})
    #[serde(default = "default_false")]
    pub allow_placeholders: bool,

    /// Detect and filter code patterns
    #[serde(default = "default_true")]
    pub detect_code_patterns: bool,
}

fn default_min_length() -> usize {
    0
}

fn default_max_length() -> usize {
    10000
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            exclude_keywords: vec![
                "TODO".to_string(),
                "FIXME".to_string(),
                "NOTE".to_string(),
                "XXX".to_string(),
                "HACK".to_string(),
                "Copyright".to_string(),
                "License".to_string(),
                "Author".to_string(),
                "Licensed".to_string(),
            ],
            exclude_patterns: vec![
                r"https?://[^\s]+".to_string(),
                r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
                r"\[[^\]]+\]\([^)]+\)".to_string(),
                r"!\[[^\]]*\]\([^)]+\)".to_string(),
                r"<[^>]+>".to_string(),
                r"`[^`]+`".to_string(),
            ],
            include_patterns: Vec::new(),
            min_length: 0,
            max_length: 10000,
            allow_placeholders: false,
            detect_code_patterns: true,
        }
    }
}
```

### 3.2 内容过滤器

```rust
use std::sync::Arc;

/// Content filter
pub struct ContentFilter {
    config: FilterConfig,
    exclude_keywords_regex: Vec<regex::Regex>,
    exclude_patterns_regex: Vec<regex::Regex>,
    include_patterns_regex: Vec<regex::Regex>,
    placeholder_regex: Vec<regex::Regex>,
    code_pattern_regex: Vec<regex::Regex>,
}

impl ContentFilter {
    pub fn new(config: FilterConfig) -> Result<Self, regex::Error> {
        // Compile exclude keywords as word-boundary regexes
        let exclude_keywords_regex = config.exclude_keywords.iter()
            .map(|kw| regex::Regex::new(&format!(r"\b{}\b", regex::escape(kw))))
            .collect::<Result<Vec<_>, _>>()?;

        // Compile exclude patterns
        let exclude_patterns_regex = config.exclude_patterns.iter()
            .map(|p| regex::Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;

        // Compile include patterns
        let include_patterns_regex = config.include_patterns.iter()
            .map(|p| regex::Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;

        // Placeholder patterns
        let placeholder_regex = vec![
            regex::Regex::new(r"%[sdvf]")?,
            regex::Regex::new(r"\$\d{1,2}\b")?,
            regex::Regex::new(r"\$\{[^}]*\}")?,
            regex::Regex::new(r"\{[^}]*\}")?,
        ];

        // Code pattern detection
        let code_pattern_regex = vec![
            regex::Regex::new(r"\w+\.\w+")?,        // Member access
            regex::Regex::new(r"\w+\([^)]*\)")?,     // Function call
            regex::Regex::new(r"\{[^}]*\}")?,       // Braces
            regex::Regex::new(r"\[[^\]]*\]")?,       // Brackets
        ];

        Ok(Self {
            config,
            exclude_keywords_regex,
            exclude_patterns_regex,
            include_patterns_regex,
            placeholder_regex,
            code_pattern_regex,
        })
    }

    /// Check if content should be translated
    pub fn should_translate(&self, text: &str) -> bool {
        // Empty check
        if text.is_empty() {
            return false;
        }

        // Length check
        let len = text.len();
        if len < self.config.min_length {
            return false;
        }
        if self.config.max_length > 0 && len > self.config.max_length {
            return false;
        }

        // Exclude keywords check
        for pattern in &self.exclude_keywords_regex {
            if pattern.is_match(text) {
                return false;
            }
        }

        // Exclude patterns check
        for pattern in &self.exclude_patterns_regex {
            if pattern.is_match(text) {
                return false;
            }
        }

        // Include patterns check
        if !self.include_patterns_regex.is_empty() {
            let included = self.include_patterns_regex.iter()
                .any(|p| p.is_match(text));
            if !included {
                return false;
            }
        }

        // Placeholder check
        if !self.config.allow_placeholders {
            for pattern in &self.placeholder_regex {
                if pattern.is_match(text) {
                    return false;
                }
            }
        }

        // Code pattern check
        if self.config.detect_code_patterns {
            for pattern in &self.code_pattern_regex {
                if pattern.is_match(text) {
                    return false;
                }
            }
        }

        // Symbol-only check
        if is_only_symbols(text) {
            return false;
        }

        true
    }

    /// Get filter configuration
    pub fn config(&self) -> &FilterConfig {
        &self.config
    }
}

/// Check if text contains only symbols/whitespace
fn is_only_symbols(text: &str) -> bool {
    text.chars().all(|c| c.is_whitespace() || is_punctuation(c))
}

/// Check if character is punctuation
fn is_punctuation(c: char) -> bool {
    matches!(
        c,
        '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ','
            | '-' | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | '['
            | '\\' | ']' | '^' | '_' | '`' | '{' | '|' | '}' | '~'
    )
}
```

### 3.3 增强的过滤器（结合语言检测）

```rust
use crate::parser::language::LanguageInfo;

/// Enhanced content filter with language detection
pub struct EnhancedContentFilter {
    base: ContentFilter,
    language_detector: Arc<dyn LanguageDetector>,
}

impl EnhancedContentFilter {
    pub fn new(
        config: FilterConfig,
        language_detector: Arc<dyn LanguageDetector>,
    ) -> Result<Self, regex::Error> {
        Ok(Self {
            base: ContentFilter::new(config)?,
            language_detector,
        })
    }

    /// Check if content should be translated with language awareness
    pub fn should_translate_with_lang(&self, text: &str, lang_info: &LanguageInfo) -> bool {
        // Basic filtering
        if !self.base.should_translate(text) {
            return false;
        }

        // Language-aware code pattern filtering
        if self.base.config().detect_code_patterns {
            if self.base.code_pattern_regex.iter().any(|p| p.is_match(text)) {
                // If text contains non-Latin script, it's likely a code example in comments
                if lang_info.script != Script::Latin && lang_info.script != Script::Unknown {
                    return true;
                }
                // Pure Latin with code patterns is likely code
                return false;
            }
        }

        true
    }
}

/// Language detector trait
pub trait LanguageDetector: Send + Sync {
    fn detect(&self, text: &str) -> LanguageInfo;
}
```

## 4. 集成到解析器

### 4.1 更新 TreeSitterParser

```rust
use crate::parser::strategy::{ExtractionStrategy, ExtractionContext, StrategyNodeType};
use crate::parser::filter::ContentFilter;

pub struct TreeSitterParser {
    config: ParserConfig,
    language_config: LanguageConfig,
    strategy: Arc<dyn ExtractionStrategy>,
    filter: Arc<ContentFilter>,
}

impl TreeSitterParser {
    pub fn new(
        language_config: LanguageConfig,
        config: ParserConfig,
        strategy: Arc<dyn ExtractionStrategy>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&language_config.language)
            .map_err(|e| TranslateError::Parse(format!("Failed to set language: {}", e)))?;

        Ok(Self {
            config,
            language_config,
            strategy,
            filter,
        })
    }

    fn extract_with_query(
        &self,
        root_node: &Node,
        content: &str,
        query_str: &str,
        strategy_node_type: StrategyNodeType,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let query = Query::new(&self.language_config.language, query_str)
            .map_err(|e| TranslateError::Parse(format!("Invalid query: {}", e)))?;

        let mut cursor = QueryCursor::new();
        let mut match_idx = 0usize;

        let mut units = Vec::new();
        let capture_names = query.capture_names();

        let text_provider: &[u8] = content.as_bytes();
        let mut matches = cursor.matches(&query, *root_node, text_provider);

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let capture_name = &capture_names[capture.index as usize];
                let node = capture.node;

                if !capture_name.contains("content") && !capture_name.contains("text") {
                    continue;
                }

                let node_text = node.utf8_text(content.as_bytes()).map_err(|e| {
                    TranslateError::Parse(format!("Failed to get node text: {}", e))
                })?;

                let text = if self.config.trim_content {
                    node_text.trim()
                } else {
                    node_text
                };

                // Apply filter
                if !self.filter.should_translate(text) {
                    continue;
                }

                // Apply strategy
                let ctx = ExtractionContext::new(text);
                if !self.strategy.should_extract(strategy_node_type, &ctx) {
                    continue;
                }

                let id = format!("{}_{}_{}", file_path, strategy_node_type.as_str(), match_idx);
                let start_pos = Position::new(
                    node.start_position().row + 1,
                    node.start_position().column + 1,
                    node.start_byte(),
                );
                let end_pos = Position::new(
                    node.end_position().row + 1,
                    node.end_position().column + 1,
                    node.end_byte(),
                );

                let node_type = self.strategy.get_node_type(strategy_node_type);
                let unit = TranslationUnit::new(id, node_type, text.to_string(), start_pos, end_pos);
                units.push(unit);
                match_idx += 1;
            }
        }

        Ok(units)
    }
}
```

### 4.2 更新 ParserCoordinator

```rust
impl ParserCoordinator {
    pub fn new_with_strategy_and_filter(
        config: ParserConfig,
        strategy: Arc<dyn ExtractionStrategy>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        let mut parsers: Vec<Arc<dyn ParserTrait>> = Vec::new();

        // Add tree-sitter parsers with custom strategy and filter
        for parser_result in TreeSitterParserFactory::create_all_parsers_with_strategy_and_filter(
            config.clone(),
            strategy.clone(),
            filter.clone(),
        ) {
            match parser_result {
                Ok(parser) => parsers.push(Arc::new(parser)),
                Err(e) => {
                    tracing::warn!("Failed to create parser: {}", e);
                }
            }
        }

        // Add regex fallback parser
        parsers.push(Arc::new(super::regex::RegexParser::new(config)));

        Ok(Self { parsers })
    }
}
```

## 5. 测试

### 5.1 策略测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_based_strategy() {
        let config = ExtractionConfig {
            comments: true,
            docstrings: false,
            error_messages: true,
            ..Default::default()
        };

        let strategy = ConfigBasedStrategy::new(config);

        let ctx = ExtractionContext::new("test");

        assert!(strategy.should_extract(StrategyNodeType::Comment, &ctx));
        assert!(!strategy.should_extract(StrategyNodeType::DocString, &ctx));
        assert!(strategy.should_extract(StrategyNodeType::ErrorMessage, &ctx));
    }

    #[test]
    fn test_combined_strategy_all() {
        let strategy1 = Box::new(ConfigBasedStrategy::new(ExtractionConfig {
            comments: true,
            ..Default::default()
        })) as Box<dyn ExtractionStrategy>;

        let strategy2 = Box::new(ConfigBasedStrategy::new(ExtractionConfig {
            docstrings: true,
            ..Default::default()
        })) as Box<dyn ExtractionStrategy>;

        let combined = CombinedStrategy::new(
            vec![strategy1, strategy2],
            CombineMode::All,
        );

        let ctx = ExtractionContext::new("test");

        // Both must agree
        assert!(!combined.should_extract(StrategyNodeType::Comment, &ctx));
        assert!(!combined.should_extract(StrategyNodeType::DocString, &ctx));
    }

    #[test]
    fn test_exported_only_strategy() {
        let base = Box::new(ConfigBasedStrategy::new(ExtractionConfig {
            comments: true,
            ..Default::default()
        })) as Box<dyn ExtractionStrategy>;

        let strategy = ExportedOnlyStrategy::new(base);

        let exported_ctx = ExtractionContext::new("test").with_exported(true);
        let private_ctx = ExtractionContext::new("test").with_exported(false);

        assert!(strategy.should_extract(StrategyNodeType::Comment, &exported_ctx));
        assert!(!strategy.should_extract(StrategyNodeType::Comment, &private_ctx));
    }
}
```

### 5.2 过滤器测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_filter_basic() {
        let config = FilterConfig::default();
        let filter = ContentFilter::new(config).unwrap();

        assert!(filter.should_translate("Hello world"));
        assert!(!filter.should_translate("TODO: fix this"));
        assert!(!filter.should_translate("x"));
    }

    #[test]
    fn test_content_filter_placeholders() {
        let config = FilterConfig {
            allow_placeholders: false,
            ..Default::default()
        };
        let filter = ContentFilter::new(config).unwrap();

        assert!(!filter.should_translate("Hello %s"));
        assert!(!filter.should_translate("Value: {value}"));
        assert!(filter.should_translate("Hello world"));
    }

    #[test]
    fn test_content_filter_code_patterns() {
        let config = FilterConfig {
            detect_code_patterns: true,
            ..Default::default()
        };
        let filter = ContentFilter::new(config).unwrap();

        assert!(!filter.should_translate("obj.property"));
        assert!(!filter.should_translate("function()"));
        assert!(filter.should_translate("This is a comment"));
    }

    #[test]
    fn test_content_filter_custom_patterns() {
        let config = FilterConfig {
            include_patterns: vec![r"^[A-Z]".to_string()],
            ..Default::default()
        };
        let filter = ContentFilter::new(config).unwrap();

        assert!(filter.should_translate("Hello world"));
        assert!(!filter.should_translate("hello world"));
    }
}
```

## 6. 使用示例

```rust
use codebase_translate::parser::{
    strategy::{ExtractionStrategy, ConfigBasedStrategy, ExtractionContext, StrategyNodeType},
    filter::{ContentFilter, FilterConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create strategy
    let config = ExtractionConfig {
        comments: true,
        docstrings: true,
        error_messages: true,
        format_strings: false,
        log_messages: true,
        custom_patterns: Vec::new(),
    };
    let strategy = Arc::new(ConfigBasedStrategy::new(config));

    // Create filter
    let filter_config = FilterConfig {
        min_length: 2,
        max_length: 1000,
        allow_placeholders: false,
        detect_code_patterns: true,
        ..Default::default()
    };
    let filter = Arc::new(ContentFilter::new(filter_config)?);

    // Use strategy
    let ctx = ExtractionContext::new("This is a comment");
    let should_extract = strategy.should_extract(StrategyNodeType::Comment, &ctx);

    // Use filter
    let should_translate = filter.should_translate("This is a comment");

    println!("Should extract: {}", should_extract);
    println!("Should translate: {}", should_translate);

    Ok(())
}
```

## 7. 最佳实践

1. **策略组合**: 使用 `CombinedStrategy` 组合多个策略以实现复杂逻辑
2. **性能优化**: 缓存编译后的正则表达式
3. **配置管理**: 使用 TOML 文件管理策略和过滤器配置
4. **测试覆盖**: 为自定义策略编写充分的单元测试
5. **文档化**: 为自定义策略提供清晰的文档和使用示例

## 8. 参考资料

- [Go 版本策略实现](../../internal/parser/strategy.go)
- [Go 版本过滤器实现](../../internal/parser/common/filter.go)
- [正则表达式文档](https://docs.rs/regex/)
- [Serde 文档](https://docs.rs/serde/)
