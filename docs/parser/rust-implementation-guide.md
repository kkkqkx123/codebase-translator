# Rust 语言解析器实现指南

## 1. 概述

本文档详细描述了如何在 `src/parser` 模块中实现 Rust 语言解析器。Rust 语言解析器基于 tree-sitter，需要处理 Rust 特有的语法特性，如文档注释、宏调用、属性等。

## 2. Rust 语言特性分析

### 2.1 注释类型

Rust 支持以下注释类型：

| 类型 | 语法 | 用途 | 是否可翻译 |
|------|------|------|-----------|
| 普通行注释 | `// comment` | 代码注释 | ✓ |
| 普通块注释 | `/* comment */` | 代码注释 | ✓ |
| 外部文档注释 | `/// comment` | 生成文档 | ✓ |
| 内部文档注释 | `//! comment` | 生成文档 | ✓ |
| 块文档注释 | `/** comment */` | 生成文档 | ✓ |

### 2.2 宏调用

Rust 的宏调用需要特殊处理：

| 宏 | 分类 | 示例 | 翻译需求 |
|----|------|------|----------|
| `panic!()` | 错误 | `panic!("Error occurred")` | ✓ |
| `assert!()` | 错误 | `assert!(condition, "Failed")` | ✓ |
| `format!()` | 格式化 | `format!("Hello {}", name)` | ✓ |
| `println!()` | 日志 | `println!("Result: {}", result)` | ✓ |
| `eprintln!()` | 日志 | `eprintln!("Error: {}", err)` | ✓ |

### 2.3 字符串类型

| 类型 | 语法 | 特殊处理 |
|------|------|----------|
| 普通字符串 | `"hello"` | 转义字符 |
| 原始字符串 | `r#"hello"#` | 无转义，支持自定义分隔符 |
| 字节字符串 | `b"hello"` | 通常不翻译 |
| C 字符串 | `c"hello"` | 通常不翻译 |

### 2.4 属性

Rust 属性中可能包含可翻译内容：

```rust
#[doc = "Documentation string"]
#[deprecated(since = "1.0", note = "Use new_function instead")]
```

## 3. Tree-sitter 查询设计

### 3.1 基础查询

#### 3.1.1 注释查询

```lisp
;; 提取所有注释
(line_comment) @comment
(block_comment) @comment
```

#### 3.1.2 文档注释查询

```lisp
;; 外部文档注释 (///)
((line_comment) @doc_comment
  (#match? @doc_comment "^///"))

;; 内部文档注释 (//!)
((line_comment) @doc_comment
  (#match? @doc_comment "^//!"))

;; 块文档注释 (/**)
((block_comment) @doc_comment
  (#match? @doc_comment "^/\\*\\*"))
```

#### 3.1.3 字符串查询

```lisp
;; 普通字符串和原始字符串
(string_literal) @string
(raw_string_literal) @string

;; 排除字节字符串
((string_literal) @byte_string
  (#match? @byte_string "^b"))
```

### 3.2 高级查询

#### 3.2.1 宏调用查询

```lisp
;; 提取宏调用中的字符串参数
(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (string_literal) @macro_string))

;; 提取宏调用中的原始字符串
(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (raw_string_literal) @macro_string))
```

#### 3.2.2 文档属性查询

```lisp
;; 提取 #[doc = "..."] 属性
(attribute
  (attribute_item
    (identifier) @attr_name
    (#eq? @attr_name "doc")
    (string_literal) @doc_content))

;; 提取 #[deprecated(note = "...")] 属性
(attribute
  (attribute_item
    (identifier) @attr_name
    (#eq? @attr_name "deprecated")
    (attribute_arguments
      (attribute_argument
        (identifier) @arg_name
        (#eq? @arg_name "note")
        (string_literal) @note_content))))
```

#### 3.2.3 函数/结构体文档查询

```lisp
;; 提取函数前的文档注释
((line_comment) @fn_doc
  (#match? @fn_doc "^///")
  (#has-ancestor? @fn_doc function_item))

;; 提取结构体前的文档注释
((line_comment) @struct_doc
  (#match? @struct_doc "^///")
  (#has-ancestor? @struct_doc struct_item))
```

## 4. 实现细节

### 4.1 增强的 Rust 解析器

在 `src/parser/tree_sitter.rs` 中增强 Rust 解析器：

```rust
impl TreeSitterParserFactory {
    /// Create a parser for Rust files with enhanced features
    pub fn create_rust_parser(config: ParserConfig) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "rust".to_string(),
            extensions: vec!["rs".to_string()],
            language: tree_sitter_rust::LANGUAGE.into(),
            comment_query: RUST_COMMENT_QUERY.to_string(),
            docstring_query: Some(RUST_DOCSTRING_QUERY.to_string()),
            string_query: Some(RUST_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config)
    }
}

// Rust 特定查询
const RUST_COMMENT_QUERY: &str = r#"
(line_comment) @comment
(block_comment) @comment
"#;

const RUST_DOCSTRING_QUERY: &str = r#"
((line_comment) @docstring
  (#match? @docstring "^///"))

((line_comment) @docstring
  (#match? @docstring "^//!"))

((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#;

const RUST_STRING_QUERY: &str = r#"
(string_literal) @string
(raw_string_literal) @string
"#;

const RUST_MACRO_QUERY: &str = r#"
(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (string_literal) @macro_string))

(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (raw_string_literal) @macro_string))
"#;

const RUST_DOC_ATTR_QUERY: &str = r#"
(attribute
  (attribute_item
    (identifier) @attr_name
    (#eq? @attr_name "doc")
    (string_literal) @doc_content))

(attribute
  (attribute_item
    (identifier) @attr_name
    (#eq? @attr_name "deprecated")
    (attribute_arguments
      (attribute_argument
        (identifier) @arg_name
        (#eq? @arg_name "note")
        (string_literal) @note_content))))
"#;
```

### 4.2 Rust 特定解析器

创建 `src/parser/rust_parser.rs`：

```rust
//! Rust language parser with enhanced features
//!
//! This module provides specialized parsing for Rust source files,
//! handling Rust-specific features like macros, attributes, and doc comments.

use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor, Tree};

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, NodeType, Position, TranslationUnit};
use crate::parser::Parser as ParserTrait;

/// Rust macro categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RustMacroCategory {
    /// Error-related macros (panic!, assert!, etc.)
    Error,
    /// Formatting macros (format!, print!, etc.)
    Format,
    /// Logging macros (println!, eprintln!, etc.)
    Log,
    /// Debug macros (dbg!, etc.)
    Debug,
}

/// Rust macro patterns
pub struct RustMacroPatterns {
    error_macros: HashSet<String>,
    format_macros: HashSet<String>,
    log_macros: HashSet<String>,
    debug_macros: HashSet<String>,
}

impl RustMacroPatterns {
    /// Create default Rust macro patterns
    pub fn default() -> Self {
        Self {
            error_macros: [
                "panic!", "assert!", "assert_eq!", "assert_ne!",
                "unreachable!", "unimplemented!", "todo!",
            ].iter().map(|s| s.to_string()).collect(),
            format_macros: [
                "format!", "print!", "println!", "eprint!", "eprintln!",
                "write!", "writeln!",
            ].iter().map(|s| s.to_string()).collect(),
            log_macros: [
                "println!", "eprintln!",
            ].iter().map(|s| s.to_string()).collect(),
            debug_macros: [
                "dbg!",
            ].iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Classify a macro by name
    pub fn classify(&self, macro_name: &str) -> Option<RustMacroCategory> {
        if self.error_macros.contains(macro_name) {
            Some(RustMacroCategory::Error)
        } else if self.format_macros.contains(macro_name) {
            Some(RustMacroCategory::Format)
        } else if self.log_macros.contains(macro_name) {
            Some(RustMacroCategory::Log)
        } else if self.debug_macros.contains(macro_name) {
            Some(RustMacroCategory::Debug)
        } else {
            None
        }
    }

    /// Add custom macro pattern
    pub fn add_macro(&mut self, category: RustMacroCategory, macro_name: String) {
        match category {
            RustMacroCategory::Error => self.error_macros.insert(macro_name),
            RustMacroCategory::Format => self.format_macros.insert(macro_name),
            RustMacroCategory::Log => self.log_macros.insert(macro_name),
            RustMacroCategory::Debug => self.debug_macros.insert(macro_name),
        };
    }
}

/// Enhanced Rust parser
pub struct RustParser {
    base_parser: super::tree_sitter::TreeSitterParser,
    macro_patterns: RustMacroPatterns,
}

impl RustParser {
    /// Create a new Rust parser
    pub fn new(config: super::tree_sitter::ParserConfig) -> Result<Self> {
        let base_parser = super::tree_sitter::TreeSitterParserFactory::create_rust_parser(config)?;
        Ok(Self {
            base_parser,
            macro_patterns: RustMacroPatterns::default(),
        })
    }

    /// Extract macro call strings
    fn extract_macro_strings(
        &self,
        tree: &Tree,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let query = Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            RUST_MACRO_QUERY,
        ).map_err(|e| TranslateError::Parse(format!("Invalid macro query: {}", e)))?;

        let mut cursor = QueryCursor::new();
        let mut units = Vec::new();
        let mut match_idx = 0;

        let text_provider: &[u8] = content.as_bytes();
        let mut matches = cursor.matches(&query, tree.root_node(), text_provider);

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let capture_name = &query.capture_names()[capture.index as usize];

                // Process macro string captures
                if capture_name == "macro_string" {
                    let node = capture.node;
                    let node_text = node.utf8_text(content.as_bytes()).map_err(|e| {
                        TranslateError::Parse(format!("Failed to get node text: {}", e))
                    })?;

                    // Extract macro name
                    let macro_name = self.extract_macro_name(&node, content)?;

                    // Classify macro
                    let node_type = match self.macro_patterns.classify(&macro_name) {
                        Some(RustMacroCategory::Error) => NodeType::ErrorMessage,
                        Some(RustMacroCategory::Format) => NodeType::FormatString,
                        Some(RustMacroCategory::Log) => NodeType::LogMessage,
                        Some(RustMacroCategory::Debug) => NodeType::LogMessage,
                        None => NodeType::FormatString,
                    };

                    // Clean and validate text
                    let text = self.clean_string_literal(node_text);
                    if !self.should_include_string(&text) {
                        continue;
                    }

                    let id = format!("{}_macro_{}", file_path, match_idx);
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

                    units.push(TranslationUnit::new(id, node_type, text, start_pos, end_pos));
                    match_idx += 1;
                }
            }
        }

        Ok(units)
    }

    /// Extract doc attribute strings
    fn extract_doc_attributes(
        &self,
        tree: &Tree,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let query = Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            RUST_DOC_ATTR_QUERY,
        ).map_err(|e| TranslateError::Parse(format!("Invalid doc attribute query: {}", e)))?;

        let mut cursor = QueryCursor::new();
        let mut units = Vec::new();
        let mut match_idx = 0;

        let text_provider: &[u8] = content.as_bytes();
        let mut matches = cursor.matches(&query, tree.root_node(), text_provider);

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let capture_name = &query.capture_names()[capture.index as usize];

                if capture_name == "doc_content" || capture_name == "note_content" {
                    let node = capture.node;
                    let node_text = node.utf8_text(content.as_bytes()).map_err(|e| {
                        TranslateError::Parse(format!("Failed to get node text: {}", e))
                    })?;

                    let text = self.clean_string_literal(node_text);
                    if !self.should_include_string(&text) {
                        continue;
                    }

                    let id = format!("{}_doc_attr_{}", file_path, match_idx);
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

                    units.push(TranslationUnit::new(
                        id,
                        NodeType::DocString,
                        text,
                        start_pos,
                        end_pos,
                    ));
                    match_idx += 1;
                }
            }
        }

        Ok(units)
    }

    /// Extract macro name from a macro invocation
    fn extract_macro_name(&self, string_node: &Node, content: &str) -> Result<String> {
        // Find parent macro_invocation node
        let mut node = *string_node;
        while let Some(parent) = node.parent() {
            if parent.kind() == "macro_invocation" {
                // Find the macro name identifier
                for child in parent.children(&parent) {
                    if child.kind() == "identifier" {
                        return Ok(child.utf8_text(content.as_bytes())?.to_string());
                    }
                }
            }
            node = parent;
        }
        Ok(String::new())
    }

    /// Clean string literal (remove quotes and escape sequences)
    fn clean_string_literal(&self, text: &str) -> String {
        // Remove quotes
        let text = text.trim_matches('"');
        let text = text.trim_start_matches('r');
        let text = text.trim_matches('#');

        // Remove escape sequences (basic implementation)
        text.replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    }

    /// Check if string should be included
    fn should_include_string(&self, text: &str) -> bool {
        // Skip empty strings
        if text.trim().is_empty() {
            return false;
        }

        // Skip strings that are too short
        if text.len() < 2 {
            return false;
        }

        // Skip strings that look like code patterns
        if text.contains("::") || text.contains("->") {
            return false;
        }

        true
    }
}

#[async_trait]
impl ParserTrait for RustParser {
    async fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let content = file.content_string()
            .map_err(|e| TranslateError::Parse(format!("Invalid UTF-8 content: {}", e)))?;

        // Parse with base parser
        let mut units = self.base_parser.parse(file).await?;

        // Parse tree for enhanced features
        let tree = self.base_parser.parse_tree(&content)?;
        let file_path = file.path.to_string_lossy().to_string();

        // Extract macro strings
        let macro_units = self.extract_macro_strings(&tree, &content, &file_path)?;
        units.extend(macro_units);

        // Extract doc attributes
        let doc_attr_units = self.extract_doc_attributes(&tree, &content, &file_path)?;
        units.extend(doc_attr_units);

        // Sort by position
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }

    fn supports(&self, filename: &str) -> bool {
        filename.to_lowercase().ends_with(".rs")
    }

    fn supported_extensions(&self) -> &[&str] {
        &["rs"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    #[tokio::test]
    async fn test_rust_parser_basic() {
        let config = super::super::tree_sitter::ParserConfig::default();
        let parser = RustParser::new(config).expect("Failed to create Rust parser");

        let content = r#"
/// This is a doc comment
fn main() {
    // This is a regular comment
    let x = 5;
}
"#;

        let file = create_test_file(content, "test.rs");
        let units = parser.parse(&file).await.expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("test.rs"));
        assert!(!parser.supports("test.go"));
    }

    #[tokio::test]
    async fn test_rust_macro_extraction() {
        let config = super::super::tree_sitter::ParserConfig::default();
        let parser = RustParser::new(config).expect("Failed to create Rust parser");

        let content = r#"
fn main() {
    panic!("Something went wrong");
    println!("Result: {}", 42);
    format!("Hello {}", "world");
}
"#;

        let file = create_test_file(content, "test.rs");
        let units = parser.parse(&file).await.expect("Parsing should succeed");

        // Should extract macro strings
        let macro_units: Vec<_> = units.iter()
            .filter(|u| matches!(u.node_type, NodeType::ErrorMessage | NodeType::FormatString | NodeType::LogMessage))
            .collect();

        assert!(!macro_units.is_empty());
    }

    #[tokio::test]
    async fn test_rust_doc_attributes() {
        let config = super::super::tree_sitter::ParserConfig::default();
        let parser = RustParser::new(config).expect("Failed to create Rust parser");

        let content = r#"
#[doc = "This is a function"]
#[deprecated(since = "1.0", note = "Use new_function instead")]
fn old_function() {}
"#;

        let file = create_test_file(content, "test.rs");
        let units = parser.parse(&file).await.expect("Parsing should succeed");

        // Should extract doc attributes
        let doc_units: Vec<_> = units.iter()
            .filter(|u| u.node_type == NodeType::DocString)
            .collect();

        assert!(!doc_units.is_empty());
    }

    #[test]
    fn test_macro_patterns() {
        let patterns = RustMacroPatterns::default();

        assert_eq!(
            patterns.classify("panic!"),
            Some(RustMacroCategory::Error)
        );
        assert_eq!(
            patterns.classify("format!"),
            Some(RustMacroCategory::Format)
        );
        assert_eq!(
            patterns.classify("println!"),
            Some(RustMacroCategory::Log)
        );
        assert_eq!(
            patterns.classify("dbg!"),
            Some(RustMacroCategory::Debug)
        );
        assert_eq!(
            patterns.classify("unknown_macro!"),
            None
        );
    }
}
```

### 4.3 更新模块导出

更新 `src/parser/mod.rs`：

```rust
pub mod regex;
pub mod r#trait;
pub mod tree_sitter;
pub mod rust_parser;  // 新增

// Re-export commonly used types
pub use r#trait::Parser;
pub use tree_sitter::{
    LanguageConfig, ParserConfig, ParserCoordinator, TreeSitterParser, TreeSitterParserFactory,
};
pub use regex::{RegexParser, RegexParserConfig, RegexParserFactory};
pub use rust_parser::{RustParser, RustMacroPatterns, RustMacroCategory};  // 新增
```

## 5. 测试用例

### 5.1 测试文件结构

```
tests/parser/rust/
├── fixtures/
│   ├── basic.rs
│   ├── macros.rs
│   ├── doc_comments.rs
│   ├── attributes.rs
│   └── complex.rs
└── rust_parser_test.rs
```

### 5.2 基础测试

```rust
// tests/parser/rust/fixtures/basic.rs
/// This is a module doc comment
//! This is an inner doc comment

/// This function does something
fn example_function() {
    // Regular comment
    let x = 42;
}
```

### 5.3 宏测试

```rust
// tests/parser/rust/fixtures/macros.rs
fn main() {
    // Error macros
    panic!("Critical error occurred");
    assert!(condition, "Assertion failed: {}", reason);
    todo!("Not implemented yet");

    // Format macros
    let s = format!("Hello, {}", name);
    print!("Output: {}", value);
    println!("Result: {}", result);

    // Log macros
    eprintln!("Error: {}", error);
}
```

### 5.4 属性测试

```rust
// tests/parser/rust/fixtures/attributes.rs
#[doc = "This is a documented function"]
fn documented_function() {}

#[deprecated(since = "1.0", note = "Use new_function instead")]
fn old_function() {}

#[doc = include_str!("../docs/function.md")]
fn function_with_external_doc() {}
```

## 6. 性能优化

### 6.1 查询缓存

```rust
use std::sync::Mutex;
use std::collections::HashMap;

pub struct QueryCache {
    cache: Mutex<HashMap<String, Query>>,
}

impl QueryCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_or_create(
        &self,
        language: &Language,
        query_str: &str,
    ) -> Result<Query> {
        let mut cache = self.cache.lock().unwrap();

        if let Some(query) = cache.get(query_str) {
            return Ok(query.clone());
        }

        let query = Query::new(language, query_str)
            .map_err(|e| TranslateError::Parse(format!("Invalid query: {}", e)))?;

        cache.insert(query_str.to_string(), query.clone());
        Ok(query)
    }
}
```

### 6.2 增量解析

```rust
pub struct IncrementalRustParser {
    parser: Parser,
    old_tree: Option<Tree>,
    query_cache: Arc<QueryCache>,
}

impl IncrementalRustParser {
    pub fn parse_incremental(&mut self, content: &str) -> Result<Tree> {
        self.parser
            .parse(content, self.old_tree.as_ref())
            .ok_or_else(|| TranslateError::Parse("Failed to parse".to_string()))
    }
}
```

## 7. 集成到 ParserCoordinator

更新 `src/parser/tree_sitter.rs` 中的 `ParserCoordinator`：

```rust
impl ParserCoordinator {
    pub fn new(config: ParserConfig) -> Result<Self> {
        let mut parsers: Vec<Arc<dyn ParserTrait>> = Vec::new();

        // Add tree-sitter parsers
        for parser_result in TreeSitterParserFactory::create_all_parsers(config.clone()) {
            match parser_result {
                Ok(parser) => parsers.push(Arc::new(parser)),
                Err(e) => {
                    tracing::warn!("Failed to create parser: {}", e);
                }
            }
        }

        // Add enhanced Rust parser
        match rust_parser::RustParser::new(config.clone()) {
            Ok(parser) => parsers.push(Arc::new(parser)),
            Err(e) => {
                tracing::warn!("Failed to create Rust parser: {}", e);
            }
        }

        // Add regex fallback parser
        parsers.push(Arc::new(super::regex::RegexParser::new(config)));

        Ok(Self { parsers })
    }
}
```

## 8. 使用示例

```rust
use codebase_translate::parser::{ParserCoordinator, ParserConfig, RustParser};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use coordinator (recommended)
    let config = ParserConfig::default();
    let coordinator = ParserCoordinator::new(config)?;

    let file = File::new(
        PathBuf::from("src/main.rs"),
        std::fs::read("src/main.rs")?,
        "utf-8",
    );

    let units = coordinator.parse_file(&file).await?;

    // Or use Rust parser directly
    let rust_parser = RustParser::new(ParserConfig::default())?;
    let units = rust_parser.parse(&file).await?;

    for unit in units {
        println!("{}: {}", unit.node_type, unit.content);
    }

    Ok(())
}
```

## 9. 已知限制

1. **宏展开**: 当前不展开宏，只提取宏调用中的字符串
2. **过程宏**: 不支持自定义过程宏的解析
3. **属性宏**: 不支持属性宏的解析
4. **derive 宏**: 不解析 derive 宏生成的内容

## 10. 未来改进

1. **宏展开**: 支持展开常见宏（如 `vec!`, `format!`）
2. **过程宏**: 支持解析过程宏的输出
3. **上下文感知**: 基于函数/模块上下文过滤翻译单元
4. **智能分类**: 使用机器学习改进文本分类

## 11. 参考资料

- [Rust 语言规范](https://doc.rust-lang.org/reference/)
- [Tree-sitter Rust 语法](https://github.com/tree-sitter/tree-sitter-rust)
- [Rust 宏文档](https://doc.rust-lang.org/reference/macros.html)
- [Rust 属性文档](https://doc.rust-lang.org/reference/attributes.html)
