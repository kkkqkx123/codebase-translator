# 策略系统与语言解析器集成指南

## 1. 策略系统概述

### 1.1 核心组件

```rust
// 策略节点类型 - 定义可以提取的内容类别
pub enum StrategyNodeType {
    Comment,        // 注释
    DocString,      // 文档字符串
    ErrorMessage,   // 错误消息
    FormatString,   // 格式化字符串
    LogMessage,     // 日志消息
    StringLiteral,  // 字符串字面量
    // ... Markdown 相关类型
}

// 提取上下文 - 提供额外信息用于策略决策
pub struct ExtractionContext {
    pub content: String,           // 内容
    pub function_name: Option<String>,  // 函数名
    pub is_exported: bool,         // 是否导出
    pub metadata: HashMap<String, String>, // 元数据
}

// 策略 trait
pub trait ExtractionStrategy {
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool;
    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType;
}
```

### 1.2 策略实现类型

```rust
pub enum ExtractionStrategyImpl {
    ConfigBased(ConfigBasedStrategy),     // 基于配置
    Combined(CombinedStrategy),           // 组合策略
    ExportedOnly(ExportedOnlyStrategy),   // 仅导出项
}
```

---

## 2. 语言解析器中的策略集成

### 2.1 解析器结构

每个语言解析器都包含策略实例：

```rust
pub struct RustParser {
    config: ParserConfig,
    strategy: Arc<ExtractionStrategyImpl>,  // ← 策略实例
    filter: Arc<ContentFilter>,
    patterns: RustPatterns,
    string_processor: StringProcessor,
}
```

### 2.2 初始化流程

```
ParserCoordinator::new()
    │
    ├── 创建策略
    │   └── default_strategy() 
    │       └── ExtractionStrategyImpl::ConfigBased(...)
    │
    └── 创建解析器
        └── RustParser::new(config, strategy, filter)
            └── 策略被 Arc 克隆共享
```

### 2.3 策略使用模式

#### 模式 1：注释提取（简单过滤）

```rust
fn extract_comments(&self, root_node: &Node, content: &str, file_path: &str) 
    -> Result<Vec<TranslationUnit>> 
{
    for m in matches {
        let text = m.text;
        
        // 1. 基础过滤（长度、符号等）
        if text.len() < self.config.min_content_length { continue; }
        
        // 2. 内容过滤（语言检测）
        if !self.filter.should_translate(text) { continue; }
        
        // 3. 策略决策
        let ctx = ExtractionContext::new(text);
        if !self.strategy.should_extract(StrategyNodeType::Comment, &ctx) {
            continue;  // ← 策略决定不提取
        }
        
        // 4. 获取节点类型映射
        let node_type = self.strategy.get_node_type(StrategyNodeType::Comment);
        
        // 5. 创建翻译单元
        let unit = TranslationUnit::new(id, node_type, text.to_string(), ...);
        units.push(unit);
    }
}
```

#### 模式 2：宏字符串提取（带函数名上下文）

```rust
fn extract_macro_strings(&self, root_node: &Node, content: &str, file_path: &str) 
    -> Result<Vec<TranslationUnit>> 
{
    for m in matches {
        match m.capture_name.as_str() {
            "macro_name" => {
                current_macro = m.text.to_string();
            }
            "macro_string" => {
                // 1. 根据宏名称分类
                let strategy_node_type = match self.patterns.classify_macro(&current_macro) {
                    Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
                    Some(FunctionCategory::Format) => StrategyNodeType::FormatString,
                    Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
                    None => continue,
                };
                
                // 2. 创建带上下文的策略决策
                let ctx = ExtractionContext::new(&text)
                    .with_function_name(&current_macro);  // ← 添加上下文
                
                // 3. 策略决策
                if !self.strategy.should_extract(strategy_node_type, &ctx) {
                    continue;
                }
                
                // 4. 获取映射后的节点类型
                let node_type = self.strategy.get_node_type(strategy_node_type);
                
                let unit = TranslationUnit::new(id, node_type, text, ...);
                units.push(unit);
            }
        }
    }
}
```

---

## 3. 配置与策略的映射

### 3.1 TOML 配置

```toml
[parser.extraction]
# 策略配置
comments = true          # StrategyNodeType::Comment
docstrings = true        # StrategyNodeType::DocString
error_messages = true    # StrategyNodeType::ErrorMessage
format_strings = false   # StrategyNodeType::FormatString
log_messages = true      # StrategyNodeType::LogMessage
string_literals = false  # StrategyNodeType::StringLiteral
```

### 3.2 配置到策略的转换

```rust
// ConfigBasedStrategy::should_extract()
fn should_extract(&self, node_type: StrategyNodeType, _ctx: &ExtractionContext) -> bool {
    match node_type {
        StrategyNodeType::Comment => self.config.comments,
        StrategyNodeType::DocString => self.config.docstrings,
        StrategyNodeType::ErrorMessage => self.config.error_messages,
        StrategyNodeType::FormatString => self.config.format_strings,
        StrategyNodeType::LogMessage => self.config.log_messages,
        StrategyNodeType::StringLiteral => self.config.string_literals,
        // ... Markdown 类型默认 true
    }
}
```

---

## 4. 不同语言解析器的策略集成对比

### 4.1 Rust 解析器

| 提取类型 | StrategyNodeType | 上下文信息 |
|---------|------------------|-----------|
| 注释 | `Comment` | 无 |
| 文档注释 | `DocString` | 无 |
| panic! | `ErrorMessage` | 宏名称 |
| format! | `FormatString` | 宏名称 |
| println! | `LogMessage` | 宏名称 |

```rust
// Rust 特有：宏分类
let strategy_node_type = match self.patterns.classify_macro(&current_macro) {
    Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
    Some(FunctionCategory::Format) => StrategyNodeType::FormatString,
    Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
    // ...
};
```

### 4.2 JavaScript 解析器

| 提取类型 | StrategyNodeType | 上下文信息 |
|---------|------------------|-----------|
| 注释 | `Comment` | 无 |
| JSDoc | `DocString` | 无 |
| throw new Error | `ErrorMessage` | 函数名 |
| console.log | `LogMessage` | 函数名 |
| template literal | `FormatString` | 变量名 |

```rust
// JavaScript 特有：函数调用分类
let strategy_node_type = if self.patterns.is_error_function(&func_name) {
    StrategyNodeType::ErrorMessage
} else if self.patterns.is_log_function(&func_name) {
    StrategyNodeType::LogMessage
} else {
    StrategyNodeType::StringLiteral
};
```

---

## 5. 高级策略组合

### 5.1 组合策略示例

```rust
// 创建组合策略：配置策略 + 仅导出策略
let config_strategy = ExtractionStrategyImpl::from_config(config);
let exported_strategy = ExtractionStrategyImpl::ExportedOnly(
    ExportedOnlyStrategy::new(config_strategy)
);

// 创建解析器
let parser = RustParser::new(
    parser_config,
    Arc::new(exported_strategy),
    filter,
)?;
```

### 5.2 决策流程

```
提取候选
    │
    ▼
┌─────────────────┐
│ 基础过滤        │ ← 长度、符号检查
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌────────┐ ┌──────────┐
│内容过滤 │ │ 策略决策  │
│(Filter)│ │(Strategy)│
└───┬────┘ └────┬─────┘
    │           │
    └─────┬─────┘
          │
    ┌─────┴─────┐
    ▼           ▼
  保留         丢弃
```

---

## 6. 添加新的策略集成

### 6.1 在语言解析器中添加新的提取类型

以添加 "UI 文本" 提取为例：

**步骤 1：添加新的 StrategyNodeType**

```rust
// src/parser/strategy.rs
pub enum StrategyNodeType {
    // ... 现有类型
    UiText,  // ← 新增
}
```

**步骤 2：在配置中添加开关**

```rust
// src/parser/strategy.rs
pub struct ExtractionConfig {
    // ... 现有字段
    #[serde(default = "default_false")]
    pub ui_text: bool,
}
```

**步骤 3：在语言解析器中实现提取**

```rust
// src/parser/languages/rust/parser.rs
fn extract_ui_strings(&self, root_node: &Node, content: &str) -> Result<Vec<TranslationUnit>> {
    // 1. 执行 tree-sitter 查询
    let executor = QueryExecutor::from_string(..., RustQueries::ui_strings())?;
    let matches = executor.execute(root_node, content)?;
    
    for m in matches {
        let text = self.string_processor.clean_string_literal(m.text);
        
        // 2. 应用策略
        let ctx = ExtractionContext::new(&text);
        if !self.strategy.should_extract(StrategyNodeType::UiText, &ctx) {
            continue;
        }
        
        // 3. 创建单元
        let node_type = self.strategy.get_node_type(StrategyNodeType::UiText);
        let unit = TranslationUnit::new(id, node_type, text, ...);
        units.push(unit);
    }
}
```

**步骤 4：更新配置映射**

```rust
// ConfigBasedStrategy::should_extract()
fn should_extract(&self, node_type: StrategyNodeType, _ctx: &ExtractionContext) -> bool {
    match node_type {
        // ... 现有映射
        StrategyNodeType::UiText => self.config.ui_text,
    }
}
```

**步骤 5：更新 TOML 配置**

```toml
[parser.extraction]
ui_text = true  # 新增配置项
```

---

## 7. 最佳实践

### 7.1 策略使用原则

1. **统一入口**：所有提取逻辑都通过 `strategy.should_extract()` 决策
2. **上下文丰富**：尽可能提供 `function_name`、`is_exported` 等上下文
3. **类型准确**：使用最具体的 `StrategyNodeType`，避免滥用 `StringLiteral`
4. **配置驱动**：新增提取类型必须对应配置项

### 7.2 调试策略决策

```rust
// 添加调试日志
let ctx = ExtractionContext::new(&text)
    .with_function_name(&func_name);

let should_extract = self.strategy.should_extract(strategy_node_type, &ctx);
tracing::debug!(
    "Strategy decision: type={:?}, func={}, extract={}",
    strategy_node_type,
    func_name,
    should_extract
);
```

### 7.3 测试策略集成

```rust
#[test]
fn test_error_message_extraction() {
    let config = ExtractionConfig {
        error_messages: true,
        ..Default::default()
    };
    let strategy = Arc::new(ExtractionStrategyImpl::from_config(config));
    
    let parser = RustParser::new(parser_config, strategy, filter).unwrap();
    
    let content = r#"panic!("error message");"#;
    let units = parser.parse(&create_test_file(content, "test.rs")).unwrap();
    
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].node_type, NodeType::ErrorMessage);
}
```
