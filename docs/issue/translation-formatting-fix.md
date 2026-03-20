# 翻译输出格式错乱问题修复报告

## 问题描述

E2E测试中发现翻译输出文件出现严重格式错乱，具体表现为：

**原始代码：**
```rust
/// 乘法运算
pub fn multiply(a: i32, b: i32) -> i32 {
```

**错误输出：**
```rust
/// multiplicationpub fn multiply(a: i32, b: i32) -> i32 {
```

文档注释和函数定义被合并到同一行，丢失了换行符。

## 根本原因分析

### 1. Tree-sitter 节点文本包含换行符

Tree-sitter 解析器返回的 `line_comment` 节点文本包含末尾换行符：

```
Match 20:
  text: "/// 乘法运算\n"
  text.len(): 17
  start_position: row=30, column=0
  end_position: row=31, column=0
```

### 2. `raw_match` 被错误分类为多行单元

在 `apply_translations` 函数中，包含换行符的单元被错误地分类为多行单元：

```rust
let multiline_units: Vec<&TranslationUnit> = units
    .iter()
    .filter(|u| u.raw_match.is_some() && u.raw_match.as_ref().unwrap().contains('\n'))
    .collect();
```

`"/// 乘法运算\n"` 被当作多行单元处理。

### 3. `format_multiline_translation` 未保留末尾换行符

`format_multiline_translation` 函数在处理单行注释时，没有保留 `raw_match` 的末尾换行符：

```rust
// raw_match = "/// 乘法运算\n"
// translated = "multiplication"
// 处理后返回: "/// multiplication" (缺少末尾换行符!)
```

### 4. 替换操作导致行合并

```rust
result = result.replace("/// 乘法运算\n", "/// multiplication");
// 结果: "/// multiplicationpub fn multiply..."
```

## 修复方案

### 修改 1: 增强 `clean_doc_comment` 函数

**文件**: `src/parser/core/string_processor.rs`

修改 `///` 和 `//!` 注释的处理逻辑，只处理以注释标记开头的行：

```rust
// Handle Rust outer doc: ///
if text.starts_with("///") {
    return text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            // Only process lines that start with ///
            // Skip lines that don't start with /// (e.g., code lines)
            trimmed
                .strip_prefix("///")
                .map(|s| s.trim_start())
        })
        .collect::<Vec<_>>()
        .join("\n");
}
```

### 修改 2: 修复 `format_multiline_translation` 函数

**文件**: `src/writer/core.rs`

添加对 `raw_match` 末尾换行符的检测和保留：

```rust
fn format_multiline_translation(raw_match: &str, translated: &str) -> String {
    let raw_lines: Vec<&str> = raw_match.lines().collect();
    let translated_lines: Vec<&str> = translated.lines().collect();

    // Check if raw_match ends with newline - we need to preserve this
    let ends_with_newline = raw_match.ends_with('\n');
    
    // ... 原有处理逻辑 ...
    
    // Preserve trailing newline if raw_match had one
    if ends_with_newline && !result.ends_with('\n') {
        result.push('\n');
    }
    
    result
}
```

## 测试验证

### 新增单元测试

**文件**: `src/parser/core/string_processor.rs`

```rust
#[test]
fn test_clean_doc_comment_with_extra_lines() {
    let processor = StringProcessor::new();

    // Test case: tree-sitter returns node text that includes code lines
    let text = "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {";
    let result = processor.clean_doc_comment(text);
    assert_eq!(result, "乘法运算", "Should only extract doc comment, not code lines");
}
```

**文件**: `src/writer/core.rs`

```rust
#[test]
fn test_raw_match_with_newline() {
    let content = "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {";
    
    let mut units = vec![TranslationUnit {
        id: "1".to_string(),
        node_type: NodeType::DocString,
        content: "乘法运算".to_string(),
        start_pos: Position::new(1, 1, 0),
        end_pos: Position::new(1, 17, 16),
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: None,
        pattern_name: None,
        // raw_match includes newline
        raw_match: Some("/// 乘法运算\n".to_string()),
    }];
    
    units[0].set_translated("multiplication");
    
    let result = TranslationApplier::apply_translations(content, &units).unwrap();
    
    // Should NOT merge lines
    assert!(!result.contains("multiplicationpub fn"));
}
```

### E2E 测试结果

修复后 E2E 测试通过，输出文件格式正确：

```rust
/// multiplication
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
```

## 相关文件

- `src/parser/core/string_processor.rs` - 注释清理逻辑
- `src/writer/core.rs` - 翻译应用逻辑
- `tests/parser_integration/tree_sitter_debug.rs` - Tree-sitter 调试测试
- `tests/parser_integration/check_line_endings.rs` - 换行符检查测试

## 修复日期

2026-03-20
