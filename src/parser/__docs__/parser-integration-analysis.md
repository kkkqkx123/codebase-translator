# Parser 模块与新架构集成分析

## 1. 架构概述

### 1.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    ParserCoordinator                        │
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │ TreeSitterParser │  │   RegexParser    │                │
│  │  (代码文件)       │  │  (文本/Markdown)  │                │
│  └────────┬─────────┘  └────────┬─────────┘                │
└───────────┼─────────────────────┼───────────────────────────┘
            │                     │
    ┌───────┴───────┐    ┌───────┴───────┐
    │ Language      │    │ StateMachine  │
    │ Parsers       │    │ Matcher       │
    │ (Rust/JS/...) │    │ (新增)        │
    └───────────────┘    └───────────────┘
```

### 1.2 核心组件

| 组件 | 职责 | 适用场景 |
|------|------|----------|
| `ParserCoordinator` | 协调器，路由文件到合适的解析器 | 所有文件 |
| `TreeSitterParser` | 基于 AST 的精确解析 | 支持的编程语言 |
| `RegexParser` | 基于正则的通用解析 | 纯文本、Markdown、Shell 等 |
| `StateMachineMatcher` | 状态机模式匹配 | 复杂提取场景（新增） |

---

## 2. 具体解析器集成分析

### 2.1 Tree-sitter 解析器集成

#### 2.1.1 架构设计

每个语言解析器遵循统一模式：

```rust
pub struct LanguageParser {
    config: ParserConfig,                    // 解析器配置
    strategy: Arc<ExtractionStrategyImpl>,   // 提取策略
    filter: Arc<ContentFilter>,              // 内容过滤器
    patterns: LanguagePatterns,              // 语言特定模式
    string_processor: StringProcessor,       // 字符串处理器
}
```

#### 2.1.2 提取流程

```
文件输入
    │
    ▼
┌─────────────────┐
│  parse_tree()   │  ← 使用 tree-sitter 生成 AST
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌────────┐ ┌──────────┐ ┌─────────────┐
│Comments│ │Docstrings│ │Macro Strings│
└───┬────┘ └────┬─────┘ └──────┬──────┘
    │           │              │
    ▼           ▼              ▼
┌─────────────────────────────────────┐
│      QueryExecutor.execute()        │  ← 执行 tree-sitter 查询
└───────────────┬─────────────────────┘
                │
    ┌───────────┼───────────┐
    ▼           ▼           ▼
┌────────┐ ┌────────┐ ┌──────────┐
│ 长度过滤 │ │内容过滤 │ │ 策略过滤  │
└────┬───┘ └───┬────┘ └────┬─────┘
     │         │           │
     └─────────┴───────────┘
                │
                ▼
        ┌───────────────┐
        │ TranslationUnit│
        └───────────────┘
```

#### 2.1.3 关键代码示例

**Rust 解析器 - 注释提取：**

```rust
fn extract_comments(&self, root_node: &Node, content: &str, file_path: &str) 
    -> Result<Vec<TranslationUnit>> 
{
    // 1. 创建查询执行器
    let executor = QueryExecutor::from_string(
        &tree_sitter_rust::LANGUAGE.into(),
        RustQueries::all_comments(),
    )?;

    // 2. 执行查询
    let matches = executor.execute(root_node, content)?;
    
    // 3. 处理匹配结果
    for m in matches {
        let text = m.text;
        
        // 4. 应用过滤器链
        if !self.filter.should_translate(text) { continue; }
        
        // 5. 应用策略
        let ctx = ExtractionContext::new(text);
        if !self.strategy.should_extract(StrategyNodeType::Comment, &ctx) {
            continue;
        }
        
        // 6. 创建翻译单元
        let unit = TranslationUnit::new(...);
        units.push(unit);
    }
}
```

#### 2.1.4 与新架构的集成点

| 集成点 | 说明 | 文件位置 |
|--------|------|----------|
| 策略系统 | 通过 `ExtractionStrategyImpl` 统一控制提取行为 | `src/parser/strategy.rs` |
| 过滤系统 | 通过 `ContentFilter` 进行语言/脚本过滤 | `src/parser/filter.rs` |
| 查询执行 | 通过 `QueryExecutor` 执行 tree-sitter 查询 | `src/parser/core/query_executor.rs` |
| 字符串处理 | 通过 `StringProcessor` 清理字符串字面量 | `src/parser/core/string_processor.rs` |

### 2.2 Regex 解析器集成

#### 2.2.1 新架构改进

**旧架构（单一文件）：**
```
regex.rs
├── RegexParserConfig
├── RegexParser
├── RegexParserFactory
└── 辅助函数
```

**新架构（模块化）：**
```
regex/
├── mod.rs           # 模块入口
├── config.rs        # 配置结构
├── parser.rs        # 主解析器
├── state_machine.rs # 状态机匹配器（新增）
├── presets.rs       # 预设解析器
├── factory.rs       # 工厂
└── utils.rs         # 工具函数
```

#### 2.2.2 状态机匹配器集成

**用途：** 处理复杂的多步骤模式匹配，如 i18n 调用提取

**配置示例：**
```toml
[[parser.extraction.string_literals.custom_patterns.state_machine_patterns]]
name = "i18n_with_default"
category = "other"
initial_state = "start"
accepting_states = ["extract"]

[[parser.extraction.string_literals.custom_patterns.state_machine_patterns.states]]
name = "start"
regex = 't\s*\(\s*["\'][^"\']+["\']\s*,\s*["\']'
transitions = [{ target = "extract" }]

[[parser.extraction.string_literals.custom_patterns.state_machine_patterns.states]]
name = "extract"
regex = '([^"\']+)'
capture_group = 1
is_final = true
```

**集成到 RegexParser：**

```rust
pub struct RegexParser {
    config: ParserConfig,
    regex_config: RegexParserConfig,
    line_comment_regex: Option<Regex>,
    block_comment_regex: Option<Regex>,
    doc_comment_regex: Option<Regex>,
    string_regex: Option<Regex>,
    state_machine_matchers: Vec<StateMachineMatcher>,  // 新增
}

fn parse_content(&self, content: &str, file_path: &str) 
    -> Result<Vec<TranslationUnit>> 
{
    // ... 其他提取逻辑 ...
    
    // 应用状态机模式
    for matcher in &self.state_machine_matchers {
        let matches = matcher.find_matches(content)?;
        for m in matches {
            let unit = TranslationUnit::new(
                id,
                NodeType::StringLiteral,
                m.content,
                m.start_pos,
                m.end_pos,
            );
            units.push(unit);
        }
    }
}
```

### 2.3 String Extractor 集成

#### 2.3.1 职责

`StringExtractor` 专门负责从代码中提取字符串字面量，与注释提取分离。

#### 2.3.2 分类系统

```rust
pub enum StringCategory {
    ErrorHandling,  // panic!, Error, throw 等
    Output,         // print!, console.log 等
    Variables,      // 变量赋值
    Properties,     // 对象属性
    Other,          // 其他
}
```

#### 2.3.3 与新架构的集成

**配置驱动：**
```rust
pub struct StringExtractorConfig {
    pub enabled_categories: HashSet<StringCategory>,
    pub patterns: CategoryPatterns,
    pub variable_patterns: Vec<String>,
    pub property_patterns: Vec<String>,
    pub custom_regex_patterns: Vec<(String, Regex, usize, StringCategory)>,
}
```

**使用方式：**
1. 从项目配置创建 `StringExtractorConfig`
2. 在语言解析器中根据配置启用/禁用字符串提取
3. 使用 `StringCategory` 对提取的字符串进行分类

---

## 3. 协调器（Coordinator）集成

### 3.1 路由逻辑

```rust
pub fn parse_file(&self, file: &File) -> Result<Vec<TranslationUnit>> {
    let filename = file.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // 1. 优先尝试 tree-sitter 解析器
    for parser in &self.tree_sitter_parsers {
        if parser.supports(filename) {
            return parser.parse(file);
        }
    }

    // 2. 回退到 regex 解析器
    if self.regex_parser.supports(filename) {
        return self.regex_parser.parse(file);
    }

    Err(TranslateError::Parse(format!(...)))
}
```

### 3.2 初始化流程

```
ParserCoordinator::new()
    │
    ├── TreeSitterParserFactory::create_all_parsers()
    │   ├── RustParser::new()
    │   ├── JavaScriptParser::new()
    │   ├── PythonParser::new()
    │   └── ...
    │
    └── presets::create_fallback_parser()
        └── RegexParser::with_config(RegexParserConfig::fallback())
```

---

## 4. 配置系统集成

### 4.1 配置层次

```
ProjectConfig (.translator)
    └── parser.extraction
        ├── comments          # 注释提取配置
        ├── docstrings        # 文档字符串配置
        └── string_literals   # 字符串字面量配置
            ├── categories    # 类别开关
            │   ├── error_handling
            │   ├── output
            │   ├── variables
            │   └── properties
            └── custom_patterns
                ├── regex_patterns      # 简单正则
                └── state_machine_patterns  # 状态机模式
```

### 4.2 配置到解析器的映射

| 配置项 | 影响组件 | 作用 |
|--------|----------|------|
| `categories.error_handling` | StringExtractor | 启用错误处理字符串提取 |
| `categories.output` | StringExtractor | 启用输出/日志字符串提取 |
| `custom_patterns.regex_patterns` | StringExtractor | 添加自定义正则匹配 |
| `custom_patterns.state_machine_patterns` | RegexParser | 添加复杂模式匹配 |

---

## 5. 扩展指南

### 5.1 添加新的语言解析器

1. **创建语言模块**
   ```
   src/parser/languages/newlang/
   ├── mod.rs
   ├── parser.rs
   ├── queries.rs
   └── patterns.rs
   ```

2. **实现解析器结构**
   ```rust
   pub struct NewLangParser {
       config: ParserConfig,
       strategy: Arc<ExtractionStrategyImpl>,
       filter: Arc<ContentFilter>,
       patterns: NewLangPatterns,
       string_processor: StringProcessor,
   }
   ```

3. **注册到工厂**
   在 `TreeSitterParserFactory` 中添加新语言的创建逻辑。

### 5.2 添加新的状态机模式

1. **在配置中定义**
   ```toml
   [[parser.extraction.string_literals.custom_patterns.state_machine_patterns]]
   name = "my_pattern"
   initial_state = "start"
   accepting_states = ["end"]
   ```

2. **定义状态**
   ```toml
   [[parser.extraction.string_literals.custom_patterns.state_machine_patterns.states]]
   name = "start"
   regex = '...'
   transitions = [{ target = "end" }]
   ```

3. **RegexParser 自动加载**
   配置会自动转换为 `StateMachineMatcher` 并集成到解析流程。

---

## 6. 总结

### 6.1 关键设计决策

1. **分层架构**：协调器 → 解析器类型 → 具体实现
2. **策略模式**：提取逻辑与策略/过滤解耦
3. **配置驱动**：所有提取行为可通过配置调整
4. **状态机扩展**：支持复杂模式匹配而不增加代码复杂度

### 6.2 集成检查清单

- [ ] 新解析器实现 `Parser` trait
- [ ] 新解析器注册到 `TreeSitterParserFactory`
- [ ] 字符串提取使用 `StringExtractor` 和分类系统
- [ ] 复杂模式考虑使用 `StateMachineMatcher`
- [ ] 配置项添加到 `ProjectConfig`
- [ ] 更新 `.translator` 配置示例
