# 强制语言提取功能设计方案

## 1. 需求背景

### 1.1 当前语言检查机制

当前项目采用**分层过滤架构**来决定哪些内容需要翻译：

```
CompositeFilter (src/parser/filtering/composite.rs)
├── LengthFilter      (O(1))  - 长度检查
├── LanguageFilter    (O(32)) - 语言检测 ⭐
├── PatternFilter     (O(n))  - 模式匹配
└── ContentFilter     (O(len))- 内容分析
```

**LanguageFilter 核心逻辑**（`src/parser/filtering/checks/language/mod.rs`）：

- **配置项**：
  - `source_langs`: 源语言列表，支持 `AUTO` 自动检测
  - `target_lang`: 目标语言

- **检查规则**：
  ```rust
  // AUTO 模式：跳过已经是目标语言的文本
  if source_langs.contains("AUTO") {
      if is_target_language(text) {
          return false;  // 不翻译
      }
      return true;  // 翻译
  }

  // 指定源语言：只翻译含指定源语言字符的文本
  if source_langs.contains("ZH") {
      if has_chinese(text) {
          return true;  // 翻译
      }
      return false;  // 不翻译
  }
  ```

### 1.2 问题场景

**场景 1：提取所有中文内容**

用户配置：
```toml
source_langs = ["AUTO"]
target_lang = "EN"
```

当前行为：
- "你好世界" → ✅ 提取（含中文）
- "Hello world" → ❌ 跳过（已是目标语言）
- "Hello 你好" → ✅ 提取（含中文）

用户期望：提取所有中文内容，包括混合内容

**场景 2：复杂模式配置的痛点**

当前解决方案需要配置多个自定义正则：
```toml
[[extraction.custom_patterns]]
name = "extract_chinese"
regex = '[\u4e00-\u9fff]+'
file_extensions = ["rs", "py", "js"]

[[extraction.custom_patterns]]
name = "extract_japanese"
regex = '[\u3040-\u309f\u30a0-\u30ff]+'
file_extensions = ["rs", "py", "js"]
```

问题：
- ❌ 需要编写多个正则表达式
- ❌ 正则表达式不易维护
- ❌ 需要为每种语言重复配置
- ❌ 容易遗漏某些语言特征

**场景 3：混合语言代码**

代码示例：
```rust
// 用户提示：请输入用户名
let username = input("请输入用户名: ");

// 错误提示：用户名不能为空
if username.is_empty() {
    println("用户名不能为空");
}
```

当前行为：
- 部分被提取（取决于具体模式和语言检测）
- 可能遗漏某些中文注释

用户期望：提取所有中文内容，无需关心是否已经部分翻译

### 1.3 需求总结

需要一个**配置选项**，能够：
- ✅ 强制提取包含特定语言字符的所有文本
- ✅ 跳过其他所有过滤条件（模式、长度、占位符等）
- ✅ 避免复杂正则配置
- ✅ 保持高性能（O(32) 字符检查）
- ✅ 向后兼容（默认关闭）

---

## 2. 设计方案

### 2.1 配置设计

在 `[filter]` 配置中添加新选项：

```toml
[filter]
# 强制提取包含目标语言字符的文本，跳过所有其他过滤
# 启用后，仅检查语言特征，忽略模式、长度、占位符等过滤
force_extract_by_language = false  # 默认关闭

# 当启用 force_extract_by_language 时，指定要提取的语言特征
# 支持的语言代码：
#   ZH, ZH-CN, ZH-TW - 中文
#   JA - 日文
#   KO - 韩文
#   EN - 英文
#   AR - 阿拉伯文
#   RU - 俄文
extract_languages = ["ZH", "JA", "KO"]
```

### 2.2 数据结构设计

#### 2.2.1 扩展 FilterConfig

在 `src/parser/filtering/config.rs` 中添加：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    // ... 现有字段 ...

    /// 强制提取包含特定语言字符的文本
    ///
    /// 当启用此选项时，将跳过所有其他过滤条件（模式、长度、占位符等），
    /// 仅检查文本是否包含指定的语言字符。
    ///
    /// 此选项适用于需要提取所有特定语言内容的场景，例如：
    /// - 提取代码库中所有中文注释
    /// - 批量处理多语言混合内容
    /// - 不关心其他过滤条件的场景
    ///
    /// 默认值：false（关闭）
    #[serde(default)]
    pub force_extract_by_language: bool,

    /// 要提取的语言列表
    ///
    /// 仅在 `force_extract_by_language` 为 true 时生效。
    /// 支持的语言代码：
    /// - `ZH`, `ZH-CN`, `ZH-TW`: 中文
    /// - `JA`: 日文
    /// - `KO`: 韩文
    /// - `EN`: 英文
    /// - `AR`: 阿拉伯文
    /// - `RU`: 俄文
    /// - `UK`: 乌克兰文
    /// - `BG`: 保加利亚文
    ///
    /// 默认值：空列表（不启用强制提取）
    #[serde(default)]
    pub extract_languages: Vec<String>,
}
```

#### 2.2.2 创建 LanguageOnlyFilter

在 `src/parser/filtering/checks/language/` 中创建 `language_only.rs`：

```rust
//! Language-only filter for forced language extraction
//!
//! This filter checks only for language characteristics and ignores all other
//! filtering rules. Used when `force_extract_by_language` is enabled.

use super::QuickDetector;
use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Language-only filter that checks only for language characteristics
///
/// When enabled, this filter bypasses all other filtering rules and only
/// checks if the text contains characters from the specified languages.
pub struct LanguageOnlyFilter {
    /// Languages to extract
    languages: Vec<String>,
    /// Quick language detector
    detector: QuickDetector,
}

impl LanguageOnlyFilter {
    /// Create a new language-only filter
    ///
    /// # Arguments
    /// * `languages` - List of language codes to extract (e.g., ["ZH", "JA", "KO"])
    pub fn new(languages: Vec<String>) -> Self {
        Self {
            languages,
            detector: QuickDetector::new(),
        }
    }

    /// Check if text contains any of the specified language characters
    fn contains_target_language(&self, text: &str) -> bool {
        for lang in &self.languages {
            let lang_upper = lang.to_uppercase();
            match lang_upper.as_str() {
                "ZH" | "ZH-CN" | "ZH-TW" | "HANS" | "HANT" => {
                    if self.detector.has_chinese(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_chinese",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                "JA" => {
                    if self.detector.has_japanese(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_japanese",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                "KO" => {
                    if self.detector.has_korean(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_korean",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                "EN" | "EN-US" | "EN-GB" => {
                    if self.detector.is_latin(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_latin",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                "AR" => {
                    if self.detector.has_arabic(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_arabic",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                "RU" | "UK" | "BG" => {
                    if self.detector.has_cyrillic(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_cyrillic",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                // For unknown languages, allow through
                _ => {
                    debug!(
                        language = %lang,
                        reason = "unknown_language_allowed",
                        "Text matched language-only filter (unknown language)"
                    );
                    return true;
                }
            }
        }
        false
    }
}

impl Filter for LanguageOnlyFilter {
    fn should_translate(&self, text: &str) -> bool {
        // Check if text contains any of the specified language characters
        if self.languages.is_empty() {
            debug!(
                reason = "no_languages_specified",
                "Text filtered by language-only filter (no languages specified)"
            );
            return false;
        }

        let matches = self.contains_target_language(text);
        if !matches {
            debug!(
                languages = ?self.languages,
                reason = "no_target_language",
                "Text filtered by language-only filter"
            );
        }
        matches
    }

    fn name(&self) -> &str {
        "LanguageOnlyFilter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        assert!(filter.should_translate("你好世界"));
        assert!(filter.should_translate("Hello 你好"));
        assert!(filter.should_translate("你好Hello"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_japanese_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["JA".to_string()]);

        assert!(filter.should_translate("こんにちは"));
        assert!(filter.should_translate("カタカナ"));
        assert!(filter.should_translate("Hello こんにちは"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_korean_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["KO".to_string()]);

        assert!(filter.should_translate("안녕하세요"));
        assert!(filter.should_translate("Hello 안녕하세요"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_multiple_languages() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string(), "JA".to_string()]);

        assert!(filter.should_translate("你好世界"));
        assert!(filter.should_translate("こんにちは"));
        assert!(filter.should_translate("Hello 你好"));
        assert!(filter.should_translate("Hello こんにちは"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_english_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["EN".to_string()]);

        assert!(filter.should_translate("Hello World"));
        assert!(filter.should_translate("你好 Hello"));
        assert!(!filter.should_translate("你好世界"));
    }

    #[test]
    fn test_empty_languages() {
        let filter = LanguageOnlyFilter::new(vec![]);

        assert!(!filter.should_translate("Hello World"));
        assert!(!filter.should_translate("你好世界"));
    }

    #[test]
    fn test_mixed_content() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // Any Chinese content should be extracted
        assert!(filter.should_translate("TODO: 修复这个bug"));
        assert!(filter.should_translate("Copyright © 2024 - 版权所有"));
        assert!(filter.should_translate("Error: 参数错误"));
        assert!(filter.should_translate("https://example.com/你好"));
    }
}
```

#### 2.2.3 修改 CompositeFilter

在 `src/parser/filtering/composite.rs` 中修改：

```rust
use crate::parser::filtering::checks::language::LanguageOnlyFilter;

/// Composite filter that orchestrates all filter checks
pub struct CompositeFilter {
    length: LengthFilter,
    language: LanguageFilter,
    pattern: PatternFilter,
    content: ContentFilter,
    language_only: Option<LanguageOnlyFilter>,  // 新增
}

impl CompositeFilter {
    pub fn new(config: FilterConfig) -> crate::core::error::Result<Self> {
        // 创建语言专用过滤器（如果启用）
        let language_only = if config.force_extract_by_language {
            if config.extract_languages.is_empty() {
                tracing::warn!(
                    "force_extract_by_language is enabled but extract_languages is empty, ignoring"
                );
                None
            } else {
                Some(LanguageOnlyFilter::new(config.extract_languages))
            }
        } else {
            None
        };

        Ok(Self {
            length: LengthFilter::new(&config),
            language: LanguageFilter::new(&config),
            pattern: PatternFilter::new(&config)?,
            content: ContentFilter::new(),
            language_only,
        })
    }

    // ... 其他方法保持不变 ...
}

impl Filter for CompositeFilter {
    fn should_translate(&self, text: &str) -> bool {
        // 如果启用了强制语言提取，只使用语言过滤
        if let Some(ref lang_filter) = self.language_only {
            return lang_filter.should_translate(text);
        }

        // 否则使用完整的过滤链
        if !self.length.should_translate(text) {
            return false;
        }

        if !self.language.should_translate(text) {
            return false;
        }

        if !self.pattern.should_translate(text) {
            return false;
        }

        if !self.content.should_translate(text) {
            return false;
        }

        debug!(text = %text, "Text passed all filter checks");
        true
    }

    fn name(&self) -> &str {
        if self.language_only.is_some() {
            "CompositeFilter (LanguageOnly mode)"
        } else {
            "CompositeFilter"
        }
    }
}
```

### 2.3 配置加载

#### 2.3.1 更新项目配置结构

在 `src/config/project.rs` 中更新 `FilterConfig`：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    // ... 现有字段 ...

    /// 强制提取包含特定语言字符的文本
    #[serde(default)]
    pub force_extract_by_language: bool,

    /// 要提取的语言列表
    #[serde(default)]
    pub extract_languages: Vec<String>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            // ... 现有字段 ...
            force_extract_by_language: false,
            extract_languages: Vec::new(),
        }
    }
}
```

#### 2.3.2 更新配置转换

在 `src/parser/filtering/composite.rs` 中更新 `from_project_config` 方法：

```rust
pub fn from_project_config(
    config: &crate::config::project::FilterConfig,
    translate_config: &crate::config::project::TranslateConfig,
) -> crate::core::error::Result<CompositeFilter> {
    let filter_config = FilterConfig {
        source_langs: translate_config.source_langs.clone(),
        target_lang: translate_config.target_lang.clone(),
        exclude_keywords: config.exclude_keywords.clone(),
        exclude_patterns: config.exclude_patterns.clone(),
        include_patterns: config.include_patterns.clone(),
        max_length: if config.max_length == 0 {
            100000
        } else {
            config.max_length
        },
        allow_placeholders: config.allow_placeholders,
        detect_code_patterns: config.detect_code_patterns,
        // 新增字段
        force_extract_by_language: config.force_extract_by_language,
        extract_languages: config.extract_languages.clone(),
    };
    CompositeFilter::new(filter_config)
}

pub fn from_project_config_with_translator(
    project_config: &crate::config::project::FilterConfig,
    translate_config: &crate::config::project::TranslateConfig,
    translator_max_length: Option<usize>,
) -> crate::core::error::Result<CompositeFilter> {
    let max_length = match (project_config.max_length, translator_max_length) {
        (0, None) => 100000,
        (0, Some(translator_max)) => translator_max,
        (project_max, None) => project_max,
        (project_max, Some(translator_max)) => project_max.min(translator_max),
    };

    let filter_config = FilterConfig {
        source_langs: translate_config.source_langs.clone(),
        target_lang: translate_config.target_lang.clone(),
        exclude_keywords: project_config.exclude_keywords.clone(),
        exclude_patterns: project_config.exclude_patterns.clone(),
        include_patterns: project_config.include_patterns.clone(),
        max_length,
        allow_placeholders: project_config.allow_placeholders,
        detect_code_patterns: project_config.detect_code_patterns,
        // 新增字段
        force_extract_by_language: project_config.force_extract_by_language,
        extract_languages: project_config.extract_languages.clone(),
    };
    CompositeFilter::new(filter_config)
}
```

### 2.4 使用示例

#### 2.4.1 提取所有中文内容（最常见场景）

```toml
[filter]
force_extract_by_language = true
extract_languages = ["ZH"]

# 注意：启用此选项后，以下配置将被忽略
# exclude_keywords = ["TODO", "FIXME"]  # 忽略
# exclude_patterns = ["https?://[^\s]+"]  # 忽略
# min_length = 2  # 忽略
# allow_placeholders = false  # 忽略
# detect_code_patterns = true  # 忽略
```

**代码示例**：
```rust
// 用户提示：请输入用户名
let username = input("请输入用户名: ");

// TODO: 修复中文编码问题
let data = load_data("文件路径");

// Error: 参数错误
if username.is_empty() {
    println("用户名不能为空");
}

// https://github.com/project/中文文档
let url = "https://github.com/project/中文文档";
```

**提取结果**：
| 文本 | 状态 | 说明 |
|------|------|------|
| "用户提示：请输入用户名" | ✅ 提取 | 含中文 |
| "请输入用户名:" | ✅ 提取 | 含中文 |
| "TODO: 修复中文编码问题" | ✅ 提取 | 含中文（忽略关键词过滤） |
| "文件路径" | ✅ 提取 | 含中文 |
| "Error: 参数错误" | ✅ 提取 | 含中文 |
| "用户名不能为空" | ✅ 提取 | 含中文 |
| "https://github.com/project/中文文档" | ✅ 提取 | 含中文（忽略 URL 过滤） |
| "username" | ❌ 跳过 | 不含中文 |
| "println" | ❌ 跳过 | 不含中文 |

#### 2.4.2 提取所有中日韩内容

```toml
[filter]
force_extract_by_language = true
extract_languages = ["ZH", "JA", "KO"]
```

**代码示例**：
```rust
// 中国用户提示
let msg_zh = "请输入用户名";

// 日本用户提示
let msg_ja = "ユーザー名を入力してください";

// 韩国用户提示
let msg_ko = "사용자 이름을 입력하세요";

// 英文提示
let msg_en = "Enter username";
```

**提取结果**：
| 文本 | 状态 | 说明 |
|------|------|------|
| "请输入用户名" | ✅ 提取 | 含中文 |
| "ユーザー名を入力してください" | ✅ 提取 | 含日文 |
| "사용자 이름을 입력하세요" | ✅ 提取 | 含韩文 |
| "Enter username" | ❌ 跳过 | 不含目标语言 |

#### 2.4.3 提取所有英文内容（反向场景）

```toml
[filter]
force_extract_by_language = true
extract_languages = ["EN"]
```

**代码示例**：
```rust
// 用户提示：Please enter username
let username = input("Please enter username: ");

// TODO: Fix the bug
let data = load_data("file path");

// Error: Invalid parameter
if username.is_empty() {
    println("Username cannot be empty");
}

// https://github.com/project
let url = "https://github.com/project";
```

**提取结果**：
| 文本 | 状态 | 说明 |
|------|------|------|
| "Please enter username:" | ✅ 提取 | 含英文 |
| "TODO: Fix the bug" | ✅ 提取 | 含英文 |
| "file path" | ✅ 提取 | 含英文 |
| "Error: Invalid parameter" | ✅ 提取 | 含英文 |
| "Username cannot be empty" | ✅ 提取 | 含英文 |
| "https://github.com/project" | ✅ 提取 | 含英文 |
| "用户提示：" | ❌ 跳过 | 不含英文 |
| "input" | ❌ 跳过 | 不含英文 |

#### 2.4.4 正常模式（默认，完整过滤链）

```toml
[filter]
force_extract_by_language = false  # 或不设置
extract_languages = []  # 或不设置

# 以下配置生效
exclude_keywords = ["TODO", "FIXME", "HACK"]
exclude_patterns = ["https?://[^\s]+", "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"]
min_length = 2
allow_placeholders = false
detect_code_patterns = true
```

**代码示例**：
```rust
// TODO: 修复这个bug
let data = load_data("file");

// 用户提示：请输入用户名
let username = input("请输入用户名:");

// https://github.com/project
let url = "https://github.com/project";

// Error: 参数错误
if username.is_empty() {
    println("用户名不能为空");
}

// Hello %s
let msg = format!("Hello %s", username);
```

**提取结果**：
| 文本 | 状态 | 说明 |
|------|------|------|
| "TODO: 修复这个bug" | ❌ 跳过 | 关键词过滤（TODO） |
| "请输入用户名:" | ✅ 提取 | 通过所有检查 |
| "https://github.com/project" | ❌ 跳过 | URL 过滤 |
| "参数错误" | ✅ 提取 | 通过所有检查 |
| "用户名不能为空" | ✅ 提取 | 通过所有检查 |
| "Hello %s" | ❌ 跳过 | 占位符过滤 |
| "file" | ✅ 提取 | 通过所有检查 |

#### 2.4.5 对比：正常模式 vs 强制提取模式

**场景：提取以下代码中的中文内容**

```rust
// TODO: 修复中文编码问题
let msg = "请输入用户名";
let url = "https://example.com/中文文档";
if msg.is_empty() {
    println("用户名不能为空");
}
```

| 配置模式 | 提取内容 | 说明 |
|---------|---------|------|
| **正常模式** | `请输入用户名`, `用户名不能为空` | `TODO` 和 URL 被过滤 |
| **强制提取** | `修复中文编码问题`, `请输入用户名`, `中文文档`, `用户名不能为空` | 提取所有含中文的文本 |

**选择建议**：
- ✅ **正常模式**：需要精细控制，避免提取不需要的内容
- ✅ **强制提取**：需要提取所有目标语言内容，不关心其他过滤条件

---

## 3. 实现步骤

### 阶段 1: 数据结构扩展（预计 2-3 小时）

#### 1.1 扩展 FilterConfig
**文件**: `src/parser/filtering/config.rs`

**具体操作**:
1. 在 `FilterConfig` 结构体中添加两个字段：
   ```rust
   #[serde(default)]
   pub force_extract_by_language: bool,

   #[serde(default)]
   pub extract_languages: Vec<String>,
   ```

2. 更新 `Default` 实现：
   ```rust
   impl Default for FilterConfig {
       fn default() -> Self {
           Self {
               // ... 现有字段 ...
               force_extract_by_language: false,
               extract_languages: Vec::new(),
           }
       }
   }
   ```

#### 1.2 创建 LanguageOnlyFilter
**文件**: `src/parser/filtering/checks/language/language_only.rs`（新建）

**具体操作**:
1. 创建新文件，包含完整的 `LanguageOnlyFilter` 实现
2. 实现核心方法：
   - `new(languages: Vec<String>) -> Self`
   - `contains_target_language(&self, text: &str) -> bool`
   - `should_translate(&self, text: &str) -> bool`
   - `name(&self) -> &str`
3. 添加完整的单元测试（约 10-15 个测试用例）

#### 1.3 更新模块导出
**文件**: `src/parser/filtering/checks/language/mod.rs`

**具体操作**:
1. 添加模块声明：
   ```rust
   mod language_only;
   ```
2. 导出类型：
   ```rust
   pub use language_only::LanguageOnlyFilter;
   ```

### 阶段 2: 过滤器集成（预计 1-2 小时）

#### 2.1 修改 CompositeFilter
**文件**: `src/parser/filtering/composite.rs`

**具体操作**:
1. 导入 `LanguageOnlyFilter`：
   ```rust
   use crate::parser::filtering::checks::language::LanguageOnlyFilter;
   ```

2. 在 `CompositeFilter` 结构体中添加字段：
   ```rust
   pub struct CompositeFilter {
       length: LengthFilter,
       language: LanguageFilter,
       pattern: PatternFilter,
       content: ContentFilter,
       language_only: Option<LanguageOnlyFilter>,  // 新增
   }
   ```

3. 修改 `new` 方法：
   ```rust
   pub fn new(config: FilterConfig) -> crate::core::error::Result<Self> {
       let language_only = if config.force_extract_by_language {
           if config.extract_languages.is_empty() {
               tracing::warn!(
                   "force_extract_by_language is enabled but extract_languages is empty, ignoring"
               );
               None
           } else {
               Some(LanguageOnlyFilter::new(config.extract_languages))
           }
       } else {
           None
       };

       Ok(Self {
           length: LengthFilter::new(&config),
           language: LanguageFilter::new(&config),
           pattern: PatternFilter::new(&config)?,
           content: ContentFilter::new(),
           language_only,
       })
   }
   ```

4. 修改 `should_translate` 方法：
   ```rust
   fn should_translate(&self, text: &str) -> bool {
       // 如果启用了强制语言提取，只使用语言过滤
       if let Some(ref lang_filter) = self.language_only {
           return lang_filter.should_translate(text);
       }

       // 否则使用完整的过滤链
       if !self.length.should_translate(text) {
           return false;
       }

       if !self.language.should_translate(text) {
           return false;
       }

       if !self.pattern.should_translate(text) {
           return false;
       }

       if !self.content.should_translate(text) {
           return false;
       }

       debug!(text = %text, "Text passed all filter checks");
       true
   }
   ```

5. 更新 `name` 方法：
   ```rust
   fn name(&self) -> &str {
       if self.language_only.is_some() {
           "CompositeFilter (LanguageOnly mode)"
       } else {
           "CompositeFilter"
       }
   }
   ```

6. 添加集成测试（约 5-8 个测试用例）

### 阶段 3: 配置加载（预计 1 小时）

#### 3.1 更新项目配置结构
**文件**: `src/config/project.rs`

**具体操作**:
1. 在 `FilterConfig` 结构体中添加字段（与阶段 1.1 相同）：
   ```rust
   #[serde(default)]
   pub force_extract_by_language: bool,

   #[serde(default)]
   pub extract_languages: Vec<String>,
   ```

2. 更新 `Default` 实现

#### 3.2 更新配置转换方法
**文件**: `src/parser/filtering/composite.rs`

**具体操作**:
1. 修改 `from_project_config` 方法：
   ```rust
   pub fn from_project_config(
       config: &crate::config::project::FilterConfig,
       translate_config: &crate::config::project::TranslateConfig,
   ) -> crate::core::error::Result<CompositeFilter> {
       let filter_config = FilterConfig {
           source_langs: translate_config.source_langs.clone(),
           target_lang: translate_config.target_lang.clone(),
           exclude_keywords: config.exclude_keywords.clone(),
           exclude_patterns: config.exclude_patterns.clone(),
           include_patterns: config.include_patterns.clone(),
           max_length: if config.max_length == 0 { 100000 } else { config.max_length },
           allow_placeholders: config.allow_placeholders,
           detect_code_patterns: config.detect_code_patterns,
           // 新增字段
           force_extract_by_language: config.force_extract_by_language,
           extract_languages: config.extract_languages.clone(),
       };
       CompositeFilter::new(filter_config)
   }
   ```

2. 修改 `from_project_config_with_translator` 方法（类似）

3. 添加配置测试（约 3-5 个测试用例）

### 阶段 4: 端到端测试（预计 2-3 小时）

#### 4.1 创建测试配置文件
**文件**: `tests/fixtures/force_language_extraction/.translator.toml`（新建）

**具体操作**:
1. 创建测试配置，包含强制提取模式
2. 创建测试配置，包含正常模式（对比）
3. 创建测试代码文件，包含多种语言和场景

#### 4.2 编写端到端测试
**文件**: `tests/parser_integration_tests.rs`

**具体操作**:
1. 添加测试函数 `test_force_extract_chinese`
2. 添加测试函数 `test_force_extract_multiple_languages`
3. 添加测试函数 `test_normal_mode_vs_force_extract`
4. 添加测试函数 `test_force_extract_edge_cases`
5. 验证测试通过

#### 4.3 更新示例配置
**文件**: `.translator.toml` 和 `e2e/.translator.toml`

**具体操作**:
1. 在配置文件中添加注释说明新功能
2. 提供使用示例

### 阶段 5: 文档和验证（预计 1 小时）

#### 5.1 更新文档
**具体操作**:
1. 更新 `README.md`（如果需要）
2. 确保代码注释完整
3. 添加使用示例

#### 5.2 验证
**具体操作**:
1. 运行所有测试：`cargo test`
2. 运行 clippy：`cargo clippy`
3. 运行格式化：`cargo fmt`
4. 编译检查：`cargo check`
5. 手动测试配置文件加载

---

## 4. 实现检查清单

### 代码实现
- [ ] `src/parser/filtering/config.rs` - 添加新字段
- [ ] `src/parser/filtering/checks/language/language_only.rs` - 创建新文件
- [ ] `src/parser/filtering/checks/language/mod.rs` - 导出类型
- [ ] `src/parser/filtering/composite.rs` - 集成 LanguageOnlyFilter
- [ ] `src/config/project.rs` - 更新项目配置

### 测试
- [ ] `LanguageOnlyFilter` 单元测试（10+ 测试用例）
- [ ] `CompositeFilter` 集成测试（5+ 测试用例）
- [ ] 配置加载测试（3+ 测试用例）
- [ ] 端到端测试（4+ 测试用例）
- [ ] 边界情况测试

### 文档
- [ ] 代码注释完整
- [ ] 使用示例清晰
- [ ] 配置文件注释更新
- [ ] 设计文档（本文档）完整

### 验证
- [ ] `cargo test` - 所有测试通过
- [ ] `cargo clippy` - 无警告
- [ ] `cargo fmt` - 格式正确
- [ ] `cargo check` - 编译通过
- [ ] 手动测试配置文件加载
- [ ] 向后兼容性验证

---

## 4. 测试计划

### 4.1 单元测试

#### 4.1.1 LanguageOnlyFilter 测试

**文件**: `src/parser/filtering/checks/language/language_only.rs`

**测试用例**:

1. **test_chinese_extraction**
   ```rust
   #[test]
   fn test_chinese_extraction() {
       let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

       assert!(filter.should_translate("你好世界"));
       assert!(filter.should_translate("Hello 你好"));
       assert!(filter.should_translate("你好Hello"));
       assert!(!filter.should_translate("Hello World"));
   }
   ```
   **预期**: ✅ 通过

2. **test_japanese_extraction**
   ```rust
   #[test]
   fn test_japanese_extraction() {
       let filter = LanguageOnlyFilter::new(vec!["JA".to_string()]);

       assert!(filter.should_translate("こんにちは"));
       assert!(filter.should_translate("カタカナ"));
       assert!(filter.should_translate("Hello こんにちは"));
       assert!(!filter.should_translate("Hello World"));
   }
   ```
   **预期**: ✅ 通过

3. **test_korean_extraction**
   ```rust
   #[test]
   fn test_korean_extraction() {
       let filter = LanguageOnlyFilter::new(vec!["KO".to_string()]);

       assert!(filter.should_translate("안녕하세요"));
       assert!(filter.should_translate("Hello 안녕하세요"));
       assert!(!filter.should_translate("Hello World"));
   }
   ```
   **预期**: ✅ 通过

4. **test_multiple_languages**
   ```rust
   #[test]
   fn test_multiple_languages() {
       let filter = LanguageOnlyFilter::new(vec!["ZH".to_string(), "JA".to_string()]);

       assert!(filter.should_translate("你好世界"));
       assert!(filter.should_translate("こんにちは"));
       assert!(filter.should_translate("Hello 你好"));
       assert!(filter.should_translate("Hello こんにちは"));
       assert!(!filter.should_translate("Hello World"));
   }
   ```
   **预期**: ✅ 通过

5. **test_english_extraction**
   ```rust
   #[test]
   fn test_english_extraction() {
       let filter = LanguageOnlyFilter::new(vec!["EN".to_string()]);

       assert!(filter.should_translate("Hello World"));
       assert!(filter.should_translate("你好 Hello"));
       assert!(!filter.should_translate("你好世界"));
   }
   ```
   **预期**: ✅ 通过

6. **test_empty_languages**
   ```rust
   #[test]
   fn test_empty_languages() {
       let filter = LanguageOnlyFilter::new(vec![]);

       assert!(!filter.should_translate("Hello World"));
       assert!(!filter.should_translate("你好世界"));
   }
   ```
   **预期**: ✅ 通过

7. **test_mixed_content_with_keywords**
   ```rust
   #[test]
   fn test_mixed_content_with_keywords() {
       let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

       // 应该提取，即使包含 TODO、Copyright 等关键词
       assert!(filter.should_translate("TODO: 修复这个bug"));
       assert!(filter.should_translate("Copyright © 2024 - 版权所有"));
       assert!(filter.should_translate("Error: 参数错误"));
   }
   ```
   **预期**: ✅ 通过（验证跳过关键词过滤）

8. **test_mixed_content_with_urls**
   ```rust
   #[test]
   fn test_mixed_content_with_urls() {
       let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

       // 应该提取，即使包含 URL
       assert!(filter.should_translate("https://example.com/你好"));
       assert!(filter.should_translate("Visit https://example.com/文档"));
   }
   ```
   **预期**: ✅ 通过（验证跳过 URL 过滤）

9. **test_mixed_content_with_placeholders**
   ```rust
   #[test]
   fn test_mixed_content_with_placeholders() {
       let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

       // 应该提取，即使包含占位符
       assert!(filter.should_translate("Hello %s, 你好"));
       assert!(filter.should_translate("Value: {name}, 值"));
   }
   ```
   **预期**: ✅ 通过（验证跳过占位符过滤）

10. **test_mixed_content_with_code_patterns**
    ```rust
    #[test]
    fn test_mixed_content_with_code_patterns() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // 应该提取，即使包含代码模式
        assert!(filter.should_translate("obj.method() 你好"));
        assert!(filter.should_translate("func(arg) 参数错误"));
    }
    ```
    **预期**: ✅ 通过（验证跳过代码模式过滤）

11. **test_chinese_variants**
    ```rust
    #[test]
    fn test_chinese_variants() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // 简体、繁体都应该被识别
        assert!(filter.should_translate("你好世界"));  // 简体
        assert!(filter.should_translate("你好世界"));  // 繁体
    }
    ```
    **预期**: ✅ 通过

12. **test_arabic_extraction**
    ```rust
    #[test]
    fn test_arabic_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["AR".to_string()]);

        assert!(filter.should_translate("مرحبا بالعالم"));
        assert!(filter.should_translate("Hello مرحبا"));
        assert!(!filter.should_translate("Hello World"));
    }
    ```
    **预期**: ✅ 通过

13. **test_cyrillic_extraction**
    ```rust
    #[test]
    fn test_cyrillic_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["RU".to_string()]);

        assert!(filter.should_translate("Привет мир"));
        assert!(filter.should_translate("Hello Привет"));
        assert!(!filter.should_translate("Hello World"));
    }
    ```
    **预期**: ✅ 通过

14. **test_unknown_language_allowed**
    ```rust
    #[test]
    fn test_unknown_language_allowed() {
        let filter = LanguageOnlyFilter::new(vec!["UNKNOWN".to_string()]);

        // 未知语言应该允许通过
        assert!(filter.should_translate("Hello World"));
        assert!(filter.should_translate("你好世界"));
    }
    ```
    **预期**: ✅ 通过

15. **test_empty_text**
    ```rust
    #[test]
    fn test_empty_text() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        assert!(!filter.should_translate(""));
        assert!(!filter.should_translate("   "));
    }
    ```
    **预期**: ✅ 通过

#### 4.1.2 CompositeFilter 测试

**文件**: `src/parser/filtering/composite.rs`

**测试用例**:

1. **test_force_extract_mode_enabled**
   ```rust
   #[test]
   fn test_force_extract_mode_enabled() {
       let config = FilterConfig {
           force_extract_by_language: true,
           extract_languages: vec!["ZH".to_string()],
           ..Default::default()
       };
       let filter = CompositeFilter::new(config).unwrap();

       // 应该提取所有含中文的文本，忽略其他过滤
       assert!(filter.should_translate("TODO: 修复中文bug"));
       assert!(filter.should_translate("https://example.com/你好"));
       assert!(filter.should_translate("Hello %s 你好"));
   }
   ```
   **预期**: ✅ 通过

2. **test_force_extract_mode_disabled**
   ```rust
   #[test]
   fn test_force_extract_mode_disabled() {
       let config = FilterConfig {
           force_extract_by_language: false,
           exclude_keywords: vec!["TODO".to_string()],
           ..Default::default()
       };
       let filter = CompositeFilter::new(config).unwrap();

       // 应该遵循完整的过滤链
       assert!(!filter.should_translate("TODO: fix this"));
       assert!(filter.should_translate("Hello World"));
   }
   ```
   **预期**: ✅ 通过

3. **test_empty_languages_warning**
   ```rust
   #[test]
   fn test_empty_languages_warning() {
       let config = FilterConfig {
           force_extract_by_language: true,
           extract_languages: vec![],
           ..Default::default()
       };
       let filter = CompositeFilter::new(config).unwrap();

       // 应该记录警告，并使用完整过滤链
       assert!(!filter.should_translate("Hello World"));
   }
   ```
   **预期**: ✅ 通过（验证警告日志）

4. **test_name_with_force_extract**
   ```rust
   #[test]
   fn test_name_with_force_extract() {
       let config = FilterConfig {
           force_extract_by_language: true,
           extract_languages: vec!["ZH".to_string()],
           ..Default::default()
       };
       let filter = CompositeFilter::new(config).unwrap();

       assert_eq!(filter.name(), "CompositeFilter (LanguageOnly mode)");
   }
   ```
   **预期**: ✅ 通过

### 4.2 集成测试

#### 4.2.1 配置加载测试

**文件**: `tests/config_validation.rs`

**测试用例**:

1. **test_force_extract_config_loading**
   ```rust
   #[test]
   fn test_force_extract_config_loading() {
       let config_str = r#"
       [filter]
       force_extract_by_language = true
       extract_languages = ["ZH", "JA"]
       "#;

       let config: FilterConfig = toml::from_str(config_str).unwrap();
       assert!(config.force_extract_by_language);
       assert_eq!(config.extract_languages, vec!["ZH", "JA"]);
   }
   ```
   **预期**: ✅ 通过

2. **test_default_config_values**
   ```rust
   #[test]
   fn test_default_config_values() {
       let config = FilterConfig::default();
       assert!(!config.force_extract_by_language);
       assert!(config.extract_languages.is_empty());
   }
   ```
   **预期**: ✅ 通过

3. **test_toml_serialization**
   ```rust
   #[test]
   fn test_toml_serialization() {
       let config = FilterConfig {
           force_extract_by_language: true,
           extract_languages: vec!["ZH".to_string()],
           ..Default::default()
       };

       let toml_str = toml::to_string(&config).unwrap();
       let deserialized: FilterConfig = toml::from_str(&toml_str).unwrap();

       assert_eq!(deserialized.force_extract_by_language, true);
       assert_eq!(deserialized.extract_languages, vec!["ZH"]);
   }
   ```
   **预期**: ✅ 通过

#### 4.2.2 Parser Coordinator 测试

**文件**: `tests/parser_integration_tests.rs`

**测试用例**:

1. **test_parser_with_force_extract**
   ```rust
   #[test]
   fn test_parser_with_force_extract() {
       let content = r#"
       // TODO: 修复中文bug
       let msg = "请输入用户名";
       "#;

       let file = create_test_file(content, "test.rs");
       let config = create_config_with_force_extract(vec!["ZH"]);
       let coordinator = ParserCoordinator::from_project_config(config).unwrap();

       let units = coordinator.parse_file(&file).unwrap();

       // 应该提取所有含中文的文本
       assert!(!units.is_empty());
       assert!(units.iter().any(|u| u.content.contains("修复中文bug")));
       assert!(units.iter().any(|u| u.content.contains("请输入用户名")));
   }
   ```
   **预期**: ✅ 通过

### 4.3 端到端测试

**文件**: `tests/e2e_tests.rs`

**测试用例**:

1. **test_e2e_force_extract_chinese**
   ```rust
   #[test]
   #[ignore] // 需要实际文件系统
   fn test_e2e_force_extract_chinese() {
       // 创建测试文件
       // 运行翻译工具
       // 验证提取结果
   }
   ```
   **预期**: ✅ 通过

2. **test_e2e_force_extract_multiple_languages**
   ```rust
   #[test]
   #[ignore]
   fn test_e2e_force_extract_multiple_languages() {
       // 测试提取多种语言
   }
   ```
   **预期**: ✅ 通过

3. **test_e2e_normal_mode_vs_force_extract**
   ```rust
   #[test]
   #[ignore]
   fn test_e2e_normal_mode_vs_force_extract() {
       // 对比两种模式的结果
   }
   ```
   **预期**: ✅ 通过

4. **test_e2e_edge_cases**
   ```rust
   #[test]
   #[ignore]
   fn test_e2e_edge_cases() {
       // 测试边界情况
       // - 空文件
       // - 只有标点符号
       // - 混合编码
   }
   ```
   **预期**: ✅ 通过

### 4.4 性能测试

**文件**: `benches/filter_performance.rs`（新建）

**测试用例**:

1. **bench_force_extract_performance**
   ```rust
   #[bench]
   fn bench_force_extract_performance(b: &mut Bencher) {
       let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);
       let text = "这是一个测试字符串";

       b.iter(|| {
           filter.should_translate(text);
       });
   }
   ```
   **预期**: 纳秒级响应时间（< 100ns）

2. **bench_force_extract_vs_normal_mode**
   ```rust
   #[bench]
   fn bench_force_extract_vs_normal_mode(b: &mut Bencher) {
       let force_filter = create_force_extract_filter();
       let normal_filter = create_normal_filter();
       let text = "这是一个测试字符串";

       b.iter(|| {
           black_box(force_filter.should_translate(text));
           black_box(normal_filter.should_translate(text));
       });
   }
   ```
   **预期**: 强制提取模式比正常模式快 2-3 倍

3. **bench_large_text**
   ```rust
   #[bench]
   fn bench_large_text(b: &mut Bencher) {
       let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);
       let text = "这是一个非常长的测试字符串...".repeat(1000);

       b.iter(|| {
           filter.should_translate(&text);
       });
   }
   ```
   **预期**: 性能不受文本长度影响（O(32)）

### 4.5 内存使用测试

**测试用例**:

1. **test_memory_usage**
   ```rust
   #[test]
   fn test_memory_usage() {
       let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);
       let before = memory_usage();

       for _ in 0..10000 {
           filter.should_translate("测试字符串");
       }

       let after = memory_usage();
       assert!(after - before < 1024); // 内存增长应小于 1KB
   }
   ```
   **预期**: ✅ 通过

---

## 5. 测试覆盖率目标

| 模块 | 目标覆盖率 | 当前覆盖率 | 状态 |
|------|-----------|-----------|------|
| LanguageOnlyFilter | 95%+ | 0% | 待实现 |
| CompositeFilter | 90%+ | 85% | 需提升 |
| FilterConfig | 100% | 80% | 需提升 |
| 端到端测试 | 80%+ | 70% | 需提升 |

**总体目标**: 90%+ 测试覆盖率

---

## 5. 文件清单

### 需要修改的文件
- `src/parser/filtering/config.rs` - 添加新字段
- `src/parser/filtering/composite.rs` - 集成 LanguageOnlyFilter
- `src/parser/filtering/checks/language/mod.rs` - 导出 LanguageOnlyFilter
- `src/config/project.rs` - 更新项目配置

### 需要创建的文件
- `src/parser/filtering/checks/language/language_only.rs` - LanguageOnlyFilter 实现

### 需要更新的测试文件
- `tests/parser_integration_tests.rs` - 添加端到端测试
- `tests/config_validation.rs` - 添加配置测试

### 配置文件示例
- `e2e/.translator.toml` - 更新示例配置

---

## 6. 风险和注意事项

### 6.1 配置风险

**风险**: 用户可能误解此选项的作用

**缓解**:
- 在文档中明确说明此选项会跳过所有其他过滤
- 在配置文件中添加详细注释
- 在日志中输出警告信息

**风险**: 用户可能提取过多不需要的内容

**缓解**:
- 默认关闭此选项
- 提供清晰的使用示例
- 建议用户先使用 dry_run 模式测试

### 6.2 性能风险

**风险**: 在某些极端情况下可能影响性能

**缓解**:
- 使用现有的 `QuickDetector`（O(32)）
- 不增加额外的正则匹配
- 避免不必要的内存分配

### 6.3 向后兼容风险

**风险**: 新增字段可能影响现有配置

**缓解**:
- 新字段默认值为空或 false
- 使用 `#[serde(default)]` 确保兼容性
- 现有配置文件无需修改即可工作

### 6.4 测试风险

**风险**: 新功能可能破坏现有测试

**缓解**:
- 默认关闭新功能
- 现有测试不受影响
- 新测试覆盖新功能

---

## 7. 关键设计原则

### 7.1 性能优先
- 使用现有的 `QuickDetector`，保持 O(32) 时间复杂度
- 避免额外的正则匹配
- 最小化内存分配

### 7.2 向后兼容
- 新功能默认关闭
- 不影响现有配置
- 不影响现有行为

### 7.3 类型安全
- 使用 `Option` 表示可选状态
- 明确的语言代码枚举（如果需要）
- 强类型检查

### 7.4 可扩展性
- 易于添加新的语言代码
- 易于扩展过滤逻辑
- 支持未来的高级过滤选项

### 7.5 可测试性
- 独立的单元测试
- 清晰的测试用例
- 覆盖所有边界情况

---

## 8. 与现有功能的对比

### 8.1 与 AUTO 模式的区别

| 特性 | AUTO 模式 | 强制提取模式 |
|------|-----------|-------------|
| 语言检测 | ✅ 启用 | ✅ 启用 |
| 长度过滤 | ✅ 启用 | ❌ 跳过 |
| 模式过滤 | ✅ 启用 | ❌ 跳过 |
| 占位符过滤 | ✅ 启用 | ❌ 跳过 |
| 代码模式过滤 | ✅ 启用 | ❌ 跳过 |
| 目标语言检查 | ✅ 启用 | ❌ 跳过 |
| 性能 | O(32) + O(n) | O(32) |

### 8.2 与自定义正则模式的对比

| 特性 | 自定义正则模式 | 强制提取模式 |
|------|---------------|-------------|
| 配置复杂度 | ⚠️ 高（需要编写正则） | ✅ 低（仅需语言代码） |
| 维护性 | ⚠️ 低（正则难维护） | ✅ 高（语言代码直观） |
| 灵活性 | ✅ 高（可自定义） | ⚠️ 中（预定义语言） |
| 性能 | O(n) 正则匹配 | O(32) 字符检查 |
| 学习曲线 | ⚠️ 高（需要正则知识） | ✅ 低（语言代码简单） |

### 8.3 使用建议

| 场景 | 推荐方案 |
|------|---------|
| 正常翻译，需要精细控制 | AUTO 模式 + 完整过滤 |
| 提取所有特定语言内容 | 强制提取模式 |
| 提取特定模式的文本 | 自定义正则模式 |
| 混合需求 | 组合使用 |

---

## 9. 预期效果

实施此方案后，用户将能够：

1. **快速提取特定语言内容**: 无需编写复杂正则，仅通过语言代码即可提取
2. **避免配置复杂性**: 不需要配置多个排除/包含模式
3. **保持高性能**: O(32) 的字符检查，不影响整体性能
4. **灵活切换**: 根据场景在正常模式和强制提取模式之间切换
5. **向后兼容**: 现有用户无需修改配置即可继续使用

### 典型使用场景

**场景 1: 代码库国际化准备**
```toml
# 提取所有中文注释和字符串，准备翻译
[filter]
force_extract_by_language = true
extract_languages = ["ZH"]
```

**场景 2: 多语言代码审计**
```toml
# 提取所有非英文内容，进行审计
[filter]
force_extract_by_language = true
extract_languages = ["ZH", "JA", "KO", "AR"]
```

**场景 3: 批量处理混合内容**
```toml
# 处理包含多种语言的内容，不关心其他过滤条件
[filter]
force_extract_by_language = true
extract_languages = ["ZH"]
```

---

## 10. 未来扩展

### 10.1 可能的增强功能

1. **更细粒度的语言控制**
   - 支持繁体/简体中文区分
   - 支持特定方言
   - 支持混合语言比例阈值

2. **性能优化**
   - 缓存检测结果
   - 批量检测
   - 并行检测

3. **高级过滤**
   - 语言密度过滤（文本中特定语言字符的比例）
   - 上下文感知过滤（根据代码上下文决定是否提取）
   - 语义过滤（NLP 辅助）

4. **统计和报告**
   - 记录提取的语言分布
   - 生成语言统计报告
   - 可视化语言分布

### 10.2 与其他功能的集成

1. **与缓存系统集成**
   - 基于语言特征的缓存策略
   - 语言特定的缓存键

2. **与翻译器集成**
   - 根据检测到的语言自动选择翻译器
   - 语言特定的翻译策略

3. **与报告系统集成**
   - 语言分布统计
   - 提取效率报告

---

## 11. 总结

本方案详细分析了当前语言检查机制的局限性，提出了"强制语言提取"功能的完整设计方案。通过添加配置选项 `force_extract_by_language` 和 `extract_languages`，用户可以快速提取包含特定语言字符的所有文本，无需编写复杂的正则表达式。

**核心优势**：
- ✅ 简化配置：仅需语言代码，无需复杂正则
- ✅ 高性能：O(32) 字符检查，不影响性能
- ✅ 向后兼容：默认关闭，不影响现有用户
- ✅ 灵活切换：根据场景选择不同模式
- ✅ 易于扩展：支持未来增强功能

**实施优先级**：中高优先级
- 对于需要快速提取特定语言内容的用户，此功能非常有用
- 实现相对简单，风险可控
- 可以显著提升用户体验

此方案将在保持系统稳定性和性能的同时，为用户提供更强大、更灵活的语言内容提取能力。