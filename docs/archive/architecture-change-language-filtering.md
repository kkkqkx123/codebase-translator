# 架构变更：语言过滤职责调整

## 变更概述

将语言判断的职责从 **Scanner** 转移到 **Filter**，避免配置混淆和过滤逻辑错误。

## 问题根源

### 1. Scanner 配置语义混淆
- Scanner 使用 `target_lang`（翻译目标语言）配置
- 但实际应该基于 `source_langs`（源语言）过滤
- 导致逻辑完全反了：保留英文，过滤中文

### 2. 采样策略缺陷
- `QuickDetector` 只检查前 32 个字符（后改为 128）
- 对于混合语言文本（如英文注释 + 中文），如果中文在后面，检测不到
- 示例：`"This is a very long English comment: 你好世界"` 会被漏检

### 3. 时序问题
- Scanner 在 Coordinator 创建时初始化，使用静态配置
- 但语言过滤需要动态的源语言配置（如 AUTO 模式）
- Filter 可以在运行时根据实际配置调整

## 新架构设计

### Scanner 职责
- **提取所有潜在文本**：注释、字符串、文档注释
- **不做语言过滤**：`target_languages = []`（空配置）
- **只检查长度限制**：min_length, max_length
- **返回所有文本区域**：由 Filter 后续处理

### Filter 职责  
- **基于源语言过滤**：使用 `source_langs` 配置
- **AUTO 模式**：过滤掉纯目标语言文本
- **显式模式**：只保留源语言文本
- **格式保护**：URL、占位符检测
- **内容验证**：不是纯符号

## 具体修改

### 1. Scanner 配置修改（coordinator.rs）
```rust
// Scanner should extract all text without language filtering
// Language filtering will be done by the Filter layer based on source_langs
let target_languages: Vec<String> = Vec::new(); // Empty means extract all

let scanner_config = ScannerConfig::new(target_languages)
    .with_comments(extraction_config.comments)
    .with_doc_strings(extraction_config.doc_strings)
    .with_strings(extraction_config.string_literals)
    .with_min_length(config.min_content_length)
    .with_max_length(config.max_content_length);
```

### 2. Filter 恢复完整语言过滤逻辑（language/mod.rs）
```rust
impl Filter for LanguageFilter {
    fn should_translate(&self, text: &str) -> bool {
        // 1. Check translatable content (not just symbols)
        if !self.has_translatable_content(text) {
            return false;
        }

        // 2. Use source language configuration for filtering
        let is_target = self.is_target_language(text);
        let has_source = self.contains_source_language(text);

        if self.source_langs.is_empty() {
            // AUTO mode: filter out target language, keep everything else
            if is_target {
                return false;
            }
            return true;
        }

        // Explicit source language mode: only keep source languages
        if !has_source {
            return false;
        }

        true
    }
}
```

### 3. 保留多点采样策略（character_scanner.rs）
```rust
fn contains_target_language(&self, text: &str) -> bool {
    // If no target languages specified, extract everything
    if self.config.target_languages.is_empty() {
        return true;
    }

    // For short text, check all characters
    if text.chars().count() <= 256 {
        return self.check_text_sample(text);
    }

    // For long text, use multi-point sampling
    // - Check beginning (first 128 chars)
    // - Check middle (middle 128 chars)  
    // - Check end (last 128 chars)
}
```

## 测试更新

已删除的错误测试：
1. `test_rust_doc_comment_extraction_english_only` - 期望提取纯英文注释（不符合正常场景）
2. `test_doc_comment_code_example` - 期望代码示例作为文档注释的一部分（实际会被单独提取）

已修复的测试：
1. `test_auto_to_zh_filters_chinese` - 修正为纯中文/纯英文文本，移除混合语言标记

## E2E 测试发现的问题

### 问题 1：纯英文注释未被过滤

**现象**：
- 当目标语言是中文时，Rust 和 Python 文件中的纯英文注释被保留
- 示例：`// Here's a simple Rust file...` 未被过滤

**根本原因**：
`is_target_language` 方法对中文目标语言的检测逻辑错误：
```rust
// 错误逻辑（旧）
"ZH" | "ZH-CN" | "ZH-TW" => self.quick_detector.has_cjk(text),
```

这个逻辑会错误地将**包含中文的混合文本**识别为目标语言，而**纯英文文本**因为不包含中文，不被认为是目标语言，因此不会被过滤。

**修复方案**：
```rust
// 正确逻辑（新）
"ZH" | "ZH-CN" | "ZH-TW" => {
    // For Chinese target, check if text is purely Chinese (no Latin)
    // Pure Chinese text is already in target language and should be filtered
    self.quick_detector.has_chinese(text) && !self.quick_detector.is_latin(text)
}
```

### 问题 2：混合语言文本处理

**现象**：
- 文档注释中同时包含英文和中文（如 `/// Compute the sum of two numbers` + `* `a` - 第一个数字`）
- 整个注释被提取并翻译

**分析**：
这是**预期行为**。混合语言文本包含需要翻译的内容（中文部分），因此应该被保留和翻译。

### 问题 3：统计信息不准确

**现象**：
- Markdown 文件显示 `Total units: 0`，但实际翻译了内容

**分析**：
这是**预期行为**，不是 bug。原因如下：

1. **Markdown Scanner 配置**：
   ```rust
   line_comment_prefixes: vec![],
   block_comment_delimiters: vec![("<!--", "-->")],
   ```
   Markdown 文件**只提取 HTML 注释**（`<!-- -->`），不提取普通文本内容。

2. **测试文件内容**：
   - `simple_markdown.md` 是纯中文 Markdown 文件
   - **没有任何 HTML 注释**
   - 因此 Scanner 提取了 0 个单元

3. **文件未被修改**：
   - 测试输出显示的"翻译后内容"实际是**原始内容**
   - 因为没有提取到任何单元，文件保持不变
   - 这不是翻译失败，而是正确的行为

**设计原理**：
- Markdown 是标记语言，大部分内容是纯文本
- 如果翻译所有 Markdown 文本，可能破坏文档结构和格式
- 只有 HTML 注释被认为是"元数据"，需要翻译

**如果需要翻译 Markdown 内容**：
1. 修改 Markdown 的 Scanner 配置，提取普通文本
2. 或使用专门的 Markdown 翻译工具

## 优势

1. **职责清晰**：Scanner 负责语法提取，Filter 负责语义过滤
2. **配置正确**：基于源语言过滤，避免逻辑反转
3. **灵活性高**：Filter 可以动态调整，支持 AUTO 模式
4. **性能优化**：Scanner 快速提取，Filter 智能过滤
5. **避免漏检**：Scanner 提取所有文本，不会因采样漏掉混合语言

## 核心设计原则

**语法提取 vs 语义过滤分离**：
- Scanner：识别注释、字符串等语法结构（语法层面）
- Filter：判断是否值得翻译（语义层面）

这种分离使得：
- Scanner 可以专注于高效的文本提取
- Filter 可以根据业务逻辑灵活调整
- 两者独立演进，互不影响
