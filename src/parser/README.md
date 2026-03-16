# Parser Module

Code parsing module with tree-sitter and regex fallback for extracting translatable content from source files.

## Architecture

The parser module is organized into several submodules:

```
src/parser/
├── core/              # Generic extraction framework
├── queries/           # Tree-sitter query builders
├── languages/         # Language-specific parsers
├── strategy/          # Extraction strategies
├── filter/            # Content filters
├── function_patterns/ # Function/macro pattern classification
├── language/          # Language detection
├── tree_sitter/       # Tree-sitter integration
├── regex/             # Regex-based fallback parsers
└── trait.rs           # Parser trait definition
```

## Core Framework

The `core/` module provides generic extraction utilities reusable across languages:

- **extractor.rs**: Generic `Extractor` trait and `ExtractionCandidate` types
- **query_executor.rs**: Tree-sitter query execution with capture filtering
- **string_processor.rs**: String literal cleaning and escape sequence handling
- **position_tracker.rs**: Source position tracking utilities

### Example: Using StringProcessor

```rust
use crate::parser::core::StringProcessor;

let processor = StringProcessor::new();

// Clean regular string
let cleaned = processor.clean_string_literal("\"hello world\"");
assert_eq!(cleaned, "hello world");

// Clean raw string
let raw = r##"r#"hello "world""#"##;
let cleaned = processor.clean_string_literal(raw);
assert_eq!(cleaned, r#"hello "world""#);

// Unescape sequences
let unescaped = processor.unescape("hello\\nworld");
assert_eq!(unescaped, "hello\nworld");
```

## Query System

The `queries/` module provides predefined queries and a query builder:

### Predefined Queries

- **CommentQueries**: Comment extraction for Rust, Go, Python, JavaScript, Java, C/C++
- **StringQueries**: String literal extraction for various languages
- **FunctionQueries**: Function call extraction with pattern matching

### Query Builder

```rust
use crate::parser::queries::QueryBuilder;

let query = QueryBuilder::new("rust")
    .with_line_comments()
    .with_doc_comments()
    .with_string_literals()
    .build();
```

## Language Parsers

The `languages/` module contains language-specific implementations:

### Rust Parser

**Location**: `src/parser/languages/rust/`

**Files**:
- `parser.rs`: Main parser implementation
- `queries.rs`: Rust-specific tree-sitter queries
- `patterns.rs`: Rust macro pattern classification

**Features**:

1. **Comment Extraction**
   - Line comments (`//`)
   - Block comments (`/* */`)

2. **Doc Comment Extraction**
   - Outer doc comments (`///`)
   - Inner doc comments (`//!`)
   - Block doc comments (`/** */`)

3. **Macro String Extraction**
   - Error macros: `panic!`, `assert!`, `assert_eq!`, `assert_ne!`, `unreachable!`, `unimplemented!`, `todo!`
   - Format macros: `format!`, `print!`, `println!`, `eprint!`, `eprintln!`, `write!`, `writeln!`
   - Log macros: `println!`, `eprintln!`
   - Debug macros: `dbg!`

4. **String Literal Support**
   - Regular strings: `"hello"`
   - Raw strings: `r"hello"`, `r#"hello "world""#`

**Usage**:

```rust
use crate::parser::{RustParser, ParserConfig};
use crate::parser::strategy::ConfigBasedStrategy;
use crate::parser::filter::ContentFilter;
use std::sync::Arc;

let config = ParserConfig::default();
let strategy = Arc::new(ConfigBasedStrategy::default());
let filter = Arc::new(ContentFilter::default().unwrap());

let parser = RustParser::new(config, strategy, filter).unwrap();

// Parse a file
let file = File::new(PathBuf::from("test.rs"), content, "utf-8");
let units = parser.parse(&file).await.unwrap();
```

### Adding New Languages

To add support for a new language:

1. Create directory: `src/parser/languages/<lang>/`
2. Create files:
   - `mod.rs`: Module exports
   - `parser.rs`: Parser implementation
   - `queries.rs`: Language-specific queries
   - `patterns.rs`: Language-specific patterns (optional)

3. Register in `src/parser/languages/mod.rs`:

```rust
pub mod <lang>;
pub use <lang>::<Lang>Parser;
```

4. Add to `ParserCoordinator` in `tree_sitter.rs`

## Extraction Strategies

The `strategy/` module provides filtering logic for extraction:

- **ExtractionStrategy**: Trait for custom extraction logic
- **ConfigBasedStrategy**: Configuration-driven extraction
- **CombinedStrategy**: Combine multiple strategies (All/Any)

### Strategy Node Types

- `Comment`: Regular comments
- `DocString`: Documentation strings
- `ErrorMessage`: Error messages
- `FormatString`: Format strings
- `LogMessage`: Log messages

## Content Filters

The `filter/` module provides content filtering:

- **ContentFilter**: Main filter implementation
- **FilterConfig**: Configuration for filtering

### Filter Options

- Min/max content length
- Exclude keywords (regex)
- Exclude patterns (regex)
- Include patterns (regex)
- Placeholder detection
- Code pattern detection

## Function Patterns

The `function_patterns/` module classifies functions/macros:

### Categories

- **Error**: Error-related functions/macros
- **Format**: Formatting functions
- **Log**: Logging functions
- **Debug**: Debug functions

### Usage

```rust
use crate::parser::function_patterns::{
    FunctionPatternRegistry, FunctionCategory
};

let registry = FunctionPatternRegistry::new();

// Classify a function
let category = registry.classify("rust", "panic!");
assert_eq!(category, Some(FunctionCategory::Error));

// Check specific categories
assert!(registry.is_error_function("rust", "panic!"));
assert!(registry.is_format_function("rust", "format!"));
```

## Language Detection

The `language/` module detects text language:

```rust
use crate::parser::language::LanguageDetector;

let detector = LanguageDetector::new();
let info = detector.detect("Hello world");

println!("Script: {:?}", info.script);
println!("Languages: {:?}", info.languages);
```

### Supported Scripts

- Latin
- CJK (Chinese, Japanese, Korean)
- Arabic
- Hebrew
- Greek
- Cyrillic

## Tree-sitter Integration

The `tree_sitter/` module provides:

- **TreeSitterParser**: Generic tree-sitter based parser
- **ParserCoordinator**: Coordinates multiple parsers
- **TreeSitterParserFactory**: Factory for creating parsers

### ParserCoordinator

```rust
use crate::parser::{ParserCoordinator, ParserConfig};

let coordinator = ParserCoordinator::with_defaults(ParserConfig::default()).unwrap();

// Parse a file (automatically selects appropriate parser)
let units = coordinator.parse_file(&file).await.unwrap();

// Get supported extensions
let extensions = coordinator.supported_extensions();
```

## Regex Parsers

The `regex/` module provides fallback parsers for simple file types:

- HTML parser
- SQL parser
- Shell script parser

## Testing

Run parser tests:

```bash
cargo test --package codebase-translate --lib -- parser::
```

### Test Coverage

- Core framework: `parser::core::`
- Query system: `parser::queries::`
- Language parsers: `parser::languages::`
- Strategies: `parser::strategy::`
- Filters: `parser::filter::`
- Function patterns: `parser::function_patterns::`

## Configuration

### ParserConfig

```rust
use crate::parser::tree_sitter::ParserConfig;

let config = ParserConfig {
    extract_comments: true,
    extract_docstrings: true,
    extract_strings: true,
    trim_content: true,
    min_content_length: 1,
    max_content_length: 10000,
};
```

### ExtractionConfig

```rust
use crate::parser::strategy::ExtractionConfig;

let config = ExtractionConfig {
    comments: true,
    docstrings: true,
    error_messages: true,
    format_strings: true,
    log_messages: true,
};
```

### FilterConfig

```rust
use crate::parser::filter::FilterConfig;

let config = FilterConfig {
    min_length: 2,
    max_length: 10000,
    exclude_keywords: vec!["TODO".to_string()],
    exclude_patterns: vec![],
    include_patterns: vec![],
    placeholder_patterns: vec![],
    code_patterns: vec![],
};
```

## Performance Considerations

1. **Query Caching**: Tree-sitter queries are compiled once and reused
2. **Lazy Loading**: Parsers are created on-demand
3. **Parallel Processing**: Multiple files can be parsed concurrently
4. **Memory Efficiency**: Streaming iteration for large files

## Error Handling

All parser operations return `Result<T, TranslateError>`:

```rust
use crate::core::error::TranslateError;

match parser.parse(&file).await {
    Ok(units) => println!("Extracted {} units", units.len()),
    Err(TranslateError::Parse(msg)) => eprintln!("Parse error: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Future Enhancements

1. **Additional Languages**: Python, JavaScript, Java, Go, C/C++
2. **Incremental Parsing**: Only re-parse changed regions
3. **Custom Extractors**: Plugin system for custom extraction logic
4. **AST Analysis**: Deeper semantic analysis for better extraction
