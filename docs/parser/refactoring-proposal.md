# Parser 模块重构方案

## 1. 当前 rust_parser.rs 职责分析

### 1.1 当前包含的职责

| 职责类别 | 具体内容 | 行数范围 |
|---------|---------|---------|
| **宏模式定义** | `RustMacroPatterns` 结构体及方法 | 25-67 |
| **Parser 结构体** | `RustParser` 及配置 | 70-85 |
| **AST 解析** | `parse_tree` 方法 | 87-97 |
| **提取协调** | `extract_units` 方法 | 99-135 |
| **注释提取** | `extract_comments` 方法 | 137-154 |
| **文档字符串提取** | `extract_docstrings` 方法 | 156-173 |
| **宏字符串提取** | `extract_macro_strings` 方法 | 175-258 |
| **文档属性提取** | `extract_doc_attributes` 方法 | 260-269 |
| **通用提取逻辑** | `extract_with_query` 方法 | 271-349 |
| **字符串处理** | `clean_string_literal`, `unescape_string` | 351-444 |
| **Parser Trait 实现** | `Parser` trait 实现 | 446-470 |
| **辅助函数** | `is_only_symbols`, `is_punctuation` | 472-520 |
| **Tree-sitter Queries** | 常量 query 定义 | 522-544 |
| **测试代码** | 所有测试函数 | 546-950+ |

### 1.2 职责统计

```
总代码行数: ~950 行
- 结构体定义: ~60 行 (6%)
- 提取逻辑: ~300 行 (32%)
- 字符串处理: ~100 行 (11%)
- Queries: ~25 行 (3%)
- 测试代码: ~400 行 (42%)
- 其他: ~65 行 (6%)
```

## 2. 问题识别

### 2.1 单一文件职责过多
- 950+ 行代码，违反了单一职责原则
- 混合了 query 定义、提取逻辑、字符串处理、测试代码

### 2.2 新增语言时的扩展问题
当前添加新语言需要：
1. 创建新的 `xxx_parser.rs` 文件
2. 复制大量通用提取逻辑
3. 重复定义类似的 query 结构
4. 目录结构会变得混乱

### 2.3 可复用性低
- `extract_with_query` 是通用逻辑，但被绑定在 RustParser 中
- 字符串处理函数（`clean_string_literal`, `unescape_string`）是通用的
- 宏模式分类逻辑可以抽象为通用框架

## 3. 重构方案

### 3.1 目标目录结构

```
src/parser/
├── mod.rs                    # 模块导出
├── trait.rs                  # Parser trait 定义
│
├── core/                     # 核心提取框架
│   ├── mod.rs
│   ├── extractor.rs          # 通用提取器 trait
│   ├── query_executor.rs     # Tree-sitter query 执行器
│   ├── string_processor.rs   # 字符串处理工具
│   └── position_tracker.rs   # 位置跟踪工具
│
├── languages/                # 语言特定实现
│   ├── mod.rs
│   ├── rust/                 # Rust 语言支持
│   │   ├── mod.rs
│   │   ├── parser.rs         # RustParser 实现（简化版）
│   │   ├── queries.rs        # Tree-sitter queries
│   │   ├── patterns.rs       # Rust 特定模式（宏、属性等）
│   │   └── tests.rs          # Rust 解析器测试
│   │
│   ├── go/                   # Go 语言支持
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   ├── queries.rs
│   │   └── tests.rs
│   │
│   └── python/               # Python 语言支持（示例）
│       ├── mod.rs
│       ├── parser.rs
│       ├── queries.rs
│       └── tests.rs
│
├── queries/                  # 可复用的 query 构建器
│   ├── mod.rs
│   ├── builder.rs            # Query 构建器
│   ├── comment_queries.rs    # 注释相关 queries
│   ├── string_queries.rs     # 字符串相关 queries
│   └── function_queries.rs   # 函数调用 queries
│
├── strategy/                 # 提取策略
│   ├── mod.rs
│   ├── trait.rs              # ExtractionStrategy trait
│   ├── config_based.rs       # ConfigBasedStrategy
│   └── combined.rs           # CombinedStrategy
│
├── filter/                   # 内容过滤器
│   ├── mod.rs
│   ├── trait.rs              # ContentFilter trait
│   ├── regex_filter.rs       # 基于正则的过滤器
│   └── length_filter.rs      # 长度过滤器
│
├── patterns/                 # 函数/宏模式
│   ├── mod.rs
│   ├── registry.rs           # FunctionPatternRegistry
│   └── categories.rs         # FunctionCategory 定义
│
├── language_detection/       # 语言检测
│   ├── mod.rs
│   └── detector.rs
│
└── tree_sitter/              # Tree-sitter 集成
    ├── mod.rs
    ├── parser.rs             # TreeSitterParser
    ├── coordinator.rs        # ParserCoordinator
    └── factory.rs            # ParserFactory
```

### 3.2 核心组件拆分

#### 3.2.1 提取通用框架 (src/parser/core/)

**extractor.rs** - 通用提取器 trait:
```rust
/// 通用提取器 trait
pub trait Extractor: Send + Sync {
    /// 提取类型
    fn extraction_type(&self) -> ExtractionType;
    
    /// 执行提取
    fn extract(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<ExtractionCandidate>>;
}

/// 提取候选
pub struct ExtractionCandidate {
    pub text: String,
    pub start_pos: Position,
    pub end_pos: Position,
    pub node_type: StrategyNodeType,
    pub metadata: HashMap<String, String>,
}
```

**query_executor.rs** - Query 执行器:
```rust
/// Tree-sitter query 执行器
pub struct QueryExecutor {
    query: Query,
    capture_filter: CaptureFilter,
}

impl QueryExecutor {
    pub fn new(query: Query) -> Self {
        Self {
            query,
            capture_filter: CaptureFilter::default(),
        }
    }
    
    /// 执行 query 并返回匹配的节点
    pub fn execute<'a>(
        &self,
        root_node: &Node,
        content: &'a str,
    ) -> Result<Vec<QueryMatch<'a>>> {
        // 通用执行逻辑
    }
}
```

**string_processor.rs** - 字符串处理:
```rust
/// 字符串处理器
pub struct StringProcessor;

impl StringProcessor {
    /// 清理字符串字面量（移除引号等）
    pub fn clean_string_literal(text: &str) -> String {
        // 通用实现
    }
    
    /// 处理转义序列
    pub fn unescape(text: &str) -> String {
        // 通用实现
    }
    
    /// 处理原始字符串
    pub fn process_raw_string(text: &str) -> String {
        // 通用实现
    }
}
```

#### 3.2.2 Query 构建器 (src/parser/queries/)

**builder.rs**:
```rust
/// Query 构建器
pub struct QueryBuilder {
    language: Language,
    patterns: Vec<QueryPattern>,
}

impl QueryBuilder {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            patterns: Vec::new(),
        }
    }
    
    /// 添加注释提取模式
    pub fn with_comments(mut self) -> Self {
        self.patterns.push(QueryPattern::Comments);
        self
    }
    
    /// 添加文档字符串提取模式
    pub fn with_docstrings(mut self) -> Self {
        self.patterns.push(QueryPattern::Docstrings);
        self
    }
    
    /// 添加函数调用提取模式
    pub fn with_function_calls(mut self, functions: &[&str]) -> Self {
        self.patterns.push(QueryPattern::FunctionCalls(functions.to_vec()));
        self
    }
    
    /// 构建 query 字符串
    pub fn build(&self) -> String {
        // 根据 patterns 构建 query
    }
}
```

**comment_queries.rs**:
```rust
/// 注释提取 queries
pub struct CommentQueries;

impl CommentQueries {
    /// 获取通用注释 query
    pub fn line_comment() -> &'static str {
        "(line_comment) @comment"
    }
    
    pub fn block_comment() -> &'static str {
        "(block_comment) @comment"
    }
    
    /// 获取 Rust 特定 doc comment query
    pub fn rust_doc_comment() -> &'static str {
        r#"
        ((line_comment) @docstring
          (#match? @docstring "^///"))
        
        ((line_comment) @docstring
          (#match? @docstring "^//!"))
        "#
    }
    
    /// 获取 Go 特定 doc comment query
    pub fn go_doc_comment() -> &'static str {
        r#"
        ((comment) @docstring
          (#match? @docstring "^// "))
        "#
    }
}
```

#### 3.2.3 语言特定实现 (src/parser/languages/)

**languages/rust/mod.rs**:
```rust
//! Rust 语言解析器

pub mod parser;
pub mod patterns;
pub mod queries;

pub use parser::RustParser;
pub use patterns::RustPatterns;
```

**languages/rust/parser.rs** - 简化后的 RustParser:
```rust
//! Rust 语言解析器实现

use crate::parser::core::{Extractor, QueryExecutor, StringProcessor};
use crate::parser::languages::rust::queries::RustQueries;
use crate::parser::languages::rust::patterns::RustPatterns;

/// Rust 解析器
pub struct RustParser {
    config: ParserConfig,
    strategy: Arc<dyn ExtractionStrategy>,
    filter: Arc<ContentFilter>,
    patterns: RustPatterns,
    extractors: Vec<Box<dyn Extractor>>,
}

impl RustParser {
    pub fn new(
        config: ParserConfig,
        strategy: Arc<dyn ExtractionStrategy>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        let patterns = RustPatterns::new();
        let extractors = Self::create_extractors(&config, &patterns)?;
        
        Ok(Self {
            config,
            strategy,
            filter,
            patterns,
            extractors,
        })
    }
    
    fn create_extractors(
        config: &ParserConfig,
        patterns: &RustPatterns,
    ) -> Result<Vec<Box<dyn Extractor>>> {
        let mut extractors: Vec<Box<dyn Extractor>> = Vec::new();
        
        if config.extract_comments {
            extractors.push(Box::new(CommentExtractor::new(
                tree_sitter_rust::LANGUAGE.into(),
                RustQueries::comments(),
            )?));
        }
        
        if config.extract_docstrings {
            extractors.push(Box::new(DocstringExtractor::new(
                tree_sitter_rust::LANGUAGE.into(),
                RustQueries::docstrings(),
            )?));
        }
        
        if config.extract_strings {
            extractors.push(Box::new(MacroExtractor::new(
                tree_sitter_rust::LANGUAGE.into(),
                RustQueries::macros(),
                patterns.clone(),
            )?));
        }
        
        Ok(extractors)
    }
}

#[async_trait]
impl Parser for RustParser {
    async fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let content = file.content_string()?;
        let tree = self.parse_tree(&content)?;
        let mut units = Vec::new();
        
        for extractor in &self.extractors {
            let candidates = extractor.extract(&tree.root_node(), &content, &file_path)?;
            
            for candidate in candidates {
                // 应用过滤器和策略
                if self.filter.should_translate(&candidate.text)
                    && self.strategy.should_extract(candidate.node_type, &ctx)
                {
                    units.push(self.candidate_to_unit(candidate, file_path));
                }
            }
        }
        
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));
        Ok(units)
    }
    
    fn supports(&self, filename: &str) -> bool {
        filename.ends_with(".rs")
    }
}
```

**languages/rust/queries.rs**:
```rust
//! Rust 特定的 tree-sitter queries

pub struct RustQueries;

impl RustQueries {
    /// 注释提取 query
    pub fn comments() -> &'static str {
        r#"
        (line_comment) @comment
        (block_comment) @comment
        "#
    }
    
    /// 文档字符串提取 query
    pub fn docstrings() -> &'static str {
        r#"
        ((line_comment) @docstring
          (#match? @docstring "^///"))
        
        ((line_comment) @docstring
          (#match? @docstring "^//!"))
        
        ((block_comment) @docstring
          (#match? @docstring "^/\\*\\*"))
        "#
    }
    
    /// 宏调用提取 query
    pub fn macros() -> &'static str {
        r#"
        (macro_invocation
          macro: (identifier) @macro_name
          (token_tree
            (string_literal) @macro_string))
        
        (macro_invocation
          macro: (identifier) @macro_name
          (token_tree
            (raw_string_literal) @macro_string))
        "#
    }
    
    /// 属性提取 query
    pub fn attributes() -> &'static str {
        r#"
        (attribute
          (identifier) @attr_name
          arguments: (token_tree
            (string_literal) @attr_value)?)
        "#
    }
}
```

**languages/rust/patterns.rs**:
```rust
//! Rust 特定模式定义

use crate::parser::patterns::{FunctionPatternRegistry, FunctionCategory};

/// Rust 模式集合
#[derive(Clone)]
pub struct RustPatterns {
    registry: FunctionPatternRegistry,
}

impl RustPatterns {
    pub fn new() -> Self {
        let mut registry = FunctionPatternRegistry::new();
        Self::register_patterns(&mut registry);
        Self { registry }
    }
    
    fn register_patterns(registry: &mut FunctionPatternRegistry) {
        // 错误宏
        registry.register_functions("rust", FunctionCategory::Error, &[
            "panic!", "assert!", "assert_eq!", "assert_ne!",
            "unreachable!", "unimplemented!", "todo!",
        ]);
        
        // 格式化宏
        registry.register_functions("rust", FunctionCategory::Format, &[
            "format!", "print!", "println!", "eprint!", "eprintln!",
            "write!", "writeln!",
        ]);
        
        // 日志宏
        registry.register_functions("rust", FunctionCategory::Log, &[
            "println!", "eprintln!",
        ]);
        
        // 调试宏
        registry.register_functions("rust", FunctionCategory::Debug, &[
            "dbg!",
        ]);
    }
    
    pub fn classify_macro(&self, name: &str) -> Option<FunctionCategory> {
        self.registry.classify("rust", name)
    }
}
```

## 4. 重构步骤

### 4.1 第一阶段：核心框架提取
1. 创建 `src/parser/core/` 目录
2. 将 `extract_with_query` 逻辑提取到 `core/query_executor.rs`
3. 将字符串处理函数提取到 `core/string_processor.rs`
4. 创建 `core/extractor.rs` 定义通用提取器 trait

### 4.2 第二阶段：Query 系统重构
1. 创建 `src/parser/queries/` 目录
2. 将现有 query 常量迁移到 `queries/comment_queries.rs`
3. 创建 `QueryBuilder` 用于动态构建 queries

### 4.3 第三阶段：语言模块重构
1. 创建 `src/parser/languages/` 目录
2. 将 `rust_parser.rs` 拆分为 `languages/rust/` 下的多个文件
3. 更新 `mod.rs` 导出

### 4.4 第四阶段：其他语言支持
1. 按照相同模式添加 `languages/go/`
2. 添加 `languages/python/`（示例）

## 5. 好处

### 5.1 代码组织
- 单一职责：每个文件只做一件事
- 清晰的层次结构：core → queries → languages
- 易于导航：按功能而非语言组织

### 5.2 可维护性
- 修改 query 不需要修改解析器逻辑
- 添加新语言只需创建新的 language 目录
- 测试代码与实现代码分离

### 5.3 可复用性
- `core/` 中的组件可被所有语言复用
- `queries/` 中的通用 queries 可跨语言使用
- `QueryBuilder` 支持动态构建 queries

### 5.4 可扩展性
- 添加新语言只需：
  1. 创建 `languages/<lang>/` 目录
  2. 实现 `queries.rs`, `patterns.rs`, `parser.rs`
  3. 在 `languages/mod.rs` 中注册
- 添加新提取类型只需：
  1. 在 `core/` 中创建新的 extractor
  2. 在语言解析器中使用

## 6. 迁移策略

### 6.1 向后兼容
- 保持 `mod.rs` 中的公开 API 不变
- 使用 `pub use` 重新导出移动后的类型

### 6.2 渐进式迁移
1. 先创建新目录结构
2. 逐步迁移代码
3. 保持测试通过
4. 最后删除旧代码

### 6.3 示例：保持向后兼容
```rust
// src/parser/mod.rs
// 旧导出（保持兼容）
pub use rust_parser::RustParser;

// 新导出
pub mod languages {
    pub use super::languages_impl::rust;
}

// 内部实现
mod languages_impl {
    pub mod rust {
        pub use super::super::languages::rust::RustParser;
    }
}
```
