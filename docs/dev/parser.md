# Parser Module Design

## 概述

Parser 模块负责从源文件中提取可翻译内容，使用字符级扫描技术替代 tree-sitter，支持多种语言和自定义提取模式，提供精确的文本区域定位和翻译应用。

## 设计目的

1. **精确提取**：准确识别注释、文档字符串、字符串字面量等可翻译内容
2. **语言支持**：支持多种编程语言的特定语法结构
3. **自定义模式**：支持正则表达式和状态机自定义提取规则
4. **文本保护**：保护格式字符串和占位符不被破坏

## 核心组件

### 1. TextScanner

**位置**：`src/parser/scanner/character_scanner.rs`

**职责**：
- 字符级文本扫描
- 识别不同类型的文本区域
- 支持语言特定配置

**关键设计**：
```rust
pub struct TextScanner {
    config: ScannerConfig,
    lang_config: ScannerLanguageConfig,
}
```

**扫描流程**：
1. 遍历文件字符
2. 检测注释开始标记
3. 提取注释内容
4. 检测字符串字面量
5. 记录文本区域（起始位置、结束位置、类型）
6. 保护格式字符串中的占位符

**文本区域类型**：
```rust
pub enum TextRegionType {
    LineComment,      // 单行注释
    BlockComment,     // 块注释
    DocString,        // 文档字符串
    StringLiteral,    // 字符串字面量
}
```

### 2. ParserCoordinator

**位置**：`src/parser/coordinator.rs`

**职责**：
- 协调多个解析器
- 应用自定义提取模式
- 合并扫描结果

**关键功能**：
```rust
pub struct ParserCoordinator {
    custom_pattern_matchers: Vec<CustomPatternMatcher>,
    state_machine_matchers: Vec<StateMachineMatcher>,
    filter: Arc<ContentFilter>,
    scanner_config: ScannerConfig,
}

impl ParserCoordinator {
    pub fn parse_file(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        // 1. 使用 TextScanner 扫描文件
        let regions = self.text_scanner.scan(&file.content)?;

        // 2. 应用自定义模式匹配
        let custom_regions = self.apply_custom_patterns(&file)?;

        // 3. 合并所有区域
        let all_regions = self.merge_regions(regions, custom_regions);

        // 4. 转换为翻译单元
        self.regions_to_units(all_regions)
    }
}
```

**设计要点**：
- 优先级：自定义模式 > 内置扫描器
- 支持语言特定的自定义模式
- 内容过滤和验证

### 3. ContentFilter

**位置**：`src/parser/filtering/`

**职责**：
- 过滤提取的文本内容
- 语言检测和验证
- 长度和模式验证

**关键功能**：
```rust
pub trait Filter: Send + Sync {
    fn should_include(&self, content: &str, context: &FilterContext) -> bool;
}

pub struct ContentFilter {
    min_length: usize,
    max_length: usize,
    exclude_patterns: Vec<Regex>,
    include_patterns: Vec<Regex>,
    placeholder_patterns: Vec<Regex>,
    code_patterns: Vec<Regex>,
    language_detector: LanguageDetector,
}
```

**过滤规则**：
1. 长度验证（min_length <= len <= max_length）
2. 排除模式匹配
3. 包含模式验证
4. 占位符检测
5. 代码模式检测
6. 语言验证（包含目标语言字符）

### 4. LanguageDetector

**位置**：`src/parser/filtering/checks/`

**职责**：
- 检测文本语言
- 识别文本脚本（拉丁文、CJK、阿拉伯文等）

**关键类型**：
```rust
pub enum Script {
    Latin,
    CJK,      // 中文、日文、韩文
    Arabic,
    Hebrew,
    Cyrillic,
    Greek,
    Other,
}

pub struct LanguageInfo {
    pub script: Script,
    pub languages: Vec<String>,
    pub confidence: f64,
}
```

**检测策略**：
1. **QuickDetector**：快速检测，基于字符范围
2. **SampledDetector**：采样检测，基于字符频率
3. **LanguageDetector**：完整检测，使用 whatlang 库

### 5. TranslationReplacer

**位置**：`src/parser/scanner/replacer.rs`

**职责**：
- 基于字节偏移应用翻译
- 保护格式和占位符
- 处理多行内容

**关键功能**：
```rust
pub struct TranslationReplacer {
    placeholder_protector: PlaceholderProtector,
}

impl TranslationReplacer {
    pub fn apply_translations(
        &self,
        content: &str,
        translations: &[TranslatedRegion],
    ) -> Result<String> {
        // 1. 保护占位符
        let protected = self.placeholder_protector.protect(content)?;

        // 2. 按字节偏移应用翻译
        let result = self.replace_by_offset(&protected, translations)?;

        // 3. 恢复占位符
        self.placeholder_protector.restore(&result)
    }
}
```

### 6. PlaceholderProtector

**位置**：`src/parser/scanner/placeholder.rs`

**职责**：
- 保护格式字符串中的占位符
- 识别常见的占位符模式

**支持的占位符**：
```rust
// Python
"Hello {}, {}".format(name, age)
f"Hello {name}, you are {age} years old"

// Rust
println!("Hello {}, you are {} years old", name, age);
format!("Value: {}", value);

// JavaScript
`Hello ${name}, you are ${age} years old`
```

**保护策略**：
- 识别占位符模式
- 替换为临时标记
- 翻译后恢复原始占位符

## 技术选型

### 扫描技术
- **字符级扫描**：O(n) 单次遍历
  - 比 tree-sitter 更简单
  - 精确的字节偏移
  - 不需要重构格式

### 语言检测
- **whatlang**：语言检测库
  - 支持多种语言
  - 提供置信度评分
  - 轻量级

### 正则表达式
- **regex**：Rust 正则表达式库
  - 高性能
  - Unicode 支持
  - 自定义模式

### 状态机
- **自定义状态机**：复杂模式匹配
  - 多步骤匹配
  - 状态追踪
  - 灵活配置

## 关键设计要点

### 1. 语言特定配置

```rust
pub struct ScannerLanguageConfig {
    pub line_comment: Vec<String>,      // 单行注释标记
    pub block_comment_start: String,    // 块注释开始
    pub block_comment_end: String,      // 块注释结束
    pub string_delimiters: Vec<String>, // 字符串分隔符
    pub escape_char: Option<char>,      // 转义字符
    pub raw_string_prefixes: Vec<String>, // 原始字符串前缀
}
```

**示例配置**：
```rust
// Rust
ScannerLanguageConfig {
    line_comment: vec!["//".to_string()],
    block_comment_start: "/*".to_string(),
    block_comment_end: "*/".to_string(),
    string_delimiters: vec!["\"".to_string()],
    escape_char: Some('\\'),
    raw_string_prefixes: vec![
        "r".to_string(),
        "r#".to_string(),
        "r##".to_string(),
    ],
}

// Python
ScannerLanguageConfig {
    line_comment: vec!["#".to_string()],
    block_comment_start: '"""'.to_string(),
    block_comment_end: '"""'.to_string(),
    string_delimiters: vec![
        "\"".to_string(),
        "'".to_string(),
    ],
    escape_char: Some('\\'),
    raw_string_prefixes: vec![
        "r".to_string(),
        "R".to_string(),
        "u".to_string(),
    ],
}
```

### 2. 文本区域表示

```rust
pub struct TextRegion {
    pub start_pos: Position,  // 起始位置（字节偏移）
    pub end_pos: Position,    // 结束位置（字节偏移）
    pub region_type: TextRegionType,
    pub content: String,      // 提取的内容
}

pub struct Position {
    pub offset: usize,    // 字节偏移
    pub line: usize,      // 行号（1-based）
    pub column: usize,    // 列号（1-based）
}
```

**关键设计**：
- 使用字节偏移而非字符偏移
- 支持多字节字符（如 UTF-8 中文）
- 保留位置信息用于错误报告

### 3. 自定义提取模式

**正则模式**：
```rust
// 匹配错误消息
CustomPatternMatcher::new(
    "error_message",
    vec![r#"panic!\("([^"]+)"\)"#.to_string()],
    vec!["rust".to_string()],
    PatternType::ErrorMessage,
)
```

**状态机模式**：
```rust
StateMachineMatcher::builder()
    .name("log_message")
    .languages(vec!["rust".to_string()])
    .initial_state("start")
    .transitions(vec![
        Transition::new("start", "open_paren", r#"println!\("#),
        Transition::new("open_paren", "content", r#"([^)]+)"#),
        Transition::new("content", "end", r#"\)"#),
    ])
    .extract_from_state("content", PatternType::LogMessage)
    .build()?
```

### 4. 语言验证

```rust
pub fn contains_target_language(&self, content: &str) -> bool {
    for lang in &self.config.source_langs {
        match lang.as_str() {
            "zh" | "zh-CN" => {
                if self.contains_chinese(content) {
                    return true;
                }
            }
            "ja" => {
                if self.contains_japanese(content) {
                    return true;
                }
            }
            "ko" => {
                if self.contains_korean(content) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}
```

### 5. 内容过滤

```rust
pub fn should_include(&self, content: &str) -> bool {
    // 1. 长度验证
    if content.len() < self.min_length || content.len() > self.max_length {
        return false;
    }

    // 2. 排除模式
    for pattern in &self.exclude_patterns {
        if pattern.is_match(content) {
            return false;
        }
    }

    // 3. 占位符检测
    for pattern in &self.placeholder_patterns {
        if pattern.is_match(content) {
            return false;
        }
    }

    // 4. 代码模式检测
    for pattern in &self.code_patterns {
        if pattern.is_match(content) {
            return false;
        }
    }

    // 5. 语言验证
    if !self.contains_target_language(content) {
        return false;
    }

    true
}
```

## 使用示例

### 基本使用

```rust
use codebase_translate::parser::{ParserCoordinator, ParserConfig};

let coordinator = ParserCoordinator::with_defaults(ParserConfig {
    extract_comments: true,
    extract_docstrings: true,
    extract_strings: false,
    trim_content: true,
    min_content_length: 2,
    max_content_length: 10000,
})?;

let file = File::new(
    PathBuf::from("test.rs"),
    "fn main() {
    // 这是注释
    println!(\"Hello\");
}",
    "utf-8",
);

let units = coordinator.parse_file(&file)?;
```

### 自定义过滤

```rust
use codebase_translate::parser::filtering::{FilterConfig, from_project_config};

let filter_config = FilterConfig {
    min_length: 3,
    max_length: 10000,
    exclude_patterns: vec![
        r#"TODO|FIXME|XXX"#.to_string(),
    ],
    include_patterns: vec![],
    placeholder_patterns: vec![
        r#"\{\{.*\}\}"#.to_string(),
    ],
    code_patterns: vec![
        r#"https?://[^\s]+"#.to_string(),
    ],
};

let filter = from_project_config(&filter_config)?;
```

### 自定义提取模式

```rust
use codebase_translate::parser::regex::CustomPatternMatcher;

let matcher = CustomPatternMatcher::new(
    "error_message",
    vec![r#"error!\("([^"]+)"\)"#.to_string()],
    vec!["rust".to_string()],
    PatternType::ErrorMessage,
);
```

## 性能考量

1. **扫描性能**：
   - 单次遍历 O(n)
   - 字符级操作
   - 最小化正则表达式使用

2. **内存效率**：
   - 流式扫描（待实现）
   - 按需分配
   - 共享字符串（待实现）

3. **并行处理**：
   - 文件级并行
   - 独立扫描器实例
   - 无共享状态

## 扩展性

1. **新的语言支持**：
   - 添加语言配置
   - 定义注释和字符串规则
   - 测试验证

2. **新的提取规则**：
   - 正则表达式模式
   - 状态机模式
   - 插件系统（待实现）

3. **高级过滤**：
   - 语义过滤
   - 上下文感知
   - 机器学习分类

4. **性能优化**：
   - 缓存扫描结果
   - 增量扫描
   - 并行扫描