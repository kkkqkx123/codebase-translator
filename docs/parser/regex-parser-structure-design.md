# Regex Parser Directory Structure Design

## Overview

This document describes the proposed directory structure for regex-based parsers, following the same design principles as `src/parser/languages` for tree-sitter based parsers.

## Current Issues

1. **Monolithic Design**: Current `src/parser/regex/` mixes infrastructure and type-specific logic
2. **Unclear Separation**: `presets.rs` and `factory.rs` blur the lines between configuration and creation
3. **No Type Isolation**: Different file types (shell, html, sql) are not properly separated
4. **Maintenance Burden**: Adding new file types requires modifying multiple files

## Proposed Structure

```
src/parser/
├── languages/              # Tree-sitter based parsers (existing)
│   ├── c/
│   ├── cpp/
│   ├── rust/
│   └── ...
├── regex_parsers/          # Regex-based parsers (new, simplified)
│   ├── mod.rs              # Module organization and exports
│   ├── fallback.rs         # Generic fallback parser
│   ├── shell.rs            # Shell script parser
│   ├── html.rs             # HTML/XML parser
│   └── sql.rs             # SQL parser
├── regex/                  # Core regex infrastructure (refactored)
│   ├── mod.rs              # Module exports
│   ├── parser.rs           # Base RegexParser trait and implementation
│   ├── config.rs           # Base configuration struct
│   ├── utils.rs            # Utility functions
│   └── state_machine.rs    # State machine for complex patterns
├── core/                   # Core parsing infrastructure (existing)
│   ├── mod.rs
│   ├── parser.rs           # Base Parser trait
│   ├── query_executor.rs
│   └── string_processor.rs
├── coordinator/            # Parser coordination (existing)
│   ├── mod.rs
│   ├── coordinator.rs
│   └── tests.rs
└── factory/                # Parser factories (existing)
    ├── mod.rs
    ├── tree_sitter.rs
    └── regex.rs
```

## Module Responsibilities

### `regex_parsers/` - Type-Specific Parsers

Each file represents a specific file type with its own implementation. No separate config files needed - all logic is contained in a single file.

**Example: `regex_parsers/shell.rs`**
```rust
//! Shell script parser

use crate::parser::regex::RegexParser;
use crate::parser::Parser as ParserTrait;

pub struct ShellParser {
    inner: RegexParser,
}

impl ShellParser {
    pub fn new(config: crate::parser::tree_sitter::ParserConfig) -> Self {
        let regex_config = crate::parser::regex::RegexParserConfig {
            extensions: vec!["sh".to_string(), "bash".to_string(), "zsh".to_string(), "fish".to_string()],
            line_comment_pattern: Some(r"(?m)^\s*#\s*(.+)$".to_string()),
            block_comment_pattern: None,
            doc_comment_pattern: None,
            string_pattern: Some(r#"["']([^"']{3,})["']"#.to_string()),
            min_content_length: config.min_content_length,
            max_content_length: config.max_content_length,
            trim_content: config.trim_content,
            state_machine_patterns: Vec::new(),
        };
        Self {
            inner: RegexParser::with_config(config, regex_config),
        }
    }
}

impl ParserTrait for ShellParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        self.inner.parse(file)
    }

    fn supports(&self, filename: &str) -> bool {
        self.inner.supports(filename)
    }
}
```

**Example: `regex_parsers/html.rs`**
```rust
//! HTML/XML parser

use crate::parser::regex::RegexParser;
use crate::parser::Parser as ParserTrait;

pub struct HtmlParser {
    inner: RegexParser,
}

impl HtmlParser {
    pub fn new(config: crate::parser::tree_sitter::ParserConfig) -> Self {
        let regex_config = crate::parser::regex::RegexParserConfig {
            extensions: vec!["html".to_string(), "htm".to_string(), "xml".to_string(), "svg".to_string()],
            line_comment_pattern: None,
            block_comment_pattern: Some(r"<!--\s*([\s\S]*?)\s*-->".to_string()),
            doc_comment_pattern: None,
            string_pattern: None,
            min_content_length: config.min_content_length,
            max_content_length: config.max_content_length,
            trim_content: config.trim_content,
            state_machine_patterns: Vec::new(),
        };
        Self {
            inner: RegexParser::with_config(config, regex_config),
        }
    }
}

impl ParserTrait for HtmlParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        self.inner.parse(file)
    }

    fn supports(&self, filename: &str) -> bool {
        self.inner.supports(filename)
    }
}
```

**Example: `regex_parsers/mod.rs`**
```rust
//! Regex-based parsers for specific file types

pub mod fallback;
pub mod shell;
pub mod html;
pub mod sql;

pub use fallback::FallbackParser;
pub use shell::ShellParser;
pub use html::HtmlParser;
pub use sql::SqlParser;
```

### `regex/` - Core Infrastructure

Simplified to provide only the base infrastructure:

- **parser.rs**: Base `RegexParser` struct that implements `Parser` trait
- **config.rs**: Base `RegexParserConfig` struct
- **utils.rs**: Utility functions for regex operations
- **state_machine.rs**: State machine for complex pattern matching

**Factory Integration:**
The factory should be located in `src/parser/factory/regex.rs` and use the `regex_parsers` module:

```rust
//! Regex parser factory

use crate::parser::regex_parsers::{FallbackParser, ShellParser, HtmlParser, SqlParser};

pub struct RegexParserFactory;

impl RegexParserFactory {
    pub fn create_for_extension(
        config: crate::parser::tree_sitter::ParserConfig,
        ext: &str,
    ) -> Option<Box<dyn Parser>> {
        match ext.to_lowercase().as_str() {
            "txt" | "md" | "markdown" | "yml" | "yaml" | "toml" => {
                Some(Box::new(FallbackParser::new(config)))
            }
            "sh" | "bash" | "zsh" | "fish" => Some(Box::new(ShellParser::new(config))),
            "html" | "htm" | "xml" | "svg" => Some(Box::new(HtmlParser::new(config))),
            "sql" | "mysql" | "pgsql" => Some(Box::new(SqlParser::new(config))),
            _ => None,
        }
    }
}
```

## Benefits

1. **Clear Separation**: Each file type has its own isolated module
2. **Easy Extension**: Adding new types only requires creating a new directory
3. **Consistent Design**: Follows the same pattern as `languages/`
4. **Better Testing**: Each type can be tested independently
5. **Reduced Coupling**: Changes to one type don't affect others
6. **Clear Ownership**: Each module has a single responsibility

## Migration Path

1. Create `src/parser/regex_parsers/` directory
2. Implement type-specific parser files (fallback.rs, shell.rs, html.rs, sql.rs)
3. Create `src/parser/regex_parsers/mod.rs` for module organization
4. Refactor `src/parser/factory/regex.rs` to use new modules
5. Update all imports and references
6. Remove `src/parser/regex/presets.rs`
7. Remove `src/parser/regex/factory.rs`
8. Update tests
9. Run full test suite

## Future Extensions

This structure makes it easy to add new regex-based parsers:

```bash
# Add a new parser type (e.g., for JSON files)
# Create a single file: src/parser/regex_parsers/json.rs
# Add to regex_parsers/mod.rs: pub mod json;
# Update factory/regex.rs routing
```

**Example: Adding a JSON parser**
```rust
// src/parser/regex_parsers/json.rs
//! JSON parser

use crate::parser::regex::RegexParser;
use crate::parser::Parser as ParserTrait;

pub struct JsonParser {
    inner: RegexParser,
}

impl JsonParser {
    pub fn new(config: crate::parser::tree_sitter::ParserConfig) -> Self {
        let regex_config = crate::parser::regex::RegexParserConfig {
            extensions: vec!["json".to_string()],
            line_comment_pattern: None,
            block_comment_pattern: None,
            doc_comment_pattern: None,
            string_pattern: Some(r#""([^"]{3,})""#.to_string()),
            min_content_length: config.min_content_length,
            max_content_length: config.max_content_length,
            trim_content: config.trim_content,
            state_machine_patterns: Vec::new(),
        };
        Self {
            inner: RegexParser::with_config(config, regex_config),
        }
    }
}

impl ParserTrait for JsonParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        self.inner.parse(file)
    }

    fn supports(&self, filename: &str) -> bool {
        self.inner.supports(filename)
    }
}
```

## Comparison with Current Design

| Aspect | Current | Proposed |
|--------|---------|----------|
| Type Isolation | ❌ Mixed in presets.rs | ✅ Separate single-file modules |
| Extensibility | ❌ Modify multiple files | ✅ Add new single file |
| Simplicity | ❌ Over-engineered with configs | ✅ Simple, self-contained files |
| Testing | ❌ Hard to test individually | ✅ Easy to test per type |
| Maintenance | ❌ High coupling | ✅ Low coupling |
| Consistency | ❌ Different from languages/ | ✅ Similar pattern to languages/ |
| File Count | ❌ Multiple files per type | ✅ One file per type |

## Conclusion

This design provides a clean, maintainable, and extensible structure for regex-based parsers, following established patterns in the codebase and addressing the current architectural issues.
