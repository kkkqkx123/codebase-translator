# Regex匹配与FormatInfo适用性分析

## 概述

本文档分析了项目中基于正则表达式的解析逻辑，评估`format_info`在正则匹配场景下的适用性，并提出改进方案。

---

## 一、Tree-sitter与正则匹配的本质差异

### 1.1 Tree-sitter匹配：基于语法结构

**特点**：
- ✅ 边界固定：注释标记、字符串引号等语法符号
- ✅ 结构清晰：AST节点类型明确
- ✅ 格式可预测：注释风格、字符串样式都是标准化的

**示例**：
```rust
// Tree-sitter提取注释
原始代码: "    // This is a comment"
提取结果:
  - base_indent: "    "
  - line_prefix: "// "
  - content: "This is a comment"
  - format_info: 完整且准确
```

### 1.2 正则匹配：基于字符序列

**特点**：
- ❌ 边界任意：可以是任何字符序列
- ❌ 结构模糊：没有语法概念
- ❌ 格式不可预测：完全取决于正则表达式

**示例**（来自配置文件）：
```toml
# 自定义正则模式
[[extraction.custom_patterns]]
name = "todo_pattern"
regex = 'TODO:\s*(.+)'
group = 1

# 匹配结果
原始代码: "TODO: Fix this bug"
提取结果:
  - raw_content: "TODO: Fix this bug"
  - extracted_text: "Fix this bug"
  - 问题: "TODO:" 不是格式标记，而是内容的一部分
```

### 1.3 核心差异对比

| 维度 | Tree-sitter | 正则匹配 |
|-----|------------|----------|
| 边界类型 | 语法符号（`//`、`/*`、`"`） | 任意字符序列 |
| 结构性 | 强（AST） | 弱（文本） |
| 格式可预测性 | 高 | 低 |
| format_info适用性 | ✅ 完美 | ❌ 不适用 |

---

## 二、format_info在正则场景下的局限性

### 2.1 format_info的设计假设

```rust
pub struct FormatInfo {
    pub style: CommentStyle,           // 注释风格（Line/Block等）
    pub base_indent: String,           // 基础缩进
    pub line_prefix: Option<String>,    // 行前缀（如" * "）
    pub string_style: Option<StringStyle>, // 字符串样式
    pub placeholders: Option<Vec<FormatPlaceholder>>, // 占位符
    pub quote_char: Option<char>,      // 引号字符
    pub is_multiline: bool,            // 是否多行
}
```

**设计假设**：
1. 有明确的"格式标记"（如`//`、`/*`、`"`）
2. 格式标记与内容是分离的
3. 可以通过移除格式标记来提取内容

### 2.2 正则匹配的现实问题

#### 案例1：TODO模式
```javascript
// 原始代码
TODO: Fix this bug
console.log("TODO: Add tests")

// 正则匹配
regex = 'TODO:\s*(.+)'
group = 1

// 问题：
// - "TODO:" 是内容的一部分，不是格式标记
// - 无法用format_info描述这种"部分提取"
// - 如果使用format_info重构，会丢失"TODO:"前缀
```

#### 案例2：错误消息
```javascript
// 原始代码
throw new Error("Invalid input")

// 正则匹配
regex = 'throw new Error\("([^"]+)"\)'
group = 1

// 问题：
// - 匹配范围跨越多个语法结构
// - "throw new Error(" 是代码的一部分，不是格式
// - 无法用format_info描述这种"跨语法提取"
```

#### 案例3：日志消息
```python
# 原始代码
logger.info("User logged in")

# 正则匹配
regex = 'logger\.(info|debug|warn|error)\("([^"]+)"\)'
group = 2

# 问题：
// - 匹配包含函数调用语法
// - 无法用format_info描述这种"函数调用内的提取"
```

### 2.3 核心矛盾

| 维度 | Tree-sitter | 正则匹配 |
|-----|------------|----------|
| 边界类型 | 语法符号 | 任意字符序列 |
| 结构性 | 强（AST） | 弱（文本） |
| 格式可预测性 | 高 | 低 |
| format_info适用性 | ✅ 完美 | ❌ 不适用 |
| 提取方式 | 完整节点 | 部分捕获组 |
| 重构需求 | 保留格式 | 直接替换 |

---

## 三、语言解析器中的正则使用分析

### 3.1 当前实现概览

经过分析，项目中的语言解析器（Rust、Python、JavaScript等）主要使用Tree-sitter进行AST解析，但在以下场景中使用了正则表达式：

#### 3.1.1 StringProcessor中的占位符提取

**位置**：`src/parser/core/string_processor.rs`

**用途**：提取字符串中的格式占位符

```rust
// Python f-string: {name}
if let Ok(re) = regex::Regex::new(r"\{([^{}]+)\}") {
    for cap in re.captures_iter(text) {
        placeholders.push(FormatPlaceholder::FString(cap[1].to_string()));
    }
}

// JS template: ${name}
if let Ok(re) = regex::Regex::new(r"\$\{([^{}]+)\}") {
    for cap in re.captures_iter(text) {
        placeholders.push(FormatPlaceholder::JSTemplate(cap[1].to_string()));
    }
}

// C-style: %s, %d, %f
if let Ok(re) = regex::Regex::new(r"%[sdifoxXeEcgG%]") {
    for mat in re.find_iter(text) {
        placeholders.push(FormatPlaceholder::CStyle(mat.as_str().to_string()));
    }
}

// Rust-style: {}
if let Ok(re) = regex::Regex::new(r"\{\}") {
    for mat in re.find_iter(text) {
        placeholders.push(FormatPlaceholder::RustStyle(mat.as_str().to_string()));
    }
}
```

**分析**：
- ✅ 这些正则用于提取占位符信息，不是用于提取翻译内容
- ✅ 提取的占位符保存在`format_info.placeholders`中
- ✅ Writer阶段使用这些占位符信息来重构字符串
- ❌ **不需要改造**：这是正确的使用方式

#### 3.1.2 语言解析器中的clean_*_text方法

**位置**：各语言解析器（Rust、Python、JavaScript等）

**用途**：清理注释/文档字符串标记

##### Rust解析器
```rust
fn clean_comment_text(&self, text: &str) -> String {
    let trimmed = text.trim();

    // Handle outer doc comments: ///
    if trimmed.starts_with("///") {
        return self.string_processor.clean_comment(trimmed, CommentType::Doc);
    }

    // Handle inner doc comments: //!
    if trimmed.starts_with("//!") {
        return self.string_processor.clean_comment(trimmed, CommentType::Doc);
    }

    // Handle block doc comments: /**
    if trimmed.starts_with("/**") {
        return self.string_processor.clean_comment(trimmed, CommentType::Doc);
    }

    // Handle regular line comments: //
    if trimmed.starts_with("//") {
        return self.string_processor.clean_comment(trimmed, CommentType::Line);
    }

    // Handle block comments: /* */
    if trimmed.starts_with("/*") {
        return self.string_processor.clean_comment(trimmed, CommentType::Block);
    }

    trimmed.to_string()
}
```

**分析**：
- ✅ 使用字符串前缀匹配（`starts_with`），不是正则
- ✅ 调用`StringProcessor::clean_comment`，返回完整的`CleanedComment`（包含format_info）
- ✅ **不需要改造**：这是正确的使用方式

##### Python解析器
```rust
fn clean_comment_text(&self, text: &str) -> String {
    self.string_processor.clean_comment(text, CommentType::Line)
}

fn clean_docstring_text(&self, text: &str) -> String {
    let trimmed = text
        .trim_start()
        .trim_end_matches(|c: char| c.is_whitespace() && c != '\n');

    let content = if trimmed.starts_with("\"\"\"") && trimmed.ends_with("\"\"\"") {
        &trimmed[3..trimmed.len() - 3]
    } else if trimmed.starts_with("'''") && trimmed.ends_with("'''") {
        &trimmed[3..trimmed.len() - 3]
    } else if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 1 {
        return trimmed[1..trimmed.len() - 1].to_string();
    } else if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() > 1 {
        return trimmed[1..trimmed.len() - 1].to_string();
    } else {
        return trimmed.to_string();
    };

    // Process lines to remove common leading indentation
    let lines: Vec<&str> = content.lines().collect();

    // Find minimum indentation
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    // Remove common indentation from each line
    let processed_lines: Vec<String> = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                line.chars().skip(min_indent).collect()
            }
        })
        .collect();

    processed_lines.join("\n").trim_end().to_string()
}
```

**分析**：
- ⚠️ 使用字符串前缀匹配（`starts_with`），不是正则
- ⚠️ **问题**：只返回清理后的文本，**没有返回format_info**
- ⚠️ **需要改造**：应该使用`clean_comment_with_format`或`clean_string_literal_with_format`

##### JavaScript解析器
```rust
fn clean_comment_text(&self, text: &str) -> String {
    let trimmed = text.trim();

    // Handle JSDoc comments: /**
    if trimmed.starts_with("/**") {
        return self.string_processor.clean_comment(trimmed, CommentType::Doc);
    }

    // Handle block comments: /*
    if trimmed.starts_with("/*") {
        return self.string_processor.clean_comment(trimmed, CommentType::Block);
    }

    // Handle line comments: //
    if trimmed.starts_with("//") {
        return self.string_processor.clean_comment(trimmed, CommentType::Line);
    }

    trimmed.to_string()
}
```

**分析**：
- ✅ 使用字符串前缀匹配（`starts_with`），不是正则
- ✅ 调用`StringProcessor::clean_comment`，但只返回文本
- ⚠️ **问题**：没有返回format_info
- ⚠️ **需要改造**：应该使用`clean_comment_with_format`

### 3.2 需要改造的部分

#### 3.2.1 Python解析器的docstring清理

**当前实现**：
```rust
fn clean_docstring_text(&self, text: &str) -> String {
    // ... 清理逻辑 ...
    processed_lines.join("\n").trim_end().to_string()
}
```

**问题**：
- 只返回清理后的文本
- 没有保存格式信息（缩进、引号类型等）
- Writer阶段无法正确重构docstring

**改造方案**：
```rust
fn clean_docstring_text(&self, text: &str) -> CleanedString {
    let trimmed = text
        .trim_start()
        .trim_end_matches(|c: char| c.is_whitespace() && c != '\n');

    let (string_style, quote_char, content) = if trimmed.starts_with("\"\"\"") && trimmed.ends_with("\"\"\"") {
        (StringStyle::DoubleQuoted, '"', &trimmed[3..trimmed.len() - 3])
    } else if trimmed.starts_with("'''") && trimmed.ends_with("'''") {
        (StringStyle::SingleQuoted, '\'', &trimmed[3..trimmed.len() - 3])
    } else if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 1 {
        (StringStyle::DoubleQuoted, '"', &trimmed[1..trimmed.len() - 1])
    } else if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() > 1 {
        (StringStyle::SingleQuoted, '\'', &trimmed[1..trimmed.len() - 1])
    } else {
        (StringStyle::DoubleQuoted, '"', trimmed)
    };

    // Process lines to remove common leading indentation
    let lines: Vec<&str> = content.lines().collect();
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    let processed_lines: Vec<String> = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                line.chars().skip(min_indent).collect()
            }
        })
        .collect();

    let cleaned_text = processed_lines.join("\n").trim_end().to_string();

    let base_indent = " ".repeat(min_indent);
    let is_multiline = cleaned_text.contains('\n');

    let format_info = FormatInfo {
        style: CommentStyle::DocBlock,
        base_indent,
        line_prefix: if is_multiline {
            Some(format!("{}{} ", base_indent, quote_char))
        } else {
            None
        },
        ends_with_newline: false,
        is_multiline,
        string_style: Some(string_style),
        placeholders: None,
        quote_char: Some(quote_char),
    };

    CleanedString {
        text: cleaned_text,
        format_info,
        placeholders: Vec::new(),
    }
}
```

#### 3.2.2 JavaScript/Rust解析器的注释清理

**当前实现**：
```rust
fn clean_comment_text(&self, text: &str) -> String {
    let trimmed = text.trim();

    if trimmed.starts_with("///") {
        return self.string_processor.clean_comment(trimmed, CommentType::Doc);
    }
    // ...
}
```

**问题**：
- 调用`clean_comment`只返回文本
- 没有保存format_info

**改造方案**：
```rust
fn clean_comment_text(&self, text: &str) -> CleanedComment {
    let trimmed = text.trim();

    let cleaned = if trimmed.starts_with("///") {
        self.string_processor.clean_comment_with_format(trimmed, CommentType::Doc)
    } else if trimmed.starts_with("//") {
        self.string_processor.clean_comment_with_format(trimmed, CommentType::Line)
    } else if trimmed.starts_with("/*") {
        self.string_processor.clean_comment_with_format(trimmed, CommentType::Block)
    } else {
        CleanedComment {
            text: trimmed.to_string(),
            format_info: FormatInfo::line_comment(""),
        }
    };

    cleaned
}
```

---

## 四、占位符替换方案设计

### 4.1 核心思想

**不使用format_info重构，而是直接替换原始匹配中的提取部分。**

### 4.2 数据结构扩展

```rust
pub struct TranslationUnit {
    pub id: String,
    pub node_type: NodeType,
    pub content: String,              // 提取的文本（用于翻译）
    pub start_pos: Position,
    pub end_pos: Position,
    
    // 新增字段
    pub raw_match: Option<String>,     // 完整的原始匹配（正则匹配用）
    pub placeholder: Option<String>,     // 占位符标识（可选）
    
    // 原有字段
    pub format_info: Option<FormatInfo>, // Tree-sitter用
    pub pattern_type: Option<PatternType>,
    pub pattern_name: Option<String>,
    // ...
}
```

### 4.3 提取阶段（Parser）

#### CustomPatternMatcher
```rust
pub struct CustomPatternMatch {
    pub raw_content: String,      // 完整匹配
    pub extracted_text: String,    // 提取的文本
    pub placeholder: String,       // 占位符
    pub start_pos: Position,
    pub end_pos: Position,
    pub pattern_name: String,
}

impl CustomPatternMatcher {
    pub fn find_matches(&self, content: &str) -> Result<Vec<CustomPatternMatch>> {
        let mut matches = Vec::new();

        for mat in self.regex.find_iter(content) {
            let raw_content = mat.as_str().to_string();
            let extracted_text = if let Some(captured) = self.regex.captures(&raw_content) {
                if let Some(group) = captured.get(self.group) {
                    group.as_str().to_string()
                } else {
                    raw_content.clone()
                }
            } else {
                raw_content.clone()
            };

            // 生成唯一占位符
            let placeholder = format!("__TRANSLATION_{}__", uuid::Uuid::new_v4());

            matches.push(CustomPatternMatch {
                raw_content,
                extracted_text,
                placeholder,
                start_pos: Position::new(0, 0, mat.start()),
                end_pos: Position::new(0, 0, mat.end()),
                pattern_name: self.name.clone(),
            });
        }

        Ok(matches)
    }
}
```

#### StateMachineMatcher
```rust
pub struct StateMachineMatch {
    pub raw_content: String,      // 完整匹配
    pub extracted_text: String,    // 提取的文本
    pub placeholder: String,       // 占位符
    pub start_pos: Position,
    pub end_pos: Position,
    pub state_name: String,
    pub captures: Vec<String>,
}
```

#### RegexParser
```rust
// 创建TranslationUnit时保存raw_match
for m in matches {
    let unit = TranslationUnit::new(
        id,
        NodeType::StringLiteral,
        m.extracted_text,
        m.start_pos,
        m.end_pos,
    );
    unit.raw_match = Some(m.raw_content);
    unit.placeholder = Some(m.placeholder);
    unit.pattern_type = Some(PatternType::CustomRegex);
    unit.pattern_name = Some(m.pattern_name);
    units.push(unit);
}
```

### 4.4 翻译阶段

```rust
// Translator
pub async fn translate(&self, units: &mut Vec<TranslationUnit>) -> Result<()> {
    for unit in units {
        // 只翻译提取的内容
        let translated = self.translate_text(&unit.content).await?;
        unit.translated = Some(translated);
    }
}
```

### 4.5 写回阶段（Writer）

```rust
pub enum ReplacementStrategy {
    FormatInfoBased,    // 使用format_info重构（Tree-sitter）
    DirectReplacement,   // 直接替换提取部分（正则）
}

impl TranslationUnit {
    pub fn replacement_strategy(&self) -> ReplacementStrategy {
        if self.format_info.is_some() {
            ReplacementStrategy::FormatInfoBased
        } else if self.raw_match.is_some() {
            ReplacementStrategy::DirectReplacement
        } else {
            ReplacementStrategy::DirectReplacement
        }
    }
}

impl TranslationApplier {
    pub fn apply_translations(content: &str, units: &[TranslationUnit]) -> Result<String> {
        let mut result = content.to_string();
        
        for unit in units {
            if let Some(translated) = &unit.translated {
                match unit.replacement_strategy() {
                    ReplacementStrategy::FormatInfoBased => {
                        // 使用format_info重构
                        if let Some(format) = &unit.format_info {
                            let formatted = Self::format_translated_text(translated, format);
                            result = Self::replace_with_format(result, unit, formatted)?;
                        }
                    }
                    ReplacementStrategy::DirectReplacement => {
                        // 直接替换提取部分
                        result = Self::replace_extracted(result, unit, translated)?;
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    fn replace_extracted(
        content: String,
        unit: &TranslationUnit,
        translated: &str,
    ) -> Result<String> {
        // 计算提取部分在raw_match中的偏移
        let raw_match = unit.raw_match.as_ref().ok_or_else(|| {
            TranslateError::Parse("raw_match is required for direct replacement".to_string())
        })?;
        
        let offset = raw_match.find(&unit.content).ok_or_else(|| {
            TranslateError::Parse("Extracted content not found in raw match".to_string())
        })?;
        
        // 计算实际替换范围
        let start = unit.start_pos.offset + offset;
        let end = start + unit.content.len();
        
        // 使用位置信息精确替换
        let mut chars: Vec<char> = content.chars().collect();
        let mut result = String::with_capacity(content.len());
        
        result.extend(&chars[..start]);
        result.push_str(translated);
        result.extend(&chars[end..]);
        
        Ok(result)
    }
}
```

---

## 五、改造方案总结

### 5.1 改造优先级

| 优先级 | 改造项 | 原因 | 复杂度 |
|-------|--------|------|--------|
| P0 | RegexParser添加raw_match支持 | 核心功能缺失 | 中 |
| P1 | Python解析器docstring清理 | 格式信息丢失 | 中 |
| P1 | JavaScript/Rust解析器注释清理 | 格式信息丢失 | 低 |
| P2 | StateMachineMatcher添加raw_match | 一致性 | 中 |
| P3 | 自定义正则模式占位符替换 | 性能优化 | 高 |

### 5.2 实施步骤

#### 步骤1：扩展数据结构
```rust
// src/core/models.rs
pub struct TranslationUnit {
    // ... 现有字段 ...
    
    /// Raw match content (for regex-based extraction)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_match: Option<String>,
    
    /// Placeholder identifier (for multi-stage replacement)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
}
```

#### 步骤2：修改Matcher
```rust
// src/parser/regex/custom_pattern_matcher.rs
pub struct CustomPatternMatch {
    pub raw_content: String,
    pub extracted_text: String,
    pub placeholder: String,
    pub start_pos: Position,
    pub end_pos: Position,
    pub pattern_name: String,
}

// src/parser/regex/state_machine.rs
pub struct StateMachineMatch {
    pub raw_content: String,
    pub extracted_text: String,
    pub placeholder: String,
    pub start_pos: Position,
    pub end_pos: Position,
    pub state_name: String,
    pub captures: Vec<String>,
}
```

#### 步骤3：修改Parser
```rust
// src/parser/regex/parser.rs
for m in matches {
    let unit = TranslationUnit::new(
        id,
        NodeType::StringLiteral,
        m.extracted_text,
        m.start_pos,
        m.end_pos,
    );
    unit.raw_match = Some(m.raw_content);
    unit.placeholder = Some(m.placeholder);
    unit.pattern_type = Some(PatternType::CustomRegex);
    unit.pattern_name = Some(m.pattern_name);
    units.push(unit);
}
```

#### 步骤4：修改语言解析器
```rust
// src/parser/languages/python/parser.rs
fn clean_docstring_text(&self, text: &str) -> CleanedString {
    // 返回CleanedString而不是String
    // 包含完整的format_info
}

// src/parser/languages/javascript/parser.rs
fn clean_comment_text(&self, text: &str) -> CleanedComment {
    // 返回CleanedComment而不是String
    // 包含完整的format_info
}
```

#### 步骤5：修改Writer
```rust
// src/writer/core.rs
pub enum ReplacementStrategy {
    FormatInfoBased,
    DirectReplacement,
}

impl TranslationApplier {
    pub fn apply_translations(content: &str, units: &[TranslationUnit]) -> Result<String> {
        // 根据replacement_strategy选择不同的替换逻辑
    }
}
```

### 5.3 测试验证

#### 测试用例1：自定义正则模式
```javascript
// 原始代码
TODO: Fix this bug

// 提取
raw_match: "TODO: Fix this bug"
extracted_text: "Fix this bug"

// 翻译
translated: "修复这个bug"

// 写回
TODO: 修复这个bug  // ✅ 正确
```

#### 测试用例2：Python docstring
```python
# 原始代码
def hello():
    """
    Hello world
    """

# 提取
content: "Hello world"
format_info: {
    style: DocBlock,
    base_indent: "    ",
    line_prefix: Some("    "),
    is_multiline: true,
    string_style: Some(DoubleQuoted),
    quote_char: Some('"'),
}

# 翻译
translated: "你好世界"

# 写回
def hello():
    """
    你好世界
    """  // ✅ 正确
```

#### 测试用例3：JavaScript注释
```javascript
// 原始代码
// This is a comment

// 提取
content: "This is a comment"
format_info: {
    style: Line,
    base_indent: "",
    line_prefix: Some("// "),
}

# 翻译
translated: "这是一个注释"

// 写回
// 这是一个注释  // ✅ 正确
```

---

## 六、结论

### 6.1 核心发现

1. **format_info不适合正则匹配场景**
   - 正则匹配的边界是任意的，不是基于语法结构
   - 无法用固定的格式信息描述动态的匹配模式
   - 强行使用format_info会增加复杂度且效果不佳

2. **语言解析器中的正则使用是合理的**
   - StringProcessor中的占位符提取是正确的使用方式
   - 语言解析器中的clean_*_text方法使用了字符串匹配，不是正则
   - 但这些方法应该返回完整的format_info

3. **需要改造的部分**
   - Python解析器的docstring清理：应该返回CleanedString
   - JavaScript/Rust解析器的注释清理：应该返回CleanedComment
   - RegexParser：应该添加raw_match支持

### 6.2 推荐方案

**采用混合策略**：
- **Tree-sitter解析**：继续使用`format_info`方案
- **正则匹配**：使用`raw_match + 直接替换`方案

### 6.3 预期效果

| 场景 | Tree-sitter | 正则匹配 |
|-----|------------|----------|
| 注释 | ✅ 完美重构 | ✅ 直接替换 |
| 字符串 | ✅ 完美重构 | ✅ 直接替换 |
| TODO模式 | N/A | ✅ 直接替换 |
| 错误消息 | N/A | ✅ 直接替换 |
| 日志消息 | N/A | ✅ 直接替换 |
| Python docstring | ✅ 完美重构 | N/A |

这样的设计既保留了Tree-sitter的强大功能，又为正则匹配提供了简单有效的解决方案。
