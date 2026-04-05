# 字符扫描提取方案设计

## 1. 背景与问题分析

### 1.1 当前 tree-sitter 方案的根本缺陷

当前实现基于 tree-sitter 进行文本提取，存在以下无法通过改进解决的根本性问题：

#### 问题1: 函数分类机制的致命缺陷

```rust
// 问题代码位置: src/parser/languages/typescript/parser.rs:266-270
let strategy_node_type = match self.patterns.classify_function(&current_func) {
    Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
    Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
    _ => continue, // ← 所有未知函数的字符串都被丢弃！
};
```

**无法解决的原因**：
- 用户代码中有无数自定义函数名
- 无法预测所有可能包含翻译文本的上下文
- 即使添加更多函数名，仍会有遗漏

#### 问题2: tree-sitter 的 rewrite 本质上就是手动处理

```
当前 tree-sitter 流程:

提取阶段:
  tree-sitter 解析 AST -> 提取节点 -> 清理标记 (//, /* */)

翻译阶段:
  纯文本内容 -> 翻译 API -> 翻译结果

写入阶段 (rewrite):
  翻译结果 -> 重新添加标记 (//, /* */) -> 替换原文
              ↑
           这里本质上就是手动处理格式，
           tree-sitter 的语法感知优势在这里完全丧失！
```

#### 问题3: 嵌套字符串无法正确处理

```javascript
// 问题示例
const msg = `错误: ${getErrorMsg("参数无效")}`;
// tree-sitter 无法正确处理嵌套的字符串边界

const text = "前缀" + getMsg("中间内容") + "后缀";
// 多个字符串在同一表达式中，边界识别困难

validate(config, "配置错误", { hint: "请检查格式" });
// 未知函数的所有参数都被跳过
```

### 1.2 tree-sitter 不适合此任务的核心原因

| 问题 | tree-sitter 状态 | 原因 |
|------|-----------------|------|
| 未知函数的字符串参数 | ❌ 无法处理 | 需要预定义函数列表 |
| 嵌套字符串 | ❌ 无法正确处理 | AST 层级关系复杂 |
| 模板字符串中的表达式 | ⚠️ 部分处理 | rewrite 时仍需手动保护 |
| 注释格式还原 | ⚠️ 需手动处理 | tree-sitter 不提供格式信息 |
| 性能 | ⚠️ 查询多时慢 | 多次遍历 AST |

## 2. 字符扫描方案设计

### 2.1 核心思想

**彻底放弃 tree-sitter，完全基于字符扫描 + 字符串边界提取 + 模板保护**

```
字符扫描流程:

单次扫描:
  扫描文件 -> 识别边界 -> 检测语言 -> 记录偏移 -> 保护模板

提取结果:
  原始文本: // 这是注释
  内容: "这是注释"
  边界: start=3, end=7 (字节偏移，不含注释标记)
  类型: LineComment
  前缀: "// "

替换: 直接基于字节偏移替换，无需重新构建格式
```

### 2.2 核心数据结构

```rust
/// 文本区域类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRegionType {
    /// 单行注释 // ...
    LineComment,
    /// 块注释 /* ... */
    BlockComment,
    /// 文档注释 /** ... */ 或 /// ...
    DocComment,
    /// 单引号字符串 '...'
    SingleQuotedString,
    /// 双引号字符串 "..."
    DoubleQuotedString,
    /// 模板字符串 `...`
    TemplateString,
    /// 原始字符串 r#"..."#, r"..."
    RawString,
    /// 多行字符串 """...""", '''...'''
    MultiLineString,
}

/// 扫描到的文本区域
#[derive(Debug, Clone)]
pub struct TextRegion {
    /// 区域类型
    pub region_type: TextRegionType,
    /// 内容起始字节偏移 (不含前缀)
    pub content_start: usize,
    /// 内容结束字节偏移 (不含后缀)
    pub content_end: usize,
    /// 完整区域起始字节偏移 (含前缀)
    pub full_start: usize,
    /// 完整区域结束字节偏移 (含后缀)
    pub full_end: usize,
    /// 前缀 (如 "// ", "/* ", "\"")
    pub prefix: String,
    /// 后缀 (如 " */", "\"")
    pub suffix: String,
    /// 模板占位符位置 (仅模板字符串)
    pub placeholders: Vec<PlaceholderSpan>,
}

/// 模板占位符
#[derive(Debug, Clone)]
pub struct PlaceholderSpan {
    /// 占位符起始位置 (相对于内容)
    pub start: usize,
    /// 占位符结束位置 (相对于内容)
    pub end: usize,
    /// 原始占位符文本
    pub original: String,
}
```

### 2.3 语言配置

```rust
/// 语言特定配置
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    /// 行注释前缀
    pub line_comment_prefixes: Vec<&'static str>,
    /// 块注释开始/结束
    pub block_comment_delimiters: Vec<(&'static str, &'static str)>,
    /// 文档注释前缀
    pub doc_comment_prefixes: Vec<&'static str>,
    /// 字符串引号
    pub string_quotes: Vec<char>,
    /// 模板字符串引号
    pub template_quote: Option<char>,
    /// 原始字符串前缀
    pub raw_string_prefixes: Vec<&'static str>,
    /// 多行字符串分隔符
    pub multiline_delimiters: Vec<&'static str>,
}

impl LanguageConfig {
    pub fn javascript() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec!["///", "/**"],
            string_quotes: vec!['"', '\''],
            template_quote: Some('`'),
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
        }
    }

    pub fn typescript() -> Self {
        Self::javascript()
    }

    pub fn python() -> Self {
        Self {
            line_comment_prefixes: vec!["#"],
            block_comment_delimiters: vec![],
            doc_comment_prefixes: vec!["\"\"\"", "'''"],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec!["r\"", "r'", "r\"\"\"", "r'''"],
            multiline_delimiters: vec!["\"\"\"", "'''"],
        }
    }

    pub fn rust() -> Self {
        Self {
            line_comment_prefixes: vec!["///", "//"],
            block_comment_delimiters: vec![("/**", "*/"), ("/*", "*/")],
            doc_comment_prefixes: vec!["///", "/**"],
            string_quotes: vec!['"'],
            template_quote: None,
            raw_string_prefixes: vec!["r#", "r\""],
            multiline_delimiters: vec![],
        }
    }

    pub fn go() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['"', '\''],
            template_quote: Some('`'),
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
        }
    }

    pub fn java() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec!["/**"],
            string_quotes: vec!['"'],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
        }
    }

    pub fn c() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
        }
    }

    pub fn cpp() -> Self {
        Self::c()
    }

    pub fn csharp() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec!["///", "/**"],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec!["@\"", "@'"],
            multiline_delimiters: vec![],
        }
    }
}
```

## 3. 扫描器实现

### 3.1 核心扫描器

```rust
use crate::parser::filtering::checks::language::strategy::QuickDetector;

/// 语言感知的文本扫描器
pub struct TextScanner {
    /// 语言配置
    language: LanguageConfig,
    /// 目标语言检测器
    detector: QuickDetector,
    /// 目标语言列表
    target_languages: Vec<String>,
}

impl TextScanner {
    /// 创建新的扫描器
    pub fn new(language: LanguageConfig, target_languages: Vec<String>) -> Self {
        Self {
            language,
            detector: QuickDetector::new(),
            target_languages,
        }
    }

    /// 扫描文件，提取所有包含目标语言的文本区域
    pub fn scan(&self, content: &str) -> Vec<TextRegion> {
        let mut regions = Vec::new();
        let bytes = content.as_bytes();
        let mut pos = 0;

        while pos < bytes.len() {
            // 按优先级尝试匹配不同类型的文本区域
            if let Some(region) = self.try_scan_region(bytes, pos) {
                if self.should_extract(&region, content) {
                    regions.push(region.clone());
                }
                pos = region.full_end;
                continue;
            }

            pos += 1;
        }

        regions
    }

    /// 尝试扫描任意类型的文本区域
    fn try_scan_region(&self, bytes: &[u8], pos: usize) -> Option<TextRegion> {
        // 按优先级顺序尝试:
        // 1. 文档注释 (最长前缀优先)
        // 2. 块注释
        // 3. 行注释
        // 4. 原始字符串
        // 5. 多行字符串
        // 6. 模板字符串
        // 7. 普通字符串

        // 文档注释
        for prefix in &self.language.doc_comment_prefixes {
            if bytes[pos..].starts_with(prefix.as_bytes()) {
                if prefix.starts_with("/*") || prefix.starts_with("/**") {
                    return self.scan_block_doc_comment(bytes, pos, prefix);
                } else {
                    return self.scan_line_doc_comment(bytes, pos, prefix);
                }
            }
        }

        // 块注释
        for (start, end) in &self.language.block_comment_delimiters {
            if bytes[pos..].starts_with(start.as_bytes()) {
                return self.scan_block_comment(bytes, pos, start, end);
            }
        }

        // 行注释
        for prefix in &self.language.line_comment_prefixes {
            if bytes[pos..].starts_with(prefix.as_bytes()) {
                return self.scan_line_comment(bytes, pos, prefix);
            }
        }

        // 原始字符串
        for prefix in &self.language.raw_string_prefixes {
            if bytes[pos..].starts_with(prefix.as_bytes()) {
                return self.scan_raw_string(bytes, pos, prefix);
            }
        }

        // 多行字符串
        for delim in &self.language.multiline_delimiters {
            if bytes[pos..].starts_with(delim.as_bytes()) {
                return self.scan_multiline_string(bytes, pos, delim);
            }
        }

        // 模板字符串
        if let Some(quote) = self.language.template_quote {
            if bytes[pos] == quote as u8 {
                return self.scan_template_string(bytes, pos);
            }
        }

        // 普通字符串
        for quote in &self.language.string_quotes {
            if bytes[pos] == *quote as u8 {
                return self.scan_quoted_string(bytes, pos, *quote);
            }
        }

        None
    }

    /// 扫描行注释
    fn scan_line_comment(&self, bytes: &[u8], pos: usize, prefix: &str) -> Option<TextRegion> {
        let prefix_len = prefix.len();
        let mut end = pos + prefix_len;

        // 找到行尾
        while end < bytes.len() && bytes[end] != b'\n' && bytes[end] != b'\r' {
            end += 1;
        }

        // 跳过前缀后的空白
        let content_start = pos + prefix_len;
        let content_start = if content_start < bytes.len() && bytes[content_start] == b' ' {
            content_start + 1
        } else {
            content_start
        };

        Some(TextRegion {
            region_type: TextRegionType::LineComment,
            content_start,
            content_end: end,
            full_start: pos,
            full_end: end,
            prefix: prefix.to_string(),
            suffix: String::new(),
            placeholders: Vec::new(),
        })
    }

    /// 扫描块注释
    fn scan_block_comment(
        &self,
        bytes: &[u8],
        pos: usize,
        start: &str,
        end: &str,
    ) -> Option<TextRegion> {
        let start_len = start.len();
        let mut current = pos + start_len;

        // 找到结束标记
        while current + end.len() <= bytes.len() {
            if bytes[current..].starts_with(end.as_bytes()) {
                let content_start = pos + start_len;
                let content_end = current;

                Some(TextRegion {
                    region_type: TextRegionType::BlockComment,
                    content_start,
                    content_end,
                    full_start: pos,
                    full_end: current + end.len(),
                    prefix: start.to_string(),
                    suffix: end.to_string(),
                    placeholders: Vec::new(),
                })
            }
            current += 1;
        }

        None
    }

    /// 扫描模板字符串 (关键：处理嵌套)
    fn scan_template_string(&self, bytes: &[u8], pos: usize) -> Option<TextRegion> {
        if bytes[pos] != b'`' {
            return None;
        }

        let mut end = pos + 1;
        let mut placeholders = Vec::new();
        let mut template_depth = 1;

        while end < bytes.len() {
            match bytes[end] {
                b'\\' => {
                    // 跳过转义字符
                    end += 2;
                    continue;
                }
                b'`' => {
                    template_depth -= 1;
                    if template_depth == 0 {
                        end += 1;
                        break;
                    }
                    end += 1;
                }
                b'$' if end + 1 < bytes.len() && bytes[end + 1] == b'{' => {
                    // 模板占位符开始 ${...}
                    let placeholder_start = end - pos - 1; // 相对于内容
                    end += 2;

                    // 找到匹配的 }，处理嵌套
                    let mut brace_depth = 1;
                    while end < bytes.len() && brace_depth > 0 {
                        match bytes[end] {
                            b'{' => brace_depth += 1,
                            b'}' => brace_depth -= 1,
                            b'`' => {
                                // 嵌套模板字符串
                                if let Some(nested) = self.scan_template_string(bytes, end) {
                                    end = nested.full_end;
                                    continue;
                                }
                            }
                            b'\\' => end += 1, // 跳过转义
                            _ => {}
                        }
                        end += 1;
                    }

                    placeholders.push(PlaceholderSpan {
                        start: placeholder_start,
                        end: end - pos - 1,
                        original: String::from_utf8_lossy(
                            &bytes[pos + 1 + placeholder_start..end - 1]
                        ).to_string(),
                    });
                }
                _ => {
                    end += 1;
                }
            }
        }

        Some(TextRegion {
            region_type: TextRegionType::TemplateString,
            content_start: pos + 1,
            content_end: end - 1,
            full_start: pos,
            full_end: end,
            prefix: "`".to_string(),
            suffix: "`".to_string(),
            placeholders,
        })
    }

    /// 扫描普通字符串
    fn scan_quoted_string(&self, bytes: &[u8], pos: usize, quote: char) -> Option<TextRegion> {
        let quote_byte = quote as u8;
        if bytes[pos] != quote_byte {
            return None;
        }

        let mut end = pos + 1;

        while end < bytes.len() {
            match bytes[end] {
                b'\\' => {
                    // 跳过转义字符
                    end += 2;
                    continue;
                }
                b'\n' | b'\r' => {
                    // 未闭合的字符串
                    return None;
                }
                c if c == quote_byte => {
                    end += 1;
                    break;
                }
                _ => {
                    end += 1;
                }
            }
        }

        Some(TextRegion {
            region_type: if quote == '"' {
                TextRegionType::DoubleQuotedString
            } else {
                TextRegionType::SingleQuotedString
            },
            content_start: pos + 1,
            content_end: end - 1,
            full_start: pos,
            full_end: end,
            prefix: quote.to_string(),
            suffix: quote.to_string(),
            placeholders: Vec::new(),
        })
    }

    /// 判断是否应该提取此区域
    fn should_extract(&self, region: &TextRegion, content: &str) -> bool {
        if region.content_start >= region.content_end {
            return false;
        }

        let text = &content[region.content_start..region.content_end];

        // 检测是否包含目标语言字符
        self.contains_target_language(text)
    }

    /// 检测文本是否包含目标语言
    fn contains_target_language(&self, text: &str) -> bool {
        for lang in &self.target_languages {
            let lang_upper = lang.to_uppercase();
            match lang_upper.as_str() {
                "ZH" | "ZH-CN" | "ZH-TW" | "HANS" | "HANT" => {
                    if self.detector.has_chinese(text) {
                        return true;
                    }
                }
                "JA" => {
                    if self.detector.has_japanese(text) {
                        return true;
                    }
                }
                "KO" => {
                    if self.detector.has_korean(text) {
                        return true;
                    }
                }
                _ => {
                    // 未知语言，默认允许
                    return true;
                }
            }
        }
        false
    }
}
```

### 3.2 模板占位符保护

```rust
impl TextScanner {
    /// 准备用于翻译的文本 (保护占位符)
    pub fn prepare_for_translation(&self, region: &TextRegion, content: &str) -> String {
        let text = &content[region.content_start..region.content_end];

        if region.placeholders.is_empty() {
            return text.to_string();
        }

        // 替换占位符为临时标记
        let mut result = text.to_string();
        // 从后往前替换，避免偏移变化
        let mut sorted = region.placeholders.clone();
        sorted.sort_by(|a, b| b.start.cmp(&a.start));

        for (idx, placeholder) in sorted.iter().enumerate() {
            let marker = format!("__PH_{}__", region.placeholders.len() - 1 - idx);
            result.replace_range(placeholder.start..placeholder.end, &marker);
        }

        result
    }

    /// 恢复模板占位符
    pub fn restore_placeholders(&self, translated: &str, region: &TextRegion) -> String {
        let mut result = translated.to_string();

        for (idx, placeholder) in region.placeholders.iter().enumerate() {
            let marker = format!("__PH_{}__", idx);
            result = result.replace(&marker, &placeholder.original);
        }

        result
    }
}
```

## 4. 翻译应用器

### 4.1 基于偏移的精确替换

```rust
/// 已翻译的区域
pub struct TranslatedRegion {
    pub content_start: usize,
    pub content_end: usize,
    pub original_content: String,
    pub translated_content: String,
}

/// 翻译应用器 - 基于字节偏移精确替换
pub struct TranslationReplacer;

impl TranslationReplacer {
    /// 应用翻译到内容
    pub fn apply(content: &str, regions: &[TranslatedRegion]) -> String {
        if regions.is_empty() {
            return content.to_string();
        }

        // 按起始位置降序排序，从后往前替换
        let mut sorted: Vec<_> = regions.iter().collect();
        sorted.sort_by(|a, b| b.content_start.cmp(&a.content_start));

        let mut result = content.to_string();

        for region in sorted {
            // 验证原始内容
            if region.content_start >= result.len() || region.content_end > result.len() {
                continue;
            }

            let original = &result[region.content_start..region.content_end];
            if original != region.original_content {
                // 内容不匹配，可能是因为之前的替换影响了偏移
                // 尝试在结果中查找原始内容
                if let Some(pos) = result.find(&region.original_content) {
                    result.replace_range(
                        pos..pos + region.original_content.len(),
                        &region.translated_content
                    );
                }
                continue;
            }

            // 执行替换
            result.replace_range(
                region.content_start..region.content_end,
                &region.translated_content
            );
        }

        result
    }
}
```

## 5. 配置设计

### 5.1 TOML 配置

```toml
# .translator.toml

[translate]
source_langs = ["ZH"]
target_lang = "EN"

[extraction]
# 扫描模式: "thorough" (字符扫描)
mode = "thorough"

[extraction.scanner]
# 要扫描的文本类型
scan_comments = true
scan_strings = true
scan_templates = true

# 注释配置
[extraction.scanner.comments]
line_comments = true
block_comments = true
doc_comments = true

# 字符串配置
[extraction.scanner.strings]
single_quoted = true
double_quoted = true
template_strings = true
raw_strings = true
multiline_strings = true

# 模板占位符保护
[extraction.scanner.placeholders]
enabled = true
# 占位符模式 (正则)
patterns = [
    "\\$\\{[^}]*\\}",      # ${...}
    "\\$\\d+",             # $1, $2
    "%[sdvf]",             # %s, %d
    "\\{\\}",              # {}
    "\\{[^}]+\\}",         # {name}
]
```

### 5.2 Rust 配置结构

```rust
/// 扫描器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    /// 扫描模式
    pub mode: ScanMode,
    /// 注释扫描配置
    pub comments: CommentScanConfig,
    /// 字符串扫描配置
    pub strings: StringScanConfig,
    /// 占位符保护配置
    pub placeholders: PlaceholderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanMode {
    /// 彻底扫描模式 (字符扫描)
    Thorough,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentScanConfig {
    pub line_comments: bool,
    pub block_comments: bool,
    pub doc_comments: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringScanConfig {
    pub single_quoted: bool,
    pub double_quoted: bool,
    pub template_strings: bool,
    pub raw_strings: bool,
    pub multiline_strings: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceholderConfig {
    pub enabled: bool,
    pub patterns: Vec<String>,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            mode: ScanMode::Thorough,
            comments: CommentScanConfig {
                line_comments: true,
                block_comments: true,
                doc_comments: true,
            },
            strings: StringScanConfig {
                single_quoted: true,
                double_quoted: true,
                template_strings: true,
                raw_strings: true,
                multiline_strings: true,
            },
            placeholders: PlaceholderConfig {
                enabled: true,
                patterns: vec![
                    r"\$\{[^}]*\}".to_string(),
                    r"\$\d+".to_string(),
                    r"%[sdvf]".to_string(),
                    r"\{\}".to_string(),
                    r"\{[^}]+\}".to_string(),
                ],
            },
        }
    }
}
```

## 6. 模块结构

```
src/parser/scanner/
├── mod.rs              # 模块入口
├── scanner.rs          # 核心扫描器
├── config.rs           # 配置定义
├── language.rs         # 语言配置
├── region.rs           # 文本区域定义
├── placeholder.rs      # 占位符处理
└── replacer.rs         # 翻译应用器
```

## 7. 优势对比

| 特性 | tree-sitter 方案 | 字符扫描方案 |
|------|-----------------|-------------|
| **提取完整性** | ❌ 依赖函数列表 | ✅ 提取所有文本 |
| **嵌套字符串** | ❌ 无法正确处理 | ✅ 基于偏移精确替换 |
| **性能** | ⚠️ 多次 AST 遍历 | ✅ 单次扫描 O(n) |
| **维护成本** | ❌ 需维护查询规则 | ✅ 仅需语言配置 |
| **代码复杂度** | ❌ 高 (AST + rewrite) | ✅ 低 (扫描 + 替换) |
| **格式保持** | ⚠️ 需手动重建 | ✅ 基于偏移保留原格式 |
| **模板保护** | ⚠️ 复杂 | ✅ 直接提取占位符位置 |

## 8. 实现计划

### 阶段1: 核心扫描器 (优先级: 高)

1. 实现 `TextScanner` 核心逻辑
2. 实现各语言配置
3. 实现模板占位符提取

### 阶段2: 集成到现有架构 (优先级: 高)

1. 创建 `ScannerParser` 替代现有语言解析器
2. 修改 `TranslationUnit` 以支持新的偏移信息
3. 更新 `TranslationReplacer`

### 阶段3: 测试和优化 (优先级: 中)

1. 单元测试覆盖各种边界情况
2. 集成测试验证完整流程
3. 性能基准测试

### 阶段4: 清理旧代码 (优先级: 低)

1. 移除 tree-sitter 相关代码
2. 移除函数分类逻辑
3. 更新文档

## 9. 风险和缓解措施

### 风险1: 转义字符处理

**风险**: 复杂的转义序列可能导致边界识别错误

**缓解措施**:
- 实现完整的转义字符处理逻辑
- 添加测试用例覆盖各种转义情况
- 对于无法确定边界的情况，跳过该区域

### 风险2: 多语言混合文件

**风险**: 某些文件可能包含多种语言的代码 (如 Vue SFC)

**缓解措施**:
- 支持基于文件扩展名的语言检测
- 支持在单个文件中切换语言配置

### 风险3: 不完整的字符串

**风险**: 代码中可能存在未闭合的字符串

**缓解措施**:
- 扫描时检测字符串是否正确闭合
- 对于未闭合的字符串，跳过该区域并记录警告

## 10. 总结

字符扫描方案彻底解决了 tree-sitter 方案的根本缺陷：

1. **完整性**: 提取所有包含目标语言的文本，不遗漏任何内容
2. **简洁性**: 单次扫描，基于偏移替换，无需复杂的 AST 处理
3. **可维护性**: 仅需维护语言配置，无需维护复杂的查询规则
4. **性能**: O(n) 时间复杂度，优于多次 AST 遍历

该方案优先考虑翻译的彻底性，接受可能的误提取，依赖编译错误进行后续修正。这是比当前实现更务实的选择。
