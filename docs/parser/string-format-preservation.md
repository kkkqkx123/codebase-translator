# Parser 字符串格式保留改进方案

## 文档信息

- **创建日期**: 2026-03-19
- **目标模块**: `src/parser`
- **关联模块**: `src/core/models`, `src/writer`
- **优先级**: 高

---

## 一、问题分析

### 1.1 当前实现的问题

当前 Parser 模块对字符串字面量、日志消息、错误信息等代码内字段的处理存在以下问题：

| 问题 | 注释处理 | 字符串/日志/错误处理 |
|------|----------|---------------------|
| **提取内容** | 纯文本（无标记） | 纯文本（无引号） |
| **格式信息** | 有 `FormatInfo` | **无格式信息** |
| **写入还原** | 自动还原格式 | 需手动包含引号 |
| **多行支持** | 完整支持 | 部分支持 |

### 1.2 具体表现

**Parser 提取阶段** (`src/parser/core/string_processor.rs:429`):
```rust
// 当前实现 - 仅返回纯文本
pub fn clean_string_literal(&self, text: &str) -> String {
    // 去除引号，处理转义，但不保留格式信息
    if text.starts_with('r') {
        self.process_raw_string(text)  // r#"..."# → ...
    } else {
        self.unescape(text.trim_matches('"'))  // "..." → ...
    }
}
```

**Writer 写入阶段** (`src/writer/core.rs:250`):
```rust
// 由于没有 format_info，无法自动还原格式
fn format_translated_text(translated: &str, format: &FormatInfo) -> String {
    match format.style {
        CommentStyle::Line => format!("{}{}", prefix, translated),
        // 字符串类型没有对应的处理分支
        _ => translated.to_string(),
    }
}
```

**测试结果** (`tests/writer_integration/complex_format_tests.rs:95`):
```rust
// 必须手动在翻译中包含引号
units[0].set_translated("\"你好世界\"");
```

---

## 二、改进目标

实现与注释处理相同的格式保留能力：

1. **提取阶段**: 识别并记录字符串的格式属性
2. **翻译阶段**: 仅翻译纯文本内容
3. **写入阶段**: 自动还原原始格式

---

## 三、设计方案

### 3.1 扩展现有 FormatInfo

在 `src/core/models.rs` 中扩展现有 `FormatInfo` 结构，添加对字符串的支持：

```rust
/// 字符串字面量样式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StringStyle {
    /// 普通双引号字符串: "hello"
    DoubleQuoted,
    /// 普通单引号字符串: 'hello' (Python/JS)
    SingleQuoted,
    /// 原始字符串: r"hello", r#"hello"#
    Raw { hash_count: u8 },
    /// 字节字符串: b"hello" (Rust)
    ByteString,
    /// 格式化字符串: f"hello {name}" (Python)
    Formatted,
    /// 模板字符串: `hello ${name}` (JS)
    Template,
    /// Go 原始字符串: `hello`
    Backtick,
}

/// 格式占位符类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatPlaceholder {
    /// Python 风格: %s, %d, %(name)s
    PythonStyle(String),
    /// C 风格: %s, %d
    CStyle(String),
    /// Rust 风格: {}, {name}
    RustStyle(String),
    /// Python f-string: {name}
    FString(String),
    /// JS 模板: ${name}
    JSTemplate(String),
}

/// 扩展现有 FormatInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    // 现有字段...
    pub style: CommentStyle,
    pub base_indent: String,
    pub line_prefix: Option<String>,
    pub ends_with_newline: bool,
    pub is_multiline: bool,
    
    // 新增字段 - 字符串专用
    /// 字符串样式（如果是字符串类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_style: Option<StringStyle>,
    /// 格式占位符列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholders: Option<Vec<FormatPlaceholder>>,
    /// 原始引号字符（", ', `）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_char: Option<char>,
}
```

### 3.2 StringProcessor 增强

在 `src/parser/core/string_processor.rs` 中添加新方法：

```rust
/// 字符串清理结果，包含格式信息
#[derive(Debug, Clone)]
pub struct CleanedString {
    /// 清理后的文本内容（无引号、无转义）
    pub text: String,
    /// 格式信息
    pub format_info: FormatInfo,
    /// 提取的占位符
    pub placeholders: Vec<FormatPlaceholder>,
}

impl StringProcessor {
    /// 清理字符串并提取格式信息
    pub fn clean_string_literal_with_format(&self, text: &str) -> CleanedString {
        let base_indent: String = text.chars().take_while(|c| c.is_whitespace()).collect();
        let trimmed = text.trim_start();
        
        // 检测字符串类型
        let (string_style, quote_char, content_start) = self.detect_string_style(trimmed);
        
        // 提取内容
        let content = self.extract_string_content(trimmed, &string_style);
        
        // 处理转义（非原始字符串）
        let cleaned_text = match string_style {
            StringStyle::Raw { .. } | StringStyle::Backtick => content.to_string(),
            _ => self.unescape(content),
        };
        
        // 提取格式占位符
        let placeholders = self.extract_placeholders(&cleaned_text, &string_style);
        
        // 构建 FormatInfo
        let format_info = FormatInfo {
            style: CommentStyle::Line, // 字符串使用 Line 作为基础
            base_indent,
            line_prefix: None,
            ends_with_newline: false,
            is_multiline: cleaned_text.contains('\n'),
            string_style: Some(string_style),
            placeholders: Some(placeholders.clone()),
            quote_char: Some(quote_char),
        };
        
        CleanedString {
            text: cleaned_text,
            format_info,
            placeholders,
        }
    }
    
    /// 检测字符串样式
    fn detect_string_style(&self, text: &str) -> (StringStyle, char, usize) {
        if text.starts_with("r#\"") {
            // 计算 hash 数量
            let hash_count = text[1..].chars().take_while(|&c| c == '#').count() as u8;
            (StringStyle::Raw { hash_count }, '"', 2 + hash_count as usize)
        } else if text.starts_with("r\"") {
            (StringStyle::Raw { hash_count: 0 }, '"', 2)
        } else if text.starts_with("b\"") {
            (StringStyle::ByteString, '"', 2)
        } else if text.starts_with("f\"") || text.starts_with("F\"") {
            (StringStyle::Formatted, '"', 2)
        } else if text.starts_with('\'') && text.ends_with('\'') {
            (StringStyle::SingleQuoted, '\'', 1)
        } else if text.starts_with('`') && text.ends_with('`') {
            (StringStyle::Backtick, '`', 1)
        } else {
            (StringStyle::DoubleQuoted, '"', 1)
        }
    }
    
    /// 提取字符串内容（去除引号和前缀）
    fn extract_string_content<'a>(&self, text: &'a str, style: &StringStyle) -> &'a str {
        match style {
            StringStyle::Raw { hash_count } => {
                let start = 2 + *hash_count as usize; // r + #*n + "
                let end = text.len() - 1 - *hash_count as usize; // " + #*n
                &text[start..end]
            }
            StringStyle::ByteString | StringStyle::Formatted => {
                &text[2..text.len()-1] // b"/f" + ... + "
            }
            StringStyle::SingleQuoted | StringStyle::Backtick => {
                &text[1..text.len()-1]
            }
            _ => {
                &text[1..text.len()-1] // "..."
            }
        }
    }
    
    /// 提取格式占位符
    fn extract_placeholders(&self, text: &str, style: &StringStyle) -> Vec<FormatPlaceholder> {
        let mut placeholders = Vec::new();
        
        match style {
            StringStyle::Formatted => {
                // Python f-string: {name} 或 {name!r}
                let re = regex::Regex::new(r"\{(\w+)(![^}]*)?(?::[^}]*)?\}").unwrap();
                for cap in re.captures_iter(text) {
                    placeholders.push(FormatPlaceholder::FString(cap[1].to_string()));
                }
            }
            StringStyle::Template => {
                // JS 模板: ${name}
                let re = regex::Regex::new(r"\$\{(\w+)\}").unwrap();
                for cap in re.captures_iter(text) {
                    placeholders.push(FormatPlaceholder::JSTemplate(cap[1].to_string()));
                }
            }
            _ => {
                // 检测 %s, %d, {} 等
                let re = regex::Regex::new(r"%[sdifoxXeEgcG]|\{\}").unwrap();
                for mat in re.find_iter(text) {
                    if mat.as_str().starts_with('%') {
                        placeholders.push(FormatPlaceholder::CStyle(mat.as_str().to_string()));
                    } else {
                        placeholders.push(FormatPlaceholder::RustStyle(mat.as_str().to_string()));
                    }
                }
            }
        }
        
        placeholders
    }
}
```

### 3.3 各语言 Parser 修改

#### Rust Parser (`src/parser/languages/rust/parser.rs`)

```rust
fn extract_macro_strings(
    &self,
    root_node: &Node,
    content: &str,
    file_path: &str,
) -> Result<Vec<TranslationUnit>> {
    // ... 现有代码 ...
    
    for m in matches {
        match m.capture_name.as_str() {
            "macro_string" => {
                // 使用新方法提取格式信息
                let cleaned = self.string_processor.clean_string_literal_with_format(m.text);
                
                // 分类宏
                let strategy_node_type = match self.patterns.classify_macro(&current_macro) {
                    Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
                    Some(FunctionCategory::Format) => StrategyNodeType::FormatString,
                    Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
                    _ => continue,
                };
                
                // 创建 TranslationUnit，包含 format_info
                let id = format!("{}_macro_{}", file_path, match_idx);
                let node_type = self.strategy.get_node_type(strategy_node_type);
                let mut unit = TranslationUnit::new(
                    id, 
                    node_type, 
                    cleaned.text,  // 纯文本内容
                    m.start_pos, 
                    m.end_pos
                );
                
                // 保存格式信息
                unit.format_info = Some(cleaned.format_info);
                units.push(unit);
                match_idx += 1;
            }
            _ => {}
        }
    }
    
    Ok(units)
}
```

#### Python Parser (`src/parser/languages/python/parser.rs`)

```rust
fn extract_function_strings(
    &self,
    root_node: &Node,
    content: &str,
    file_path: &str,
) -> Result<Vec<TranslationUnit>> {
    // ... 现有代码 ...
    
    for m in matches {
        match m.capture_name.as_str() {
            "func_string" => {
                let cleaned = self.string_processor.clean_string_literal_with_format(m.text);
                
                // 分类函数
                let strategy_node_type = match self.patterns.classify_function(&full_func_name) {
                    Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
                    Some(FunctionCategory::Format) => StrategyNodeType::FormatString,
                    Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
                    _ => continue,
                };
                
                let id = format!("{}_func_{}", file_path, match_idx);
                let node_type = self.strategy.get_node_type(strategy_node_type);
                let mut unit = TranslationUnit::new(
                    id,
                    node_type,
                    cleaned.text,
                    m.start_pos,
                    m.end_pos
                );
                
                unit.format_info = Some(cleaned.format_info);
                units.push(unit);
                match_idx += 1;
            }
            _ => {}
        }
    }
    
    Ok(units)
}
```

### 3.4 Writer 模块修改

在 `src/writer/core.rs` 中添加字符串格式处理：

```rust
impl TranslationApplier {
    /// 格式化翻译后的文本
    fn format_translated_text(translated: &str, format: &FormatInfo) -> String {
        // 如果是字符串类型，使用字符串格式化
        if let Some(string_style) = &format.string_style {
            return Self::format_string_literal(translated, format, string_style);
        }
        
        // 原有注释处理逻辑...
        match format.style {
            CommentStyle::Line => { /* ... */ }
            // ...
        }
    }
    
    /// 格式化字符串字面量
    fn format_string_literal(
        translated: &str, 
        format: &FormatInfo,
        style: &StringStyle
    ) -> String {
        let quote = format.quote_char.unwrap_or('"');
        
        match style {
            StringStyle::DoubleQuoted => {
                // 转义引号并包装
                let escaped = translated.replace(quote, &format!("\\{}", quote));
                format!("{}{}{}", 
                    format.base_indent,
                    quote,
                    escaped
                )
            }
            StringStyle::Raw { hash_count } => {
                // 重建原始字符串: r#"..."#
                let hashes = "#".repeat(*hash_count as usize);
                format!("{}r{}\"{}\"{}",
                    format.base_indent,
                    hashes,
                    translated,
                    hashes
                )
            }
            StringStyle::ByteString => {
                let escaped = translated.replace('"', "\\\"");
                format!("{}b\"{}\"", format.base_indent, escaped)
            }
            StringStyle::Formatted => {
                // 保留占位符
                let escaped = translated.replace('"', "\\\"");
                format!("{}f\"{}\"", format.base_indent, escaped)
            }
            StringStyle::Backtick => {
                format!("{}`{}`", format.base_indent, translated)
            }
            _ => translated.to_string(),
        }
    }
}
```

---

## 四、占位符处理策略

### 4.1 占位符保护

在翻译过程中需要保护格式占位符不被翻译：

```rust
/// 占位符保护器
pub struct PlaceholderProtector;

impl PlaceholderProtector {
    /// 将占位符替换为标记
    pub fn protect(text: &str, placeholders: &[FormatPlaceholder]) -> (String, Vec<String>) {
        let mut protected = text.to_string();
        let mut markers = Vec::new();
        
        for (i, placeholder) in placeholders.iter().enumerate() {
            let marker = format!("<<<PLACEHOLDER_{}>>>", i);
            let pattern = match placeholder {
                FormatPlaceholder::FString(name) => format!("{{{}}}", name),
                FormatPlaceholder::JSTemplate(name) => format!("${{{}}}", name),
                FormatPlaceholder::CStyle(s) => s.clone(),
                FormatPlaceholder::RustStyle(s) => s.clone(),
                FormatPlaceholder::PythonStyle(s) => s.clone(),
            };
            
            protected = protected.replace(&pattern, &marker);
            markers.push(pattern);
        }
        
        (protected, markers)
    }
    
    /// 恢复占位符
    pub fn restore(text: &str, markers: &[String]) -> String {
        let mut restored = text.to_string();
        
        for (i, marker) in markers.iter().enumerate() {
            let placeholder = format!("<<<PLACEHOLDER_{}>>>", i);
            restored = restored.replace(&placeholder, marker);
        }
        
        restored
    }
}
```

### 4.2 翻译流程集成

```rust
// 在翻译服务中
fn translate_with_protection(
    text: &str,
    placeholders: &[FormatPlaceholder],
    translator: &dyn Translator
) -> Result<String> {
    // 1. 保护占位符
    let (protected, markers) = PlaceholderProtector::protect(text, placeholders);
    
    // 2. 翻译
    let translated = translator.translate(&protected)?;
    
    // 3. 恢复占位符
    let restored = PlaceholderProtector::restore(&translated, &markers);
    
    Ok(restored)
}
```

---

## 五、实现步骤

### 5.1 第一阶段：核心数据结构

1. **修改 `src/core/models.rs`**
   - 添加 `StringStyle` 枚举
   - 添加 `FormatPlaceholder` 枚举
   - 扩展 `FormatInfo` 结构

2. **修改 `src/parser/core/string_processor.rs`**
   - 添加 `CleanedString` 结构
   - 实现 `clean_string_literal_with_format` 方法
   - 实现 `detect_string_style` 方法
   - 实现 `extract_placeholders` 方法

### 5.2 第二阶段：Parser 集成

3. **修改 `src/parser/languages/rust/parser.rs`**
   - 更新 `extract_macro_strings` 使用新方法

4. **修改 `src/parser/languages/python/parser.rs`**
   - 更新 `extract_function_strings` 使用新方法

5. **修改 `src/parser/languages/go/parser.rs`**
   - 更新 `extract_function_strings` 使用新方法

6. **修改其他语言 Parser**
   - JavaScript/TypeScript、Java、C/C++、C#

### 5.3 第三阶段：Writer 集成

7. **修改 `src/writer/core.rs`**
   - 添加 `format_string_literal` 方法
   - 更新 `format_translated_text` 处理字符串类型

### 5.4 第四阶段：占位符保护

8. **创建 `src/translation/placeholder.rs`**
   - 实现 `PlaceholderProtector`

9. **集成到翻译服务**
   - 修改翻译流程，添加占位符保护

### 5.5 第五阶段：测试

10. **添加单元测试**
    - `string_processor` 测试各种字符串类型
    - `placeholder_protector` 测试占位符保护/恢复

11. **添加集成测试**
    - 端到端字符串翻译测试
    - 多语言字符串格式保留测试

---

## 六、预期效果

### 6.1 使用示例

**提取前**:
```rust
println!(r#"Hello "world"!"#);
log::info!("User {} logged in", username);
```

**提取后** (`TranslationUnit`):
```rust
TranslationUnit {
    content: "Hello \"world\"!",  // 纯文本
    format_info: Some(FormatInfo {
        string_style: Some(StringStyle::Raw { hash_count: 1 }),
        quote_char: Some('"'),
        placeholders: Some(vec![]),
        // ...
    }),
    // ...
}

TranslationUnit {
    content: "User {} logged in",  // 纯文本
    format_info: Some(FormatInfo {
        string_style: Some(StringStyle::DoubleQuoted),
        quote_char: Some('"'),
        placeholders: Some(vec![FormatPlaceholder::RustStyle("{}".to_string())]),
        // ...
    }),
    // ...
}
```

**翻译后写入**:
```rust
println!(r#"你好 "世界"!"#);
log::info!("用户 {} 已登录", username);
```

### 6.2 支持的字符串类型

| 语言 | 字符串类型 | 示例 | 支持状态 |
|------|-----------|------|----------|
| Rust | 普通字符串 | `"hello"` | ✅ |
| Rust | 原始字符串 | `r#"hello"#` | ✅ |
| Rust | 字节字符串 | `b"hello"` | ✅ |
| Python | 普通字符串 | `"hello"`, `'hello'` | ✅ |
| Python | f-string | `f"hello {name}"` | ✅ |
| Python | 原始字符串 | `r"hello"` | ✅ |
| Go | 解释字符串 | `"hello"` | ✅ |
| Go | 原始字符串 | `` `hello` `` | ✅ |
| JavaScript | 普通字符串 | `"hello"`, `'hello'` | ✅ |
| JavaScript | 模板字符串 | `` `hello ${name}` `` | ✅ |

---

## 七、风险与注意事项

### 7.1 潜在风险

1. **向后兼容性**: `FormatInfo` 结构变更可能影响现有缓存
   - **缓解**: 使用 `#[serde(default)]` 和 `skip_serializing_if`

2. **性能影响**: 正则表达式提取占位符可能较慢
   - **缓解**: 使用惰性求值，仅在需要时提取

3. **多语言复杂性**: 不同语言的占位符语法差异大
   - **缓解**: 分阶段实现，优先支持主要语言

### 7.2 注意事项

1. **转义处理**: 确保翻译后的文本正确转义
2. **多行字符串**: 正确处理包含换行符的字符串
3. **占位符验证**: 翻译后验证占位符数量和类型是否匹配

---

## 八、参考资源

### 8.1 内部文档

- [design.md](./design.md) - Parser 模块整体设计
- [strategy-and-filter-guide.md](./strategy-and-filter-guide.md) - 策略和过滤器指南
- [功能分析.md](./功能分析.md) - 功能分析文档

### 8.2 外部资源

- [Tree-sitter Query Predicates](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/3-predicates-and-directives.html)
- [Rust Raw String Literals](https://rahul-thakoor.github.io/rust-raw-string-literals/)
- [Python Raw String](https://www.digitalocean.com/community/tutorials/python-raw-string)

---

## 九、总结

本改进方案旨在实现 Parser 模块对字符串字面量的完整格式保留能力，使其达到与注释处理相同的水平。通过扩展 `FormatInfo`、增强 `StringProcessor`、修改各语言 Parser 和 Writer 模块，实现：

1. **提取阶段**: 识别字符串类型、提取纯文本、记录格式信息
2. **翻译阶段**: 保护占位符、翻译纯文本
3. **写入阶段**: 自动还原原始格式

这将大大提升代码翻译的准确性和可用性，特别是对于包含大量字符串字面量的代码库。
