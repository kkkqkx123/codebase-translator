# 状态机提取规则重构设计文档 v2

## 📋 概述

本次重构的目标是让状态机模式匹配器作为通用逻辑在协调器层面应用，支持所有解析器（tree-sitter 和 regex），并通过配置与文件后缀绑定，减少不必要的解析。

**核心原则**：
- 状态机作为通用逻辑，不局限于 regex_parsers
- 在协调器层面应用状态机，统一管理所有自定义模式
- 通过配置绑定文件后缀，精确控制应用范围
- 职责分离：解析器负责提取，协调器负责补充

## 🎯 设计目标

1. **通用性**：状态机逻辑适用于所有解析器（tree-sitter 和 regex）
2. **统一管理**：在协调器层面统一管理所有自定义模式
3. **配置驱动**：通过配置文件指定模式与文件后缀的绑定
4. **性能优化**：只对需要的文件应用状态机，减少不必要的解析
5. **代码复用**：避免在多个语言解析器中重复实现相同逻辑

## 🏗️ 架构设计

### 当前架构问题

```
┌─────────────────────────────────────────────────────────────┐
│                    当前架构                          │
├─────────────────────────────────────────────────────────────┤
│                                                     │
│  配置文件                                             │
│    ↓                                                 │
│  各语言解析器加载状态机                               │
│    ├─ RustParser::load_state_machine_patterns()          │
│    ├─ GoParser::load_state_machine_patterns()            │
│    ├─ JavaParser::load_state_machine_patterns()          │
│    └─ ... (重复9次)                                │
│    ↓                                                 │
│  各解析器独立应用状态机                                │
│    └─ 代码重复，维护困难                             │
│                                                     │
└─────────────────────────────────────────────────────────────┘

问题：
- 代码重复：每个语言解析器都要实现相同的状态机逻辑
- 维护困难：修改状态机逻辑需要修改9个文件
- 不一致：不同语言可能有不同的实现
- 性能浪费：所有文件都尝试应用状态机
```

### 重构后架构

```
┌─────────────────────────────────────────────────────────────┐
│                    重构后架构                        │
├─────────────────────────────────────────────────────────────┤
│                                                     │
│  配置文件                                             │
│    ↓                                                 │
│  ParserCoordinator 统一加载状态机                        │
│    ├─ 解析配置，建立文件后缀与模式的映射              │
│    └─ 创建 StateMachineMatcher 实例                   │
│    ↓                                                 │
│  解析器解析文件（tree-sitter 或 regex）               │
│    ↓                                                 │
│  返回 TranslationUnit 列表                              │
│    ↓                                                 │
│  Coordinator 应用状态机（仅对匹配的文件后缀）          │
│    ├─ 查找适用于当前文件的模式                        │
│    ├─ 应用状态机匹配                                    │
│    └─ 合并结果                                        │
│    ↓                                                 │
│  Writer 应用翻译                                       │
│                                                     │
└─────────────────────────────────────────────────────────────┘

优势：
- 代码复用：状态机逻辑只实现一次
- 统一管理：所有状态机模式在协调器中管理
- 灵活配置：通过配置文件指定哪些文件使用哪些模式
- 性能优化：只对需要的文件应用状态机
- 易于维护：修改状态机逻辑只需修改一个地方
```

## 📊 数据流设计

### 示例场景：自定义 add_message 方法

```javascript
// 原始代码
add_message("ERR001", "Invalid input parameter", { severity: "high" });
```

### 步骤1：配置加载

```toml
# config.toml

[[extraction.state_machine_patterns]]
name = "add_message"
file_extensions = ["js", "ts", "jsx", "tsx"]  # 绑定文件后缀
category = "error_handling"

# 提取规则
extraction_rule = { type = "remove_quotes" }

initial_state = "start"
accepting_states = ["extract"]

[[extraction.state_machine_patterns.states]]
name = "start"
regex = r#"add_message\s*\(\s*["'][^"']+["']\s*,\s*["']"#
capture_group = 0
is_final = false

[[extraction.state_machine_patterns.states.transitions]]
target = "extract"

[[extraction.state_machine_patterns.states]]
name = "extract"
regex = r#"([^"']+)"#
capture_group = 1
is_final = true
```

### 步骤2：Coordinator 加载

```rust
// src/parser/coordinator/coordinator.rs

pub struct ParserCoordinator {
    tree_sitter_parsers: HashMap<String, Box<dyn Parser>>,
    regex_parser: Option<RegexParser>,
    
    // 新增：状态机匹配器
    state_machine_matchers: Vec<StateMachineMatcher>,
    
    // 新增：文件后缀到状态机的映射
    extension_to_matchers: HashMap<String, Vec<usize>>,
}

impl ParserCoordinator {
    pub fn with_defaults(config: ParserConfig) -> Result<Self> {
        // ... 加载 tree-sitter 和 regex 解析器 ...
        
        // 加载状态机模式
        let state_machine_patterns = Self::load_state_machine_patterns(&config)?;
        
        // 创建状态机匹配器
        let state_machine_matchers: Vec<_> = state_machine_patterns
            .iter()
            .filter_map(|pattern| {
                StateMachineMatcher::from_config(
                    pattern.name.clone(),
                    pattern.initial_state.clone(),
                    pattern.accepting_states.clone(),
                    &pattern.states,
                    pattern.extraction_rule.clone(),
                ).ok()
            })
            .collect();
        
        // 建立文件后缀到状态机的映射
        let mut extension_to_matchers = HashMap::new();
        for (idx, pattern) in state_machine_patterns.iter().enumerate() {
            for ext in &pattern.file_extensions {
                extension_to_matchers
                    .entry(ext.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }
        
        Ok(Self {
            tree_sitter_parsers,
            regex_parser,
            state_machine_matchers,
            extension_to_matchers,
        })
    }
}
```

### 步骤3：解析器解析

```rust
// 使用 tree-sitter 或 regex 解析器解析文件
let units = parser.parse(&file)?;

// 此时 units 包含：
// - tree-sitter 提取的注释、字符串等
// - 但不包含自定义的 add_message 调用
```

### 步骤4：Coordinator 应用状态机

```rust
// src/parser/coordinator/coordinator.rs

impl ParserCoordinator {
    pub fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        // 1. 使用适当的解析器解析文件
        let mut units = self.parse_with_parser(file)?;
        
        // 2. 应用状态机模式（仅对匹配的文件后缀）
        let file_ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        if let Some(matcher_indices) = self.extension_to_matchers.get(&file_ext) {
            let content = file.content_string()?;
            
            for &idx in matcher_indices {
                let matcher = &self.state_machine_matchers[idx];
                let matches = matcher.find_matches(&content)?;
                
                for m in matches {
                    // 使用提取的文本
                    let text = &m.extracted_text;
                    
                    if should_include(
                        text,
                        self.config.min_content_length,
                        self.config.max_content_length,
                    ) {
                        let id = format!("{}_sm_{}_{}", 
                            file.path.display(), 
                            matcher.name, 
                            units.len()
                        );
                        
                        let unit = TranslationUnit::new(
                            id,
                            NodeType::StringLiteral,
                            text.clone(),
                            m.start_pos,
                            m.end_pos,
                        );
                        units.push(unit);
                    }
                }
            }
        }
        
        // 3. 按位置排序
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));
        
        Ok(units)
    }
}
```

### 步骤5：翻译和写入

```rust
// 翻译
for unit in &mut units {
    unit.set_translated(translate(&unit.content)?);
}

// Writer 使用 start_pos/end_pos 定位原始内容
let result = TranslationApplier::apply_translations(content, &units)?;

// 结果：
add_message("ERR001", "无效的输入参数", { severity: "high" });
```

## 🔧 需要修改的文件

### 1. 配置层

#### 文件：`src/config/project.rs`

**修改 ExtractionConfig**：
```rust
/// Extraction settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Extract comments
    #[serde(default)]
    pub comments: bool,
    
    /// Extract docstrings
    #[serde(default)]
    pub doc_strings: bool,
    
    /// Extract error messages
    #[serde(default)]
    pub error_messages: bool,
    
    /// Extract format strings
    #[serde(default)]
    pub format_strings: bool,
    
    /// Custom regex patterns
    #[serde(default)]
    pub regex_patterns: Vec<CustomRegexPattern>,
    
    /// Advanced state machine patterns
    #[serde(default)]
    pub state_machine_patterns: Vec<StateMachinePattern>,  // 移到顶层
}
```

**修改 StateMachinePattern**：
```rust
/// State machine pattern for complex extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachinePattern {
    /// Pattern name for identification
    pub name: String,
    
    /// File extensions this pattern applies to
    /// Empty means applies to all files
    #[serde(default)]
    pub file_extensions: Vec<String>,
    
    /// Category this pattern belongs to
    #[serde(default)]
    pub category: StringLiteralCategory,
    
    /// Extraction rule
    #[serde(default)]
    pub extraction_rule: ExtractionRule,
    
    /// States and their transitions
    pub states: Vec<PatternState>,
    
    /// Initial state name
    pub initial_state: String,
    
    /// Accepting state names
    pub accepting_states: Vec<String>,
}
```

**移除 CustomStringPatterns**：
```rust
// 删除这个结构，将 state_machine_patterns 移到 ExtractionConfig 顶层
```

### 2. 状态机层

#### 文件：`src/parser/regex/state_machine.rs`

**修改 StateMachineMatch**：
```rust
/// A match result from state machine
#[derive(Debug, Clone)]
pub struct StateMachineMatch {
    /// Complete matched content (including format markers)
    pub raw_content: String,
    
    /// Extracted text content (for translation)
    pub extracted_text: String,
    
    /// Start position in source
    pub start_pos: Position,
    
    /// End position in source
    pub end_pos: Position,
    
    /// Name of accepting state
    pub state_name: String,
    
    /// Capture groups (if any)
    pub captures: Vec<String>,
}
```

**修改 StateMachineMatcher**：
```rust
pub struct StateMachineMatcher {
    states: HashMap<String, State>,
    initial_state: String,
    accepting_states: Vec<String>,
    pub name: String,
    extraction_rule: ExtractionRule,
}

impl StateMachineMatcher {
    pub fn from_config(
        name: String,
        initial_state: String,
        accepting_states: Vec<String>,
        state_configs: &[crate::config::project::PatternState],
        extraction_rule: ExtractionRule,
    ) -> Result<Self> {
        // ... 现有代码 ...
        
        Ok(Self {
            states,
            initial_state,
            accepting_states,
            name,
            extraction_rule,
        })
    }
    
    /// Extract text from complete content
    fn extract_text(&self, raw_content: &str) -> String {
        use crate::parser::core::{StringProcessor, CommentType};
        
        match &self.extraction_rule {
            ExtractionRule::None => raw_content.to_string(),
            
            ExtractionRule::RemoveQuotes => {
                let text = raw_content.trim();
                if (text.starts_with('"') && text.ends_with('"')) ||
                   (text.starts_with('\'') && text.ends_with('\'')) {
                    text[1..text.len()-1].to_string()
                } else {
                    text.to_string()
                }
            }
            
            ExtractionRule::Regex { pattern, group } => {
                if let Ok(re) = Regex::new(pattern) {
                    if let Some(caps) = re.captures(raw_content) {
                        if *group > 0 && caps.len() > *group {
                            caps[*group].as_str().to_string()
                        } else if *group == 0 && !caps.is_empty() {
                            caps[0].as_str().to_string()
                        } else {
                            raw_content.to_string()
                        }
                    } else {
                        raw_content.to_string()
                    }
                } else {
                    raw_content.to_string()
                }
            }
            
            ExtractionRule::RemoveCommentMarkers { comment_type } => {
                let processor = StringProcessor::new();
                match comment_type.as_str() {
                    "line" => processor.clean_comment(raw_content, CommentType::Line),
                    "block" => processor.clean_comment(raw_content, CommentType::Block),
                    "doc" => processor.clean_comment(raw_content, CommentType::Doc),
                    _ => raw_content.to_string(),
                }
            }
            
            ExtractionRule::RemoveBrackets { bracket_type } => {
                let text = raw_content.trim();
                let (open, close) = match bracket_type.as_str() {
                    "round" => ('(', ')'),
                    "square" => ('[', ']'),
                    "curly" => ('{', '}'),
                    _ => return raw_content.to_string(),
                };
                
                if text.starts_with(open) && text.ends_with(close) {
                    text[1..text.len()-1].to_string()
                } else {
                    text.to_string()
                }
            }
        }
    }
    
    /// Modify try_match to return StateMachineMatch with extracted_text
    fn try_match(&self, content: &str, start_pos: usize) -> Result<Option<StateMachineMatch>> {
        // ... 现有匹配逻辑 ...
        
        if let Some((extracted_content, end_offset, captures)) = last_accepting_match {
            let raw_content = extracted_content;
            let extracted_text = self.extract_text(&raw_content);
            
            let start_pos_obj = self.byte_to_position(content, start_pos);
            let end_pos_obj = self.byte_to_position(content, end_offset);

            Ok(Some(StateMachineMatch {
                raw_content,
                extracted_text,
                start_pos: start_pos_obj,
                end_pos: end_pos_obj,
                state_name: current_state_name,
                captures,
            }))
        } else {
            Ok(None)
        }
    }
}
```

### 3. 协调器层

#### 文件：`src/parser/coordinator/coordinator.rs`

**修改 ParserCoordinator**：
```rust
pub struct ParserCoordinator {
    /// Tree-sitter parsers indexed by extension
    tree_sitter_parsers: HashMap<String, Box<dyn Parser>>,
    
    /// Regex parser for fallback
    regex_parser: Option<RegexParser>,
    
    /// State machine matchers
    state_machine_matchers: Vec<StateMachineMatcher>,
    
    /// Map from file extension to state machine matcher indices
    extension_to_matchers: HashMap<String, Vec<usize>>,
    
    /// Parser configuration
    config: ParserConfig,
}

impl ParserCoordinator {
    pub fn with_defaults(config: ParserConfig) -> Result<Self> {
        // ... 加载 tree-sitter 和 regex 解析器 ...
        
        // 加载状态机模式
        let state_machine_patterns = Self::load_state_machine_patterns(&config)?;
        
        // 创建状态机匹配器
        let state_machine_matchers: Vec<_> = state_machine_patterns
            .iter()
            .filter_map(|pattern| {
                StateMachineMatcher::from_config(
                    pattern.name.clone(),
                    pattern.initial_state.clone(),
                    pattern.accepting_states.clone(),
                    &pattern.states,
                    pattern.extraction_rule.clone(),
                ).ok()
            })
            .collect();
        
        // 建立文件后缀到状态机的映射
        let mut extension_to_matchers = HashMap::new();
        for (idx, pattern) in state_machine_patterns.iter().enumerate() {
            let extensions = if pattern.file_extensions.is_empty() {
                // 空列表表示适用于所有文件
                vec!["*".to_string()]
            } else {
                pattern.file_extensions.clone()
            };
            
            for ext in extensions {
                extension_to_matchers
                    .entry(ext.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }
        
        Ok(Self {
            tree_sitter_parsers,
            regex_parser,
            state_machine_matchers,
            extension_to_matchers,
            config,
        })
    }
    
    /// Load state machine patterns from configuration
    fn load_state_machine_patterns(config: &ParserConfig) -> Result<Vec<StateMachinePattern>> {
        // 从配置中加载状态机模式
        // 可以从配置文件或环境变量加载
        Ok(Vec::new())
    }
    
    /// Parse file with appropriate parser and apply state machines
    pub fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        // 1. 使用适当的解析器解析文件
        let mut units = self.parse_with_parser(file)?;
        
        // 2. 应用状态机模式（仅对匹配的文件后缀）
        let file_ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // 检查是否有适用的状态机
        if let Some(matcher_indices) = self.extension_to_matchers.get(&file_ext) {
            let content = file.content_string()?;
            
            for &idx in matcher_indices {
                let matcher = &self.state_machine_matchers[idx];
                
                debug!(
                    matcher_name = %matcher.name,
                    file_extension = %file_ext,
                    "Applying state machine pattern"
                );
                
                let matches = matcher.find_matches(&content)?;
                
                for m in matches {
                    // 使用提取的文本
                    let text = &m.extracted_text;
                    
                    if should_include(
                        text,
                        self.config.min_content_length,
                        self.config.max_content_length,
                    ) {
                        let id = format!("{}_sm_{}_{}", 
                            file.path.display(), 
                            matcher.name, 
                            units.len()
                        );
                        
                        let unit = TranslationUnit::new(
                            id,
                            NodeType::StringLiteral,
                            text.clone(),
                            m.start_pos,
                            m.end_pos,
                        );
                        units.push(unit);
                    }
                }
            }
            
            debug!(
                matcher_count = matcher_indices.len(),
                units_added = units.len() - initial_count,
                "State machine patterns applied"
            );
        }
        
        // 3. 按位置排序
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));
        
        Ok(units)
    }
}
```

### 4. Regex Parser 层

#### 文件：`src/parser/regex/parser.rs`

**移除状态机支持**：
```rust
// 删除 state_machine_matchers 字段和相关逻辑
// 状态机现在由协调器统一管理

pub struct RegexParser {
    config: ParserConfig,
    regex_config: RegexParserConfig,
    line_comment_regex: Option<Regex>,
    block_comment_regex: Option<Regex>,
    doc_comment_regex: Option<Regex>,
    string_regex: Option<Regex>,
    string_processor: StringProcessor,
    // 移除：state_machine_matchers: Vec<StateMachineMatcher>,
}
```

### 5. Tree-sitter Parser 层

#### 文件：`src/parser/languages/{lang}/parser.rs`

**无需修改**：
- 所有语言解析器保持不变
- 状态机逻辑由协调器统一处理
- 避免代码重复

## 📝 配置示例

### 示例1：JavaScript/TypeScript 文件的自定义方法

```toml
# config.toml

[[extraction.state_machine_patterns]]
name = "add_message_js"
file_extensions = ["js", "ts", "jsx", "tsx"]
category = "error_handling"

# 提取规则：移除引号
extraction_rule = { type = "remove_quotes" }

initial_state = "start"
accepting_states = ["extract"]

[[extraction.state_machine_patterns.states]]
name = "start"
regex = r#"add_message\s*\(\s*["'][^"']+["']\s*,\s*["']"#
capture_group = 0
is_final = false

[[extraction.state_machine_patterns.states.transitions]]
target = "extract"

[[extraction.state_machine_patterns.states]]
name = "extract"
regex = r#"([^"']+)"#
capture_group = 1
is_final = true
```

**效果**：
```javascript
// 原始代码
add_message("ERR001", "Invalid input parameter", { severity: "high" });

// 解析结果
units = [
    TranslationUnit {
        content: "Invalid input parameter",  // 提取的文本
        start_pos: Position { line: 1, column: 20, offset: 19 },
        end_pos: Position { line: 1, column: 44, offset: 43 },
        ...
    }
]

// 翻译后
add_message("ERR001", "无效的输入参数", { severity: "high" });
```

### 示例2：所有文件的日志格式

```toml
[[extraction.state_machine_patterns]]
name = "custom_log_format"
file_extensions = []  # 空列表表示适用于所有文件
category = "output"

# 提取规则：使用正则提取
extraction_rule = { 
    type = "regex", 
    pattern = r"\[ERROR\]\s*(.+)", 
    group = 1 
}

initial_state = "start"
accepting_states = ["match"]

[[extraction.state_machine_patterns.states]]
name = "start"
regex = r#"\[ERROR\]\s*"#
is_final = false

[[extraction.state_machine_patterns.states.transitions]]
target = "match"

[[extraction.state_machine_patterns.states]]
name = "match"
regex = r#"(.+)"#
capture_group = 1
is_final = true
```

**效果**：
```javascript
// 原始代码
[ERROR] Invalid input

// 解析结果（适用于所有文件）
units = [
    TranslationUnit {
        content: "Invalid input",
        start_pos: Position { line: 1, column: 1, offset: 0 },
        end_pos: Position { line: 1, column: 22, offset: 21 },
        ...
    }
]

// 翻译后
[ERROR] 无效的输入
```

### 示例3：Python 文件的自定义装饰器

```toml
[[extraction.state_machine_patterns]]
name = "python_decorator"
file_extensions = ["py"]
category = "error_handling"

# 提取规则：移除括号
extraction_rule = { 
    type = "remove_brackets", 
    bracket_type = "round" 
}

initial_state = "start"
accepting_states = ["extract"]

[[extraction.state_machine_patterns.states]]
name = "start"
regex = r#"@custom_error\s*\(\s*["']"#
capture_group = 0
is_final = false

[[extraction.state_machine_patterns.states.transitions]]
target = "extract"

[[extraction.state_machine.states]]
name = "extract"
regex = r#"([^"']+)"#
capture_group = 1
is_final = true
```

**效果**：
```python
# 原始代码
@custom_error("This is an error message")

# 解析结果
units = [
    TranslationUnit {
        content: "This is an error message",
        start_pos: Position { line: 1, column: 1, offset: 0 },
        end_pos: Position { line: 1, column: 38, offset: 37 },
        ...
    }
]

# 翻译后
@custom_error("这是一个错误消息")
```

## 🧪 测试策略

### 单元测试

```rust
// tests/state_machine_tests.rs

#[test]
fn test_extraction_rule_remove_quotes() {
    let matcher = StateMachineBuilder::new()
        .name("test")
        .initial_state("start")
        .accepting_state("end")
        .extraction_rule(ExtractionRule::RemoveQuotes)
        .state("start", r#""([^"]+)""#, 1, true)
        .build()
        .expect("Failed to build");

    let content = r#""Hello World""#;
    let matches = matcher.find_matches(content).expect("Matching failed");

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].raw_content, "\"Hello World\"");
    assert_eq!(matches[0].extracted_text, "Hello World");
}

#[test]
fn test_extraction_rule_regex() {
    let matcher = StateMachineBuilder::new()
        .name("test")
        .initial_state("start")
        .accepting_state("end")
        .extraction_rule(ExtractionRule::Regex {
            pattern: r"\[ERROR\]\s*(.+)".to_string(),
            group: 1,
        })
        .state("start", r#"\[ERROR\]\s*"#, 0, false)
        .transition("end")
        .state("end", r#"(.+)"#, 1, true)
        .build()
        .expect("Failed to build");

    let content = "[ERROR] Something went wrong";
    let matches = matcher.find_matches(content).expect("Matching failed");

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].raw_content, "[ERROR] Something went wrong");
    assert_eq!(matches[0].extracted_text, "Something went wrong");
}
```

### 集成测试

```rust
// tests/coordinator_state_machine_tests.rs

#[test]
fn test_coordinator_applies_state_machine_to_matching_extension() {
    let config = ParserConfig::default();
    let coordinator = ParserCoordinator::with_defaults(config)
        .expect("Failed to create coordinator");

    let content = r#"
add_message("ERR001", "Invalid input parameter", { severity: "high" });
"#;

    let file = create_test_file(content, "test.js");
    let units = coordinator.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());
    
    let unit = &units[0];
    assert_eq!(unit.content, "Invalid input parameter");
    assert!(!unit.content.contains('"'), "Should not contain quotes");
}

#[test]
fn test_coordinator_skips_state_machine_for_non_matching_extension() {
    let config = ParserConfig::default();
    let coordinator = ParserCoordinator::with_defaults(config)
        .expect("Failed to create coordinator");

    let content = r#"
add_message("ERR001", "Invalid input parameter", { severity: "high" });
"#;

    // 使用不匹配的扩展名
    let file = create_test_file(content, "test.rs");
    let units = coordinator.parse(&file).expect("Parsing should succeed");

    // 状态机不应该被应用
    assert!(units.is_empty());
}

#[test]
fn test_coordinator_applies_state_machine_to_all_files() {
    let config = ParserConfig::default();
    let coordinator = ParserCoordinator::with_defaults(config)
        .expect("Failed to create coordinator");

    let content = r#"
[ERROR] Something went wrong
"#;

    // 状态机配置为适用于所有文件
    let file = create_test_file(content, "test.rs");
    let units = coordinator.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());
    
    let unit = &units[0];
    assert_eq!(unit.content, "Something went wrong");
}
```

## 📊 影响范围

### 修改的文件

1. **配置层**（1个文件）
   - `src/config/project.rs` - 修改 `ExtractionConfig`，添加 `file_extensions` 字段

2. **状态机层**（1个文件）
   - `src/parser/regex/state_machine.rs` - 修改 `StateMachineMatch` 和 `StateMachineMatcher`

3. **协调器层**（1个文件）
   - `src/parser/coordinator/coordinator.rs` - 添加状态机管理逻辑

4. **Regex Parser 层**（1个文件）
   - `src/parser/regex/parser.rs` - 移除状态机支持

5. **测试文件**（新增）
   - `tests/state_machine_tests.rs` - 单元测试
   - `tests/coordinator_state_machine_tests.rs` - 集成测试

### 不需要修改的文件

- **Tree-sitter Parser 层**（9个文件）- 无需修改
  - `src/parser/languages/rust/parser.rs`
  - `src/parser/languages/go/parser.rs`
  - `src/parser/languages/java/parser.rs`
  - `src/parser/languages/javascript/parser.rs`
  - `src/parser/languages/typescript/parser.rs`
  - `src/parser/languages/python/parser.rs`
  - `src/parser/languages/c/parser.rs`
  - `src/parser/languages/cpp/parser.rs`
  - `src/parser/languages/csharp/parser.rs`

### 兼容性

- **向后兼容**：如果不配置 `file_extensions`，默认为空列表，不应用任何状态机
- **配置迁移**：现有配置无需修改，新配置可选添加
- **API 兼容**：不改变现有的公共接口

## ⚠️ 风险和注意事项

### 风险

1. **配置复杂性**
   - 用户需要理解 `file_extensions` 的含义
   - 缓解措施：提供详细的配置示例和文档

2. **性能影响**
   - 协调器需要额外的状态机匹配步骤
   - 缓解措施：通过 `file_extensions` 精确控制应用范围

3. **测试覆盖**
   - 需要为各种文件后缀组合添加测试
   - 缓解措施：优先覆盖常用场景

### 注意事项

1. **文件后缀匹配**
   - 使用不区分大小写的匹配
   - 支持通配符 `*` 表示所有文件

2. **状态机优先级**
   - 多个状态机可能匹配同一个文件
   - 按配置顺序应用，后应用的优先级高

3. **结果合并**
   - 需要正确合并解析器和状态机的结果
   - 按位置排序，避免顺序混乱

## 🚀 实施计划

### 阶段1：核心功能（高优先级）

1. 添加 `ExtractionRule` 枚举
2. 修改 `ExtractionConfig`，将 `state_machine_patterns` 移到顶层
3. 修改 `StateMachinePattern` 添加 `file_extensions` 字段
4. 修改 `StateMachineMatch` 添加 `extracted_text` 字段
5. 在 `StateMachineMatcher` 中实现 `extract_text()` 方法
6. 修改 `try_match()` 返回 `extracted_text`

### 阶段2：协调器集成（中优先级）

7. 在 `ParserCoordinator` 中添加状态机管理
8. 实现 `load_state_machine_patterns()` 方法
9. 实现 `extension_to_matchers` 映射
10. 修改 `parse()` 方法应用状态机
11. 从 `RegexParser` 移除状态机支持

### 阶段3：测试和文档（低优先级）

12. 添加单元测试
13. 添加集成测试
14. 完善配置示例
15. 更新文档

## 📚 文档更新

需要更新的文档：

1. **配置文档**：说明 `state_machine_patterns` 的使用方法
2. **架构文档**：更新协调器的工作流程
3. **示例文档**：提供各种场景的配置示例
4. **迁移指南**：帮助用户从旧配置迁移到新配置

## ✅ 验收标准

1. ✅ 所有单元测试通过
2. ✅ 所有集成测试通过
3. ✅ 配置示例正确工作
4. ✅ 文件后缀绑定正确
5. ✅ 文档完整且准确
6. ✅ 代码通过 clippy 检查
7. ✅ 代码格式符合项目规范
8. ✅ 性能无明显下降

## 📞 反馈请求

请审阅本设计文档，并提供以下反馈：

1. **架构合理性**：在协调器层面应用状态机是否合理？是否有更好的设计方案？
2. **文件后缀绑定**：`file_extensions` 的设计是否合理？是否需要其他控制方式？
3. **性能考虑**：这种设计对性能的影响如何？如何优化？
4. **配置灵活性**：配置是否足够灵活？是否需要添加其他选项？
5. **实施优先级**：阶段的划分是否合理？是否需要调整？
6. **其他建议**：是否有其他需要考虑的方面？
