# cargo test --all 问题报告

## 测试执行日期
2026-04-05

## 执行结果
- **通过**: 394 tests
- **失败**: 4 tests
- **总计**: 398 tests

## 已修复的问题

### 1. PlaceholderSpan 类型未声明
**文件**: `src/parser/scanner/placeholder.rs`
**问题**: `PlaceholderSpan` 类型在使用前未导入
**修复**: 在 import 语句中添加 `PlaceholderSpan` 到 `use crate::parser::scanner::region::{PlaceholderSpan, TextRegion};`

### 2. 孤立的 tree-sitter 测试模块
**文件**: `tests/parser_integration/debug_format_macro.rs`, `tests/parser_integration/tree_sitter_debug.rs`
**问题**: 这些测试文件引用了不存在的 `tree-sitter-rust` crate，但 `Cargo.toml` 中已移除 tree-sitter 相关依赖
**修复**: 删除这两个孤立的测试文件，并从 `mod.rs` 中移除对应的模块声明

### 3. `ParserCoordinator::with_extraction_config` 参数缺失
**文件**: `tests/parser_integration/custom_pattern_tests.rs`
**问题**: 函数签名需要 4 个参数，但测试只提供了 3 个
**修复**: 为所有 5 处调用添加缺失的 `"en"` 参数

### 4. ScannerConfig 默认值问题
**文件**: `src/parser/scanner/character_scanner.rs`
**问题**: `ScannerConfig::default()` 中 `extract_templates` 和 `extract_strings` 默认为 `false`，导致需要这些功能的测试失败
**修复**:
- `test_scan_template_string`: 显式设置 `.with_templates(true)`
- `test_scan_python_multiline`: 显式设置 `.with_strings(true)`
- `test_nested_template_string`: 显式设置 `.with_templates(true)`

### 5. TextRegion 字节偏移问题
**文件**: `src/parser/scanner/region.rs`
**问题**: `test_extract_content` 测试中的字节偏移与中文字符串实际字节长度不匹配
**修复**: 修正 `full_end` 从 12 改为 15（中文 "这是注释" 的 UTF-8 字节长度）

### 6. PlaceholderSpan 偏移与内容长度不匹配
**文件**: `src/parser/scanner/placeholder.rs`
**问题**: 测试中的 placeholder 偏移值超出了提取内容的长度范围
**修复**: 调整 `test_protect_placeholders` 和 `test_restore_placeholders` 中的偏移值，使其与内容长度匹配

## 仍存在的已知问题 (4 tests)

### 1. test_markdown_patterns
**位置**: `src/parser/filtering/checks/pattern.rs:339`
**现象**: `assertion failed: !filter.should_translate("<div>HTML tag</div>")`
**原因**: HTML 标签被错误地判定为需要翻译的内容

### 2. test_scan_python_multiline
**位置**: `src/parser/scanner/character_scanner.rs:702`
**现象**: `assertion 'left == right' failed: left: 0, right: 1`
**原因**: Python 多行字符串扫描逻辑问题

### 3. test_apply_multiple_translations
**位置**: `src/parser/scanner/replacer.rs:223`
**现象**: `assertion 'left == right' failed`
**原因**: 多次翻译应用时的偏移处理问题

### 4. test_apply_single_translation
**位置**: `src/parser/scanner/replacer.rs:45`
**现象**: `end of range should be a character boundary`
**原因**: 单次翻译应用时的字符边界问题

## 总结

本次修复解决了以下编译错误和测试失败:
- 删除了 2 个孤立的 tree-sitter 测试文件
- 修复了 import 语句缺失
- 修复了 5 处函数参数缺失
- 修复了 6 个测试配置问题
- 修正了字节偏移计算问题

仍需进一步调查的问题涉及:
- HTML 标签过滤逻辑
- Python 多行字符串扫描
- 翻译应用器的字符边界处理