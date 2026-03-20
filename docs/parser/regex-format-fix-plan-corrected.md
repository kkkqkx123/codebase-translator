# Regex和状态机格式信息修复方案（修正版）

## 文档信息

- **创建日期**: 2026-03-20
- **目标**: 修复regex和状态机提取时的格式信息丢失问题
- **优先级**: 高
- **关联文档**: [regex-format-info-analysis.md](./regex-format-info-analysis.md)

---

## 一、核心认知修正

### 1.1 Tree-sitter 与 Regex/状态机的本质差异

**Tree-sitter（语法解析）**：
- ✅ 基于AST结构，边界固定（注释标记、字符串引号）
- ✅ 格式可预测，使用 `format_info` 重构是正确的
- ✅ 提取完整节点，格式标记与内容分离

**Regex/状态机（文本匹配）**：
- ❌ 基于字符序列，边界任意
- ❌ 格式不可预测，"格式标记"可能是内容的一部分
- ❌ 部分捕获组提取，无法用 `format_info` 描述

### 1.2 为什么 format_info 不适用于 Regex/状态机

**示例1：TODO 模式**
```javascript
原始代码: "TODO: Fix this bug"
正则匹配: 'TODO:\s*(.+)'
捕获组1: "Fix this bug"

问题：
- "TODO:" 不是格式标记，而是内容的一部分
- 如果用 format_info 重构，会丢失 "TODO:" 前缀
- 正确做法：直接替换捕获组内容，保留 "TODO:"
```

**示例2：错误消息**
```javascript
原始代码: 'throw new Error("Invalid input")'
正则匹配: 'throw new Error\("([^"]+)"\)'
捕获组1: "Invalid input"

问题：
- "throw new Error(" 是代码的一部分，不是格式
- 无法用 format_info 描述这种"跨语法提取"
- 正确做法：直接替换捕获组内容，保留函数调用语法
```

**示例3：日志消息**
```python
原始代码: 'logger.info("User logged in")'
正则匹配: 'logger\.(info|debug|warn|error)\("([^"]+)"\)'
捕获组2: "User logged in"

问题：
- 匹配包含函数调用语法
- 无法用 format_info 描述这种"函数调用内的提取"
- 正确做法：直接替换捕获组内容，保留函数调用
```

### 1.3 核心结论

**Tree-sitter 使用 format_info 重构**：
- 提取完整节点（包括格式标记）
- 翻译时只翻译内容
- 写入时用 format_info 重构完整节点

**Regex/状态机使用直接替换**：
- 提取部分内容（捕获组）
- 翻译时只翻译提取的部分
- 写入时在原始匹配中找到提取部分，直接替换

**两者是截然不同的处理逻辑，不能混用！**

---

## 二、修改方案

### 2.1 核心策略：直接替换而非 format_info 重构

#### 2.1.1 数据流设计

```
提取阶段：
  raw_match: "TODO: Fix this bug"           ← 完整匹配
  extracted_text: "Fix this bug"               ← 提取的文本（用于翻译）
  start_pos/end_pos: 基于原始匹配的位置

翻译阶段：
  翻译 "Fix this bug" → "修复此问题"

写入阶段：
  在 raw_match 中找到 extracted_text 的位置
  替换为翻译后的文本
  结果: "TODO: 修复此问题"                   ← 保留了 "TODO:" 前缀
```

#### 2.1.2 关键数据结构

**TranslationUnit 扩展**：
```rust
pub struct TranslationUnit {
    // 原有字段
    pub id: String,
    pub node_type: NodeType,
    pub content: String,              // 提取的文本（extracted_text）
    pub start_pos: Position,
    pub end_pos: Position,
    pub format_info: Option<FormatInfo>, // Tree-sitter 用，Regex/状态机为 None

    // 新增字段（Regex/状态机专用）
    pub raw_match: Option<String>,     // 完整的原始匹配
    pub pattern_type: Option<PatternType>, // Builtin/CustomRegex/StateMachine
    pub pattern_name: Option<String>,
    pub translated: Option<String>,
    // ...
}
```

**字段用途说明**：
- `content`: 提取的文本，用于翻译（Tree-sitter 和 Regex/状态机都使用）
- `format_info`: Tree-sitter 专用，用于重构完整节点
- `raw_match`: Regex/状态机专用，用于直接替换策略
- `pattern_type`: 区分提取来源，决定使用哪种写入策略

---

### 2.2 第一阶段：保存 raw_match（高优先级）

#### 2.2.1 修改目标

在创建 TranslationUnit 时，对于 Regex/状态机提取的内容，必须保存 `raw_match`。

#### 2.2.2 修改位置

**文件**: `src/parser/coordinator/coordinator.rs`

**CustomPatternMatcher 处理**：
- 当前问题：创建了 TranslationUnit，但未设置 `raw_match`
- 修改方案：在创建后立即设置 `unit.raw_match = Some(m.raw_content)`

**StateMachineMatcher 处理**：
- 当前问题：创建了 TranslationUnit，但未设置 `raw_match`
- 修改方案：在创建后立即设置 `unit.raw_match = Some(m.raw_content)`

#### 2.2.3 注意事项

1. **raw_match 必须包含完整的匹配文本**
   - 包括所有前缀、后缀、格式标记
   - 例如：`"TODO: Fix this bug"` 而非 `"Fix this bug"`

2. **位置信息必须基于原始匹配**
   - `start_pos` 和 `end_pos` 应该是 `raw_match` 的位置
   - 不能是 `extracted_text` 的位置

3. **format_info 应该为 None**
   - Regex/状态机提取的内容不使用 format_info
   - 写入时使用直接替换策略

---

### 2.3 第二阶段：写入逻辑修改（高优先级）

#### 2.3.1 修改目标

在写入阶段，根据提取来源选择不同的策略：
- Tree-sitter 提取：使用 format_info 重构
- Regex/状态机提取：使用直接替换

#### 2.3.2 修改位置

**文件**: `src/writer/core.rs`

**当前逻辑问题**：
- 写入时只检查 `raw_match` 是否存在
- 如果存在，直接使用翻译文本替换整个 `raw_match`
- 这是错误的！应该只替换提取的部分

**正确的逻辑**：
```rust
// 判断使用哪种策略
if unit.pattern_type == Some(PatternType::Builtin) {
    // Tree-sitter 提取：使用 format_info 重构
    if let Some(format) = &unit.format_info {
        formatted = Self::format_translated_text(translated, format);
    }
} else {
    // Regex/状态机提取：使用直接替换
    if let Some(raw_match) = &unit.raw_match {
        formatted = Self::replace_in_raw_match(raw_match, &unit.content, translated);
    }
}
```

#### 2.3.3 直接替换算法

**核心思想**：
在 `raw_match` 中找到 `extracted_text`（即 `unit.content`）的位置，替换为翻译后的文本。

**实现要点**：
1. 在 `raw_match` 中查找 `extracted_text` 的位置
2. 计算偏移量
3. 替换对应位置的文本
4. 返回替换后的完整文本

**边界情况处理**：
- 如果 `extracted_text` 在 `raw_match` 中出现多次，使用第一次出现的位置
- 如果找不到，记录错误并跳过
- 如果翻译文本与原文长度不同，不影响替换逻辑

---

### 2.4 第三阶段：位置信息处理（中优先级）

#### 2.4.1 问题描述

当前实现中，位置信息可能不准确：
- `start_pos` 和 `end_pos` 是基于 `extracted_text` 的
- 但写入时需要基于 `raw_match` 的位置
- 导致替换位置错误

#### 2.4.2 解决方案

**方案A：使用字节偏移而非字符偏移**
- 所有位置信息使用字节偏移
- 确保与 UTF-8 编码兼容
- 在显示时转换为行号和列号

**方案B：保存双重位置信息**
- 添加 `raw_start_pos` 和 `raw_end_pos` 字段
- 分别保存 `raw_match` 和 `extracted_text` 的位置
- 写入时使用正确的位置

**推荐方案**：方案A（字节偏移）
- 更简单，不需要修改数据结构
- 正则表达式已经返回字节偏移
- 只需要确保一致性

#### 2.4.3 实施要点

1. **统一使用字节偏移**
   - CustomPatternMatcher 和 StateMachineMatcher 的位置计算
   - TranslationUnit 的位置字段
   - Writer 的替换逻辑

2. **验证位置准确性**
   - 在写入前验证位置是否在文件范围内
   - 检查起始位置是否小于结束位置
   - 如果无效，记录错误并跳过

---

## 三、关键问题处理

### 3.1 多行匹配问题

#### 3.1.1 问题描述

当 `raw_match` 跨越多行时，直接替换可能失败：
- 位置信息可能不准确
- 换行符处理可能不一致
- 缩进可能丢失

#### 3.1.2 解决方案

**提取阶段**：
- 正确计算多行匹配的起始和结束位置
- 保留原始的换行符和缩进
- 在 `raw_match` 中保存完整的多行文本

**写入阶段**：
- 在 `raw_match` 中查找 `extracted_text` 的位置
- 如果 `extracted_text` 也跨多行，需要特殊处理
- 确保替换后的文本保持相同的行数和换行符

**具体策略**：
1. 检测 `raw_match` 中是否包含换行符
2. 检测 `extracted_text` 中是否包含换行符
3. 如果都跨多行，按行分割并逐行替换
4. 如果只有 `raw_match` 跨多行，需要确定 `extracted_text` 在哪一行
5. 保留原始的换行符和缩进

#### 3.1.3 注意事项

- 不同操作系统的换行符（CRLF vs LF）需要统一处理
- 缩进可能是空格或制表符，需要保留原始类型
- 翻译文本的行数可能与原文不同，需要处理这种情况

---

### 3.2 嵌套匹配问题

#### 3.2.1 问题描述

当多个匹配项重叠或嵌套时，直接替换可能导致冲突：
- 内部匹配的替换可能影响外部匹配的位置
- 替换顺序可能影响最终结果
- 可能导致文本损坏

#### 3.2.2 解决方案

**检测嵌套匹配**：
- 在写入前，检查所有匹配项的位置关系
- 检测是否存在重叠或嵌套
- 如果发现嵌套，记录警告并采取相应策略

**处理策略**：

**策略A：按位置排序，从后向前替换**
- 将所有匹配项按起始位置排序
- 从后向前替换（避免位置偏移影响）
- 适用于大多数情况

**策略B：在提取阶段拒绝嵌套匹配**
- 在提取时就检测嵌套
- 只保留最外层的匹配
- 简单但可能丢失信息

**推荐策略**：策略A（从后向前替换）
- 不会丢失信息
- 对于大多数情况都有效
- 记录警告供用户参考

#### 3.2.3 实施要点

1. **嵌套检测算法**
   - 比较所有匹配项的位置范围
   - 检测是否存在包含关系
   - 检测是否存在部分重叠

2. **排序和替换**
   - 按起始位置排序
   - 从后向前遍历
   - 每次替换后更新位置信息

3. **警告和日志**
   - 检测到嵌套时记录警告
   - 提供匹配项的详细信息
   - 便于用户调试

---

### 3.3 提取内容查找问题

#### 3.3.1 问题描述

在 `raw_match` 中查找 `extracted_text` 的位置时，可能遇到以下问题：
- `extracted_text` 在 `raw_match` 中出现多次
- `extracted_text` 是 `raw_match` 的子串，但位置不明确
- 正则表达式的捕获组可能包含额外的字符

#### 3.3.2 解决方案

**方案A：使用正则表达式捕获组的位置**
- 正则表达式匹配时，记录捕获组的起始和结束位置
- 直接使用这些位置，无需查找
- 最准确，但需要修改匹配器接口

**方案B：在 raw_match 中查找**
- 使用字符串查找方法找到 `extracted_text` 的位置
- 如果出现多次，使用第一次出现的位置
- 如果找不到，记录错误并跳过

**推荐方案**：方案A（捕获组位置）
- 更准确，不依赖字符串查找
- 正则表达式已经提供了捕获组位置
- 只需要修改匹配器接口，保存位置信息

#### 3.3.3 实施要点

1. **修改匹配器接口**
   - CustomPatternMatch 添加 `extracted_start` 和 `extracted_end` 字段
   - StateMachineMatch 添加 `extracted_start` 和 `extracted_end` 字段
   - 使用正则表达式的 `capture.start()` 和 `capture.end()` 方法

2. **保存位置信息**
   - 在匹配时记录捕获组的字节偏移
   - 保存到 TranslationUnit 的相应字段
   - 写入时直接使用这些位置

3. **降级处理**
   - 如果位置信息不可用，降级到字符串查找
   - 记录警告，便于调试
   - 确保系统不会崩溃

---

### 3.4 编码和字符问题

#### 3.4.1 问题描述

多字节字符（如中文）可能导致位置计算错误：
- 字节偏移 vs 字符偏移
- UTF-8 编码的复杂性
- 不同语言的字符宽度

#### 3.4.2 解决方案

**统一使用字节偏移**：
- 所有位置信息使用字节偏移
- 正则表达式已经返回字节偏移
- 确保与 UTF-8 编码兼容

**字符处理**：
- 在显示时转换为行号和列号
- 使用 `str::char_indices()` 方法处理多字节字符
- 避免直接使用字符索引

#### 3.4.3 实施要点

1. **位置信息类型**
   - 使用 `usize` 表示字节偏移
   - 在文档中明确说明是字节偏移
   - 提供转换函数（字节偏移 → 行号/列号）

2. **字符串操作**
   - 使用 `&str[byte_offset..]` 切片
   - 避免使用 `chars().nth()` 等方法
   - 使用 `char_indices()` 进行字符级别的操作

3. **测试覆盖**
   - 测试纯 ASCII 文本
   - 测试混合 ASCII 和多字节字符
   - 测试纯多字节字符（如纯中文）

---

## 四、测试用例设计

### 4.1 基础功能测试

#### 测试用例1：简单 TODO 模式提取和还原

**测试目标**: 验证基本的正则提取、翻译和还原流程

**测试场景**:
- 源代码包含一个 TODO 注释
- 使用 CustomPatternMatcher 提取（正则：`TODO:\s*(.+)`）
- 翻译提取的文本
- 写入并验证格式是否正确还原

**输入示例**:
```javascript
// TODO: Fix this bug
function example() {
    console.log("hello");
}
```

**预期结果**:
- `raw_match`: `"TODO: Fix this bug"`
- `extracted_text`: `"Fix this bug"`
- 翻译后: `"修复此问题"`
- 写入后: `"// TODO: 修复此问题"`（保留了 "TODO:" 前缀）

**边界条件**:
- TODO 后面没有空格
- TODO 内容为空
- TODO 内容包含特殊字符

---

#### 测试用例2：错误消息提取和还原

**测试目标**: 验证函数调用中的字符串提取

**测试场景**:
- 源代码包含一个 throw 语句
- 使用 CustomPatternMatcher 提取（正则：`throw new Error\("([^"]+)"\)`）
- 翻译提取的文本
- 写入并验证格式是否正确还原

**输入示例**:
```javascript
function validate(input) {
    if (!input) {
        throw new Error("Invalid input");
    }
}
```

**预期结果**:
- `raw_match`: `'throw new Error("Invalid input")'`
- `extracted_text`: `"Invalid input"`
- 翻译后: `"无效输入"`
- 写入后: `'throw new Error("无效输入")'`（保留了函数调用语法）

**边界条件**:
- 错误消息为空
- 错误消息包含引号
- 错误消息包含转义字符

---

### 4.2 多行匹配测试

#### 测试用例3：多行 TODO 提取和还原

**测试目标**: 验证多行匹配的正确处理

**测试场景**:
- 源代码包含一个多行 TODO 注释
- 使用 StateMachineMatcher 提取
- 翻译提取的文本
- 写入并验证格式是否正确还原

**输入示例**:
```javascript
// TODO: Fix this bug
// This is a detailed description
// of the issue
function example() {
    console.log("hello");
}
```

**预期结果**:
- `raw_match`: 包含完整的多行注释
- `extracted_text`: 包含多行文本（不含 "//" 前缀）
- 翻译后: 翻译后的多行文本
- 写入后: 保留了 "//" 前缀和换行符

**边界条件**:
- TODO 内容为空
- TODO 内容包含代码示例
- 不同缩进级别

---

#### 测试用例4：多行字符串提取和还原

**测试目标**: 验证多行字符串的正确处理

**测试场景**:
- 源代码包含一个多行字符串
- 使用 StateMachineMatcher 提取
- 翻译提取的文本
- 写入并验证格式是否正确还原

**输入示例**:
```python
message = """
    This is a multi-line
    string literal
    in Python
"""
```

**预期结果**:
- `raw_match`: 包含完整的多行字符串（包括三引号）
- `extracted_text`: 包含多行文本（不含三引号）
- 翻译后: 翻译后的多行文本
- 写入后: 保留了三引号和缩进

**边界条件**:
- 字符串为空
- 字符串包含转义字符
- 字符串包含内嵌的引号

---

### 4.3 嵌套匹配测试

#### 测试用例5：嵌套匹配检测和处理

**测试目标**: 验证嵌套匹配的检测和处理

**测试场景**:
- 源代码包含多个可能重叠的匹配项
- 使用多个 CustomPatternMatcher 提取
- 验证系统是否正确检测嵌套
- 验证写入时是否正确处理

**输入示例**:
```javascript
// TODO: Fix this bug
// TODO: Add tests
console.log("TODO: Don't forget this");
```

**预期结果**:
- 系统检测到多个匹配项
- 检测到可能的嵌套或重叠
- 按位置排序，从后向前替换
- 写入后的代码不损坏

**边界条件**:
- 完全嵌套（内部匹配完全在外部匹配内）
- 部分重叠（匹配项部分重叠）
- 相邻匹配（匹配项紧挨着）

---

### 4.4 位置信息测试

#### 测试用例6：字节偏移 vs 字符偏移

**测试目标**: 验证位置信息使用字节偏移的正确性

**测试场景**:
- 源代码包含多字节字符（如中文）
- 使用匹配器提取
- 验证位置信息是否使用字节偏移
- 验证写入时是否正确替换

**输入示例**:
```javascript
// TODO: 修复这个中文bug
function example() {
    console.log("你好世界");
}
```

**预期结果**:
- 位置信息使用字节偏移
- 写入时正确替换，不破坏多字节字符
- 翻译后的文本正确显示

**边界条件**:
- 纯 ASCII 文本
- 混合 ASCII 和多字节字符
- 纯多字节字符（如纯中文）

---

### 4.5 提取内容查找测试

#### 测试用例7：提取内容多次出现

**测试目标**: 验证提取内容在 raw_match 中多次出现时的处理

**测试场景**:
- 源代码中，提取的内容在 raw_match 中出现多次
- 使用匹配器提取
- 验证系统是否使用正确的位置

**输入示例**:
```javascript
// TODO: Fix bug bug bug
```

**预期结果**:
- `raw_match`: `"TODO: Fix bug bug bug"`
- `extracted_text`: `"Fix bug bug bug"`
- 翻译后: `"修复bug bug bug"`
- 写入后: `"// TODO: 修复bug bug bug"`（使用第一次出现的位置）

**边界条件**:
- 提取内容为空
- 提取内容只出现一次
- 提取内容出现多次，但位置不同

---

### 4.6 集成测试

#### 测试用例8：完整流程测试（简单场景）

**测试目标**: 验证从提取到写入的完整流程

**测试场景**:
- 创建一个包含多个可翻译文本的源文件
- 使用多个匹配器提取（包括 Tree-sitter 和 Regex）
- 翻译所有提取的文本
- 写入并验证结果

**输入示例**:
```javascript
// This is a comment
function example() {
    // TODO: Fix this bug
    console.log("Hello world");
    throw new Error("Invalid input");
}
```

**预期结果**:
- Tree-sitter 提取的注释使用 format_info 重构
- Regex 提取的 TODO 使用直接替换
- 所有匹配项都被正确提取和翻译
- 写入后的文件格式正确
- 原始代码结构保持不变

**边界条件**:
- 文件包含多种类型的匹配项
- 匹配项分散在文件的不同位置
- 翻译文本长度与原文不同

---

#### 测试用例9：完整流程测试（复杂场景）

**测试目标**: 验证复杂场景下的完整流程

**测试场景**:
- 创建一个包含复杂情况的源文件
  - 多行注释
  - 多行字符串
  - 嵌套的匹配项
  - 多字节字符
  - 边界位置的匹配项
- 使用多个匹配器提取
- 翻译所有提取的文本
- 写入并验证结果

**输入示例**:
```javascript
/**
 * This is a multi-line
 * doc comment
 */
function validate(input) {
    // TODO: 修复这个中文bug
    // This is a detailed description
    if (!input) {
        throw new Error("无效输入");
    }
    console.log("你好世界");
}
```

**预期结果**:
- 所有匹配项都被正确提取
- Tree-sitter 提取使用 format_info 重构
- Regex 提取使用直接替换
- 系统正确处理复杂情况
- 写入后的文件格式正确
- 不产生任何损坏或错误

**边界条件**:
- 所有复杂情况同时出现
- 翻译文本长度与原文差异很大
- 文件很大（性能测试）

---

## 五、实施计划

### 5.1 第一阶段：基础修复（高优先级）

**目标**: 修复最关键的问题，确保基本功能正常

**任务**:
1. 在 ParserCoordinator 中保存 `raw_match`
2. 修改 Writer 的写入逻辑，区分 Tree-sitter 和 Regex/状态机
3. 实现直接替换算法
4. 添加基础的测试用例

**时间估计**: 2-3天

**验收标准**:
- 所有基础功能测试通过
- 简单场景的完整流程测试通过
- Tree-sitter 和 Regex/状态机的写入策略正确分离

---

### 5.2 第二阶段：位置信息优化（中优先级）

**目标**: 优化位置信息的准确性和一致性

**任务**:
1. 统一使用字节偏移
2. 修改匹配器接口，保存捕获组位置
3. 添加位置信息验证
4. 添加位置信息相关测试

**时间估计**: 2-3天

**验收标准**:
- 位置信息测试通过
- 多字节字符测试通过
- 位置信息验证逻辑正常工作

---

### 5.3 第三阶段：复杂场景处理（中优先级）

**目标**: 处理多行匹配、嵌套匹配等复杂场景

**任务**:
1. 实现多行匹配处理逻辑
2. 实现嵌套匹配检测和处理
3. 添加复杂场景测试

**时间估计**: 3-4天

**验收标准**:
- 多行匹配测试通过
- 嵌套匹配测试通过
- 复杂场景集成测试通过

---

### 5.4 第四阶段：优化和完善（低优先级）

**目标**: 优化性能，完善文档和测试

**任务**:
1. 性能优化（大文件处理）
2. 完善文档和注释
3. 添加更多边界测试

**时间估计**: 2-3天

**验收标准**:
- 性能测试通过
- 文档完整
- 测试覆盖率达到目标

---

## 六、风险评估

### 6.1 技术风险

**风险1：直接替换算法错误**
- **影响**: 高
- **概率**: 中
- **缓解措施**: 严格的测试，特别是多行和嵌套情况
- **回退方案**: 如果替换失败，保留原始文本

**风险2：位置信息不准确**
- **影响**: 高
- **概率**: 中
- **缓解措施**: 使用正则表达式的捕获组位置，而非字符串查找
- **回退方案**: 添加位置验证，无效时跳过

**风险3：多行匹配处理复杂**
- **影响**: 中
- **概率**: 高
- **缓解措施**: 分阶段实现，先处理简单情况
- **回退方案**: 对于复杂多行匹配，降级到逐行处理

---

### 6.2 兼容性风险

**风险1：向后兼容性**
- **影响**: 高
- **概率**: 低
- **缓解措施**: 新字段使用 `Option`，提供默认值
- **回退方案**: 保持旧字段不变，添加新字段

**风险2：跨平台兼容性**
- **影响**: 中
- **概率**: 低
- **缓解措施**: 统一使用字节偏移，处理不同换行符
- **回退方案**: 检测原始换行符并保留

---

## 七、总结

本修复方案基于正确的认知：**Regex/状态机与 Tree-sitter 是截然不同的处理逻辑**。

### 核心要点

1. **Tree-sitter 使用 format_info 重构**
   - 提取完整节点
   - 格式可预测
   - 用 format_info 重构

2. **Regex/状态机使用直接替换**
   - 提取部分内容（捕获组）
   - 格式不可预测
   - 在 raw_match 中直接替换提取部分

3. **关键修改**
   - 保存 `raw_match`
   - 区分写入策略
   - 实现直接替换算法

4. **重点问题**
   - 多行匹配处理
   - 嵌套匹配检测
   - 位置信息准确性
   - 编码和字符处理

通过分阶段实施，可以逐步提高系统的稳定性和可靠性，最终实现 Regex/状态机的正确格式保留能力。
