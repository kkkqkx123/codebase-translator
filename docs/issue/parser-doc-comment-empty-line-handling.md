# Parser 文档注释空行处理问题

## 问题描述

Parser 在处理带有空行的文档注释时存在两个问题：

### 问题 1: 带注释标记的空行被误判为分隔符

**现象**：连续文档注释中的 `/// `（带空格的空文档注释）被当作空行处理，导致注释被错误拆分。

**示例**：
```rust
/// 创建新的计算器实例
/// 
/// # Arguments
/// 
/// * `name` - 计算器名称
```

**当前解析结果**：
```
Content:
创建新的计算器实例
# Arguments
Raw Match:
/// 创建新的计算器实例

/// # Arguments
```

**问题**：
1. `/// ` 行被跳过，没有出现在 `Raw Match` 中
2. 内容被错误合并，`# Arguments` 与前面的内容连在一起

### 问题 2: 代码示例被错误提取

**现象**：文档注释中的代码示例（如 `assert_eq!(result, 3);`）被当作独立单元提取。

**示例**：
```rust
/// # Examples
/// 
/// ```
/// let result = add(1, 2);
/// assert_eq!(result, 3);
/// ```
```

**当前解析结果**：
- Unit 3: `# Examples`
- Unit 4: `assert_eq!(result, 3);`

**问题**：
1. 代码示例被孤立提取
2. 代码示例被标记为 `Should Translate: true`，但实际上不应该翻译

## 影响

1. **翻译质量下降**：Markdown 格式（如 `# Arguments`）与内容分离，导致翻译后格式混乱
2. **翻译内容错误**：代码示例被翻译，可能导致代码无法运行
3. **Raw Match 不完整**：无法正确还原原始格式

## 根本原因

1. **空行判断逻辑过于简单**：仅使用 `line.trim().is_empty()` 判断空行，没有考虑带注释标记的"空行"
2. **合并逻辑问题**：合并时没有正确处理带注释标记的空行
3. **代码块识别缺失**：没有识别文档注释中的代码块（被 ``` 包围的内容）

## 期望行为

### 场景 1: 带注释标记的空行
```rust
/// 创建新的计算器实例
/// 
/// # Arguments
```

**期望解析结果**：
```
Content:
创建新的计算器实例

# Arguments
Raw Match:
/// 创建新的计算器实例
/// 
/// # Arguments
```

### 场景 2: 代码示例
```rust
/// # Examples
/// 
/// ```
/// assert_eq!(result, 3);
/// ```
```

**期望解析结果**：
- 要么将整个代码块作为一个单元保留
- 要么跳过代码块内的内容，只翻译说明文字

## 修复方案

### 修复内容

**文件**: `src/parser/tree_sitter.rs`

**问题**: 空文档注释行（如 `/// `）被过滤逻辑错误地排除

**修复**: 在过滤逻辑中添加特殊处理，保留文档注释中的空行：

```rust
// For doc comments, preserve empty lines (e.g., "/// ") for proper merging
// Check if original text is a doc comment marker with empty content
let is_doc_empty_line = {
    let trimmed = node_text.trim();
    (trimmed == "///" || trimmed == "//!" || trimmed.starts_with("/// ") || trimmed.starts_with("//! "))
        && strategy_node_type == StrategyNodeType::DocString
};

if !is_doc_empty_line {
    // Apply normal filtering logic
    if text.len() < self.config.min_content_length { continue; }
    if text.len() > self.config.max_content_length { continue; }
    if is_only_symbols(&text) { continue; }
    if !self.filter.should_translate(&text) { continue; }
}
```

### 修复效果

**修复前**:
```
Content: "创建新的计算器实例\n# Arguments"
Raw Match: "/// 创建新的计算器实例\n/// # Arguments"
```

**修复后**:
```
Content: "创建新的计算器实例\n\n# Arguments\n\n* `name` - 计算器名称"
Raw Match: "/// 创建新的计算器实例\n/// \n/// # Arguments\n/// \n/// * `name` - 计算器名称"
```

### 代码示例处理

代码示例现在被正确地合并到同一个文档注释单元中，而不是被拆分为多个单元：

**修复前**:
- Unit 1: `# Examples`
- Unit 2: `assert_eq!(result, 3);`

**修复后**:
- Unit 1: `# Examples\n\n```\nlet result = add(1, 2);\nassert_eq!(result, 3);\n```'

## 相关文件

- `src/parser/tree_sitter.rs` - 注释提取和合并逻辑
- `src/parser/core/string_processor.rs` - 字符串处理
- `tests/parser_integration/output/chinese_rust_parse.txt` - 解析结果示例

## 测试用例

### 测试 1: 带空行的文档注释
```rust
/// 第一行
/// 
/// 第二行
```

期望：
- Content: `第一行

第二行`
- Raw Match: `/// 第一行
/// 
/// 第二行`

### 测试 2: 包含代码示例的文档注释
```rust
/// # Examples
/// 
/// ```
/// let x = 1;
/// ```
```

期望：
- 代码块内容（`let x = 1;`）不应被提取为独立单元
- 或代码块应被标记为 `Should Translate: false`
