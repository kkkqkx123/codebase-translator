# Parser 模块设计方案

## 1. 概述

本文档描述了 `src/parser` 模块的设计方案，该模块负责从源代码文件中提取可翻译的文本内容（注释、文档字符串、错误消息等）。

## 2. 架构设计

### 2.1 整体架构

Parser 模块采用分层架构设计：

```
┌─────────────────────────────────────────┐
│         ParserCoordinator               │
│    (解析器协调器，管理多个解析器)          │
└──────────────┬──────────────────────────┘
               │
       ┌───────┴────────┐
       │                │
┌──────▼──────┐  ┌─────▼──────┐
│TreeSitter   │  │  Regex     │
│Parser       │  │  Parser    │
│(精确解析)    │  │  (回退)    │
└──────┬──────┘  └─────┬──────┘
       │                │
       └────────┬───────┘
                │
       ┌────────▼─────────┐
       │  Parser Trait    │
       │  (统一接口)       │
       └──────────────────┘
```

### 2.2 核心组件

#### 2.2.1 Parser Trait (trait.rs)

定义统一的解析器接口：

```rust
#[async_trait]
pub trait Parser: Send + Sync {
    async fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>>;
    fn supports(&self, filename: &str) -> bool;
    fn supported_extensions(&self) -> &[&str];
}
```

#### 2.2.2 TreeSitterParser (tree_sitter.rs)

基于 tree-sitter 的精确解析器，支持多种编程语言：

- 使用 tree-sitter 语法树进行精确解析
- 支持注释、文档字符串、字符串字面量的提取
- 通过查询语言（Query Language）定义提取规则
- 支持语言检测和过滤

#### 2.2.3 RegexParser (regex.rs)

基于正则表达式的回退解析器：

- 用于 tree-sitter 不支持的语言
- 处理简单的文件类型（如 Markdown、Shell 脚本）
- 提供通用回退机制

#### 2.2.4 ParserCoordinator (tree_sitter.rs)

解析器协调器，负责：

- 管理多个解析器实例
- 根据文件扩展名选择合适的解析器
- 提供统一的解析入口

## 3. TreeSitterParser 详细设计

### 3.1 核心结构

```rust
pub struct TreeSitterParser {
    config: ParserConfig,
    language_config: LanguageConfig,
}

pub struct LanguageConfig {
    pub name: String,
    pub extensions: Vec<String>,
    pub language: TSLanguage,
    pub comment_query: String,
    pub docstring_query: Option<String>,
    pub string_query: Option<String>,
}

pub struct ParserConfig {
    pub extract_comments: bool,
    pub extract_docstrings: bool,
    pub extract_strings: bool,
    pub min_content_length: usize,
    pub max_content_length: usize,
    pub trim_content: bool,
}
```

### 3.2 解析流程

```
输入文件
   │
   ▼
解析内容为语法树 (parse_tree)
   │
   ▼
遍历语法树节点 (extract_units)
   │
   ├─→ 提取注释 (comment_query)
   ├─→ 提取文档字符串 (docstring_query)
   └─→ 提取字符串字面量 (string_query)
   │
   ▼
过滤和验证
   │
   ├─→ 长度检查
   ├─→ 内容过滤（符号、占位符）
   └─→ 语言检测
   │
   ▼
生成 TranslationUnit
   │
   ▼
返回结果
```

### 3.3 Tree-sitter 查询设计

#### 3.3.1 Rust 语言查询

**注释查询：**
```lisp
(line_comment) @comment
(block_comment) @comment
```

**文档字符串查询：**
```lisp
((line_comment) @docstring
  (#match? @docstring "^///"))

((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
```

**字符串查询：**
```lisp
(string_literal) @string
(raw_string_literal) @string
```

#### 3.3.2 Go 语言查询

**注释查询：**
```lisp
(comment) @comment
```

**字符串查询：**
```lisp
(raw_string_literal) @string
(interpreted_string_literal) @string
```

## 4. Rust 语言解析器实现

### 4.1 Rust 语言特性

Rust 语言有以下特殊特性需要处理：

1. **文档注释**：
   - `///` - 外部文档注释
   - `//!` - 内部文档注释
   - `/** ... */` - 块文档注释

2. **宏调用**：
   - `panic!()`, `assert!()`, `format!()` 等宏
   - 需要识别宏调用中的字符串参数

3. **字符串类型**：
   - 普通字符串：`"hello"`
   - 原始字符串：`r#"hello"#`, `r##"hello"##`
   - 字节字符串：`b"hello"`

4. **属性**：
   - `#[doc = "..."]` - 文档属性
   - 需要提取属性中的文档字符串

### 4.2 Rust 解析器增强

在现有 tree-sitter 解析器基础上，需要添加 Rust 特定的处理逻辑：

#### 4.2.1 宏调用识别

```rust
// 识别 Rust 宏调用并提取字符串参数
const RUST_MACRO_QUERY: &str = r#"
(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (string_literal) @macro_string))
"#;

// 支持的宏列表
const RUST_MACROS: &[&str] = &[
    "panic!", "assert!", "assert_eq!", "assert_ne!",
    "unreachable!", "unimplemented!", "todo!",
    "format!", "print!", "println!", "eprint!", "eprintln!",
    "write!", "writeln!", "dbg!",
];
```

#### 4.2.2 文档属性提取

```rust
// 提取 #[doc = "..."] 属性
const RUST_DOC_ATTR_QUERY: &str = r#"
(attribute
  (attribute_item
    (identifier) @attr_name
    (#eq? @attr_name "doc")
    (string_literal) @doc_content))
"#;
```

#### 4.2.3 函数分类映射

参考 Go 版本的 `function_patterns.go`，在 Rust 中实现类似的函数分类：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionCategory {
    Error,
    Format,
    Log,
}

pub struct FunctionPatterns {
    pub error_functions: HashSet<String>,
    pub format_functions: HashSet<String>,
    pub log_functions: HashSet<String>,
}

impl FunctionPatterns {
    pub fn rust() -> Self {
        Self {
            error_functions: [
                "panic!", "assert!", "assert_eq!", "assert_ne!",
                "unreachable!", "unimplemented!", "todo!",
            ].iter().map(|s| s.to_string()).collect(),
            format_functions: [
                "format!", "print!", "println!", "eprint!", "eprintln!",
                "write!", "writeln!",
            ].iter().map(|s| s.to_string()).collect(),
            log_functions: [
                "println!", "eprintln!",
            ].iter().map(|s| s.to_string()).collect(),
        }
    }
}
```

## 5. 与 Go 版本的对比

### 5.1 架构对比

| 特性 | Go 版本 | Rust 版本 |
|------|---------|-----------|
| 解析器接口 | `Parser` interface | `Parser` trait |
| 异步模型 | Goroutine + Channel | Tokio async/await |
| 错误处理 | `error` interface | `Result<T, E>` |
| AST 解析 | 语言原生 AST | tree-sitter 统一 AST |
| 语言检测 | 自定义实现 | whatlang 库 |
| 配置管理 | 结构体 + 方法 | 结构体 + impl |

### 5.2 功能对比

| 功能 | Go 版本 | Rust 版本 |
|------|---------|-----------|
| 注释提取 | ✓ | ✓ |
| 文档字符串 | ✓ | ✓ |
| 错误消息 | ✓ | ✓ |
| 格式化字符串 | ✓ | ✓ |
| 日志消息 | ✓ | ✓ |
| 函数调用识别 | ✓ | ✓ (需要增强) |
| 语言检测 | ✓ | ✓ |
| 内容过滤 | ✓ | ✓ |
| 策略模式 | ✓ | ✓ (待实现) |

### 5.3 代码组织对比

**Go 版本：**
```
internal/parser/
├── base.go              # 基础解析器
├── coordinator.go       # 协调器
├── parser.go            # 接口定义
├── strategy.go          # 策略模式
├── strategy_config.go   # 策略配置
├── go_parser.go         # Go 解析器
├── rust_parser.go       # Rust 解析器
├── common/
│   ├── filter.go        # 过滤器
│   ├── language.go      # 语言检测
│   ├── function_patterns.go  # 函数模式
│   └── ...
```

**Rust 版本：**
```
src/parser/
├── mod.rs               # 模块导出
├── trait.rs             # Parser trait
├── tree_sitter.rs       # Tree-sitter 解析器
├── regex.rs             # Regex 解析器
└── (待添加)
    ├── strategy.rs      # 策略模式
    ├── filter.rs        # 过滤器
    ├── language.rs      # 语言检测
    └── function_patterns.rs  # 函数模式
```

## 6. 第一阶段实现计划

### 6.1 目标

实现 Parser 模块的基础功能和 Rust 语言解析器。

### 6.2 任务清单

#### 6.2.1 基础设施 (已完成)

- [x] `trait.rs` - Parser trait 定义
- [x] `tree_sitter.rs` - Tree-sitter 解析器基础框架
- [x] `regex.rs` - Regex 回退解析器
- [x] `mod.rs` - 模块导出

#### 6.2.2 Rust 语言解析器增强 (待实现)

- [ ] 增强 Rust tree-sitter 查询
  - [ ] 添加宏调用查询
  - [ ] 添加文档属性查询
  - [ ] 优化文档注释识别

- [ ] 实现 Rust 特定的字符串提取
  - [ ] 原始字符串处理
  - [ ] 字节字符串过滤
  - [ ] 转义字符处理

- [ ] 实现函数模式识别
  - [ ] 创建 `function_patterns.rs`
  - [ ] 实现 Rust 宏分类
  - [ ] 支持自定义函数模式

#### 6.2.3 策略模式实现 (待实现)

- [ ] 创建 `strategy.rs`
  - [ ] 定义 `ExtractionStrategy` trait
  - [ ] 实现 `ConfigBasedStrategy`
  - [ ] 支持策略组合

- [ ] 创建 `strategy_config.rs`
  - [ ] 定义 `ExtractionConfig`
  - [ ] 实现配置加载
  - [ ] 支持动态配置

#### 6.2.4 过滤器增强 (待实现)

- [ ] 创建 `filter.rs`
  - [ ] 移植 Go 版本的过滤逻辑
  - [ ] 实现代码模式检测
  - [ ] 实现占位符检测
  - [ ] 支持自定义过滤规则

#### 6.2.5 语言检测增强 (待实现)

- [ ] 创建 `language.rs`
  - [ ] 集成 whatlang 库
  - [ ] 实现文字系统检测
  - [ ] 支持多语言混合检测
  - [ ] 实现语言置信度评分

#### 6.2.6 测试 (待实现)

- [ ] 单元测试
  - [ ] Rust 解析器测试
  - [ ] 策略模式测试
  - [ ] 过滤器测试
  - [ ] 语言检测测试

- [ ] 集成测试
  - [ ] 端到端解析测试
  - [ ] 多语言文件测试
  - [ ] 性能测试

### 6.3 实现优先级

**P0 (核心功能):**
1. Rust 宏调用识别
2. 文档属性提取
3. 函数模式分类

**P1 (重要功能):**
4. 策略模式实现
5. 过滤器增强
6. 语言检测增强

**P2 (优化功能):**
7. 性能优化
8. 错误处理改进
9. 测试覆盖率提升

## 7. 技术细节

### 7.1 Tree-sitter 查询优化

#### 7.1.1 查询缓存

```rust
pub struct QueryCache {
    cache: HashMap<String, Query>,
}

impl QueryCache {
    pub fn get_or_create(&mut self, language: &TSLanguage, query_str: &str) -> Result<&Query> {
        if !self.cache.contains_key(query_str) {
            let query = Query::new(language, query_str)?;
            self.cache.insert(query_str.to_string(), query);
        }
        Ok(self.cache.get(query_str).unwrap())
    }
}
```

#### 7.1.2 增量解析

```rust
pub struct IncrementalParser {
    parser: Parser,
    old_tree: Option<Tree>,
}

impl IncrementalParser {
    pub fn parse_incremental(&mut self, content: &str) -> Result<Tree> {
        self.parser.parse(content, self.old_tree.as_ref())
            .ok_or_else(|| TranslateError::Parse("Failed to parse".to_string()))
    }
}
```

### 7.2 内存管理

#### 7.2.1 节点池

```rust
pub struct NodePool {
    pool: Vec<Node>,
}

impl NodePool {
    pub fn acquire(&mut self) -> Option<Node> {
        self.pool.pop()
    }

    pub fn release(&mut self, node: Node) {
        self.pool.push(node);
    }
}
```

#### 7.2.2 字符串去重

```rust
pub use std::collections::HashSet;

pub fn deduplicate_units(units: Vec<TranslationUnit>) -> Vec<TranslationUnit> {
    let mut seen = HashSet::new();
    units.into_iter()
        .filter(|unit| seen.insert(unit.content.clone()))
        .collect()
}
```

### 7.3 并发处理

#### 7.3.1 并行解析

```rust
use tokio::task::JoinSet;

pub async fn parse_files_parallel(
    coordinator: &ParserCoordinator,
    files: Vec<File>,
) -> Result<Vec<Vec<TranslationUnit>>> {
    let mut join_set = JoinSet::new();

    for file in files {
        let coordinator = coordinator.clone();
        join_set.spawn(async move {
            coordinator.parse_file(&file).await
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        results.push(result??);
    }

    Ok(results)
}
```

### 7.4 错误处理

#### 7.4.1 错误类型定义

```rust
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to parse file: {0}")]
    ParseFailed(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

#### 7.4.2 错误恢复

```rust
pub fn recoverable_parse(content: &str) -> Result<Vec<TranslationUnit>> {
    match parse_with_treesitter(content) {
        Ok(units) => Ok(units),
        Err(e) => {
            tracing::warn!("Tree-sitter parsing failed, falling back to regex: {}", e);
            parse_with_regex(content)
        }
    }
}
```

## 8. 性能考虑

### 8.1 性能指标

- **解析速度**: > 1000 LOC/秒
- **内存使用**: < 100MB (对于 10MB 文件)
- **缓存命中率**: > 90% (对于重复文件)

### 8.2 优化策略

1. **查询预编译**: 在解析器初始化时预编译所有查询
2. **增量解析**: 对于小改动使用增量解析
3. **并行处理**: 多文件并行解析
4. **内存池**: 重用解析器实例
5. **惰性求值**: 只提取需要的节点类型

### 8.3 基准测试

```rust
#[bench]
fn bench_rust_parser(b: &mut Bencher) {
    let parser = TreeSitterParserFactory::create_rust_parser(ParserConfig::default()).unwrap();
    let content = std::fs::read_to_string("tests/fixtures/large_rust_file.rs").unwrap();

    b.iter(|| {
        let file = File::new(PathBuf::from("test.rs"), content.as_bytes().to_vec(), "utf-8");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(parser.parse(&file)).unwrap()
    });
}
```

## 9. 扩展性设计

### 9.1 新语言支持

添加新语言支持只需要：

1. 添加 tree-sitter 语言依赖
2. 定义语言查询
3. 在 `TreeSitterParserFactory` 中添加创建方法

示例：

```rust
// 1. 添加依赖
// tree-sitter-kotlin = "0.3"

// 2. 定义查询
const KOTLIN_COMMENT_QUERY: &str = r#"
(comment) @comment
"#;

// 3. 添加工厂方法
impl TreeSitterParserFactory {
    pub fn create_kotlin_parser(config: ParserConfig) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "kotlin".to_string(),
            extensions: vec!["kt".to_string(), "kts".to_string()],
            language: tree_sitter_kotlin::LANGUAGE.into(),
            comment_query: KOTLIN_COMMENT_QUERY.to_string(),
            docstring_query: Some(KOTLIN_DOCSTRING_QUERY.to_string()),
            string_query: Some(KOTLIN_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config)
    }
}
```

### 9.2 自定义提取规则

支持用户自定义提取规则：

```rust
pub struct CustomExtractionRule {
    pub name: String,
    pub query: String,
    pub node_type: NodeType,
    pub filter: Option<Box<dyn Fn(&str) -> bool>>,
}

impl TreeSitterParser {
    pub fn add_custom_rule(&mut self, rule: CustomExtractionRule) {
        self.custom_rules.push(rule);
    }
}
```

### 9.3 插件系统

支持第三方解析器插件：

```rust
pub trait ParserPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn supported_extensions(&self) -> &[&str];
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn ParserPlugin>>,
}

impl PluginManager {
    pub fn register_plugin(&mut self, plugin: Box<dyn ParserPlugin>) {
        self.plugins.push(plugin);
    }
}
```

## 10. 测试策略

### 10.1 单元测试

- 每个解析器的独立测试
- 查询语言正确性测试
- 过滤器逻辑测试
- 语言检测准确性测试

### 10.2 集成测试

- 端到端解析流程测试
- 多语言混合文件测试
- 大文件性能测试
- 错误恢复测试

### 10.3 回归测试

- 使用真实代码库作为测试集
- 对比 Go 版本和 Rust 版本的输出
- 确保功能一致性

### 10.4 测试覆盖率目标

- 核心解析逻辑: > 90%
- 过滤和检测逻辑: > 85%
- 错误处理路径: > 80%

## 11. 文档和示例

### 11.1 API 文档

- 所有公共 API 必须有文档注释
- 包含使用示例
- 说明性能特征

### 11.2 示例代码

```rust
use codebase_translate::parser::{ParserCoordinator, ParserConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ParserConfig {
        extract_comments: true,
        extract_docstrings: true,
        extract_strings: false,
        ..Default::default()
    };

    let coordinator = ParserCoordinator::new(config)?;

    let file = File::new(
        PathBuf::from("src/main.rs"),
        std::fs::read("src/main.rs")?,
        "utf-8",
    );

    let units = coordinator.parse_file(&file).await?;

    for unit in units {
        println!("{}: {}", unit.node_type, unit.content);
    }

    Ok(())
}
```

### 11.3 最佳实践

- 使用 `ParserCoordinator` 而不是直接使用具体解析器
- 合理配置 `ParserConfig` 以平衡性能和准确性
- 对于大文件，考虑使用增量解析
- 启用适当的过滤规则以提高翻译质量

## 12. 未来改进方向

### 12.1 短期改进

1. 完善策略模式实现
2. 增强语言检测准确性
3. 优化内存使用
4. 改进错误消息

### 12.2 中期改进

1. 支持更多编程语言
2. 实现插件系统
3. 添加可视化调试工具
4. 支持自定义提取规则

### 12.3 长期改进

1. 机器学习辅助的内容分类
2. 上下文感知的翻译单元提取
3. 实时解析和增量更新
4. 分布式解析支持

## 13. 参考资料

- [Tree-sitter 官方文档](https://tree-sitter.github.io/tree-sitter/)
- [Rust 语言规范](https://doc.rust-lang.org/reference/)
- [Go 版本实现](../../internal/parser/)
- [whatlang 文档](https://github.com/greyblake/whatlang-rs)

## 14. 附录

### 14.1 术语表

- **AST (Abstract Syntax Tree)**: 抽象语法树
- **Tree-sitter**: 增量解析系统
- **Query Language**: Tree-sitter 查询语言
- **Translation Unit**: 翻译单元，最小可翻译文本单位
- **NodeType**: 节点类型，标识文本的语义类型

### 14.2 相关文档

- [架构设计文档](../architecture.md)
- [Rust 迁移指南](../rust-migration/architecture-design.md)
- [依赖映射](../rust-migration/dependency-mapping.md)
- [需求规格](../spec/requirements.md)

### 14.3 变更日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 0.1.0 | 2025-03-15 | 初始版本，基础架构设计 |
