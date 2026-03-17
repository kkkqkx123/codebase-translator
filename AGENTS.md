# Codebase Translate

**Security Assurance**
Always avoid the use of unwrap. In testing, substitute with expect.
Refrain from using unsafe methods except where directly involving low-level operations.
All instances of unsafe usage must be explicitly documented in the unsafe.md file within the docs\archive directory.

**Type Design Guidelines**
Minimise the use of dynamic dispatch forms such as `dyn`, always prioritising deterministic types.
All instances of dynamic dispatch must be explicitly documented in the `dynamic.md` file within the `docs\archive` directory.

## Project Overview

Codebase Translate is a command-line tool developed in **Rust** that automatically translates comments, documentation strings, and error messages within codebases. It integrates multiple translation APIs (DeepLX, LLM, Tencent Cloud), supports various programming languages, and offers intelligent incremental translation alongside CI/CD integration capabilities.

**Core Value:**
- **CI/CD-Friendly**: Configuration-driven design for seamless automation workflow integration
- **Intelligent Incremental Processing**: File-hash-based caching mechanism processing only modified files
- **Precise Extraction**: Extracts text containing target language characters (e.g., Chinese or English) based on configured language settings
- **Unified Encoding**: Fully UTF-8-based operation with automatic non-standard encoding conversion
- **Type Safety**: Leverages Rust's type system for compile-time correctness guarantees

## Technology Stack

| Category | Technology |
|------|------|
| Language | Rust 1.80+ |
| Configuration Format | TOML |
| Concurrency Model | Tokio Async Runtime |
| Rate Limiting | governor |
| HTTP Client | reqwest |
| Language Detection | whatlang |
| Compression Support | brotli, flate2 |
| Serialization | serde (JSON, TOML, MessagePack) |
| Environment Variables | dotenvy |
| Tree-sitter | tree-sitter (Native Rust bindings) |
| Logging | tracing |
| Error Handling | thiserror, anyhow |

## Command Execution

**Quality Verify**
```shell
cargo clippy --all-targets --all-features
cargo fmt --check
```

**Build**
```shell
cargo build --release
```

**Test**
```shell
cargo test --all
```

## Workflow

```
Scan directory → Detect encoding → Parse file → Check cache → Batch translation → Write back to file → Update cache
```

## Translation Providers

- **DeepLX**: Free translation service based on DeepL
- **LLM**: Supports multiple large language model providers (OpenAI, Anthropic, etc.)
- **Tencent Cloud**: Tencent Cloud Machine Translation Service

### Error Handling
```rust
#[derive(Error, Debug)]
pub enum TranslateError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {message}")]
    Parse { message: String },
}
```
