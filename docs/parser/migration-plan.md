# Tree-sitter 迁移到字符扫描方案

## 1. 迁移原因

### 当前 tree-sitter 方案的致命问题

1. **函数分类机制导致大量遗漏**: 所有不在预定义列表中的函数调用，其字符串参数都会被跳过
2. **rewrite 阶段本质上就是手动处理**: tree-sitter 的语法感知优势在替换阶段完全丧失
3. **嵌套字符串无法正确处理**: 模板字符串中的嵌套表达式会导致边界识别错误
4. **维护成本高**: 需要为每种语言维护复杂的查询规则

### 字符扫描方案的优势

- 单次扫描，O(n) 时间复杂度
- 基于字节偏移精确替换，无需重建格式
- 提取所有包含目标语言的文本，无遗漏
- 仅需维护简单的语言配置

## 2. 需要移除的代码

### 2.1 解析器模块

```
src/parser/languages/
├── typescript/
│   ├── parser.rs      # 移除
│   ├── patterns.rs    # 移除
│   └── queries.rs     # 移除
├── javascript/
│   ├── parser.rs      # 移除
│   ├── patterns.rs    # 移除
│   └── queries.rs     # 移除
├── python/
│   ├── parser.rs      # 移除
│   └── queries.rs     # 移除
├── rust/
│   ├── parser.rs      # 移除
│   └── queries.rs     # 移除
├── go/
│   ├── parser.rs      # 移除
│   └── queries.rs     # 移除
├── java/
│   ├── parser.rs      # 移除
│   └── queries.rs     # 移除
├── c/
│   ├── parser.rs      # 移除
│   └── queries.rs     # 移除
├── cpp/
│   ├── parser.rs      # 移除
│   └── queries.rs     # 移除
└── csharp/
    ├── parser.rs      # 移除
    └── queries.rs     # 移除
```

### 2.2 Tree-sitter 相关

```
src/parser/tree_sitter/
├── mod.rs             # 移除
├── parser.rs          # 移除
└── language_config.rs # 移除
```

### 2.3 核心模块中的废弃代码

```rust
// src/core/models.rs - 移除
pub enum StrategyNodeType {
    LogMessage,      // 移除
    ErrorMessage,    // 移除
    FormatString,    // 移除
    VariableString,  // 移除
    PropertyString,  // 移除
    TestDescription, // 移除
}

// src/config/project.rs - 移除
pub struct ExtractionConfig {
    pub log_messages: bool,      // 移除
    pub error_messages: bool,    // 移除
    pub format_strings: bool,    // 移除
    pub variable_strings: bool,  // 移除
    pub property_strings: bool,  // 移除
    pub test_descriptions: bool, // 移除
}
```

### 2.4 Writer 模块简化

```
src/writer/
├── applier/
│   ├── multiline.rs   # 简化 - 不再需要复杂的格式重建
│   └── line.rs        # 简化 - 基于偏移替换
└── format/
    └── replacement.rs # 简化 - 不再需要 raw_match 处理
```

## 3. 需要新增的代码

### 3.1 扫描器模块

```
src/parser/scanner/
├── mod.rs              # 模块入口
├── scanner.rs          # 核心扫描器 TextScanner
├── config.rs           # ScannerConfig 定义
├── language.rs         # LanguageConfig 及各语言实现
├── region.rs           # TextRegion, TextRegionType, PlaceholderSpan
├── placeholder.rs      # 占位符保护和恢复
└── replacer.rs         # TranslationReplacer
```

### 3.2 简化的模型

```rust
// src/core/models.rs - 新增/修改

/// 简化的节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Comment,        // 注释
    DocString,      // 文档字符串
    String,         // 字符串
    TemplateString, // 模板字符串
}

/// 简化的翻译单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationUnit {
    pub id: String,
    pub node_type: NodeType,
    pub content: String,
    /// 内容起始字节偏移
    pub start_offset: usize,
    /// 内容结束字节偏移
    pub end_offset: usize,
    pub translated: Option<String>,
    /// 模板占位符 (仅模板字符串)
    pub placeholders: Vec<PlaceholderSpan>,
}
```

### 3.3 简化的配置

```rust
// src/config/project.rs - 新增/修改

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// 提取模式
    pub mode: ExtractionMode,
    /// 提取注释
    pub comments: bool,
    /// 提取文档字符串
    pub doc_strings: bool,
    /// 提取字符串
    pub strings: bool,
    /// 提取模板字符串
    pub templates: bool,
    /// 占位符保护
    pub protect_placeholders: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionMode {
    Thorough, // 字符扫描
}
```

## 4. 迁移步骤

### 阶段1: 创建扫描器模块 (不影响现有功能)

1. 创建 `src/parser/scanner/` 目录
2. 实现核心扫描器 `TextScanner`
3. 实现语言配置 `LanguageConfig`
4. 实现占位符保护
5. 实现翻译应用器 `TranslationReplacer`
6. 编写单元测试

### 阶段2: 集成扫描器

1. 修改 `src/parser/mod.rs`，添加扫描器入口
2. 修改 `src/parser/coordinator.rs`，支持扫描器模式
3. 更新 `TranslationUnit` 结构
4. 更新配置解析

### 阶段3: 简化 Writer

1. 简化 `TranslationApplier`，基于偏移替换
2. 移除复杂的格式重建逻辑
3. 更新 `FileWriter`

### 阶段4: 移除旧代码

1. 移除 `src/parser/languages/` 下的所有解析器
2. 移除 `src/parser/tree_sitter/` 模块
3. 移除 `StrategyNodeType` 中的废弃类型
4. 清理 `Cargo.toml` 中的 tree-sitter 依赖

### 阶段5: 测试和验证

1. 运行所有现有测试
2. 更新集成测试
3. 性能基准测试
4. 文档更新

## 5. Cargo.toml 变更

### 移除的依赖

```toml
# 移除
tree-sitter = "0.24"
tree-sitter-javascript = "0.21"
tree-sitter-typescript = "0.21"
tree-sitter-python = "0.21"
tree-sitter-rust = "0.22"
tree-sitter-go = "0.21"
tree-sitter-java = "0.21"
tree-sitter-c = "0.22"
tree-sitter-cpp = "0.22"
tree-sitter-c-sharp = "0.22"
```

### 保留的依赖

```toml
# 保留
regex = "1"
serde = { version = "1", features = ["derive"] }
tracing = "0.1"
```

## 6. 兼容性考虑

### 配置文件兼容

```toml
# 旧配置 (仍然支持，但会忽略部分选项)
[extraction]
comments = true
doc_strings = true
error_messages = true    # 忽略
format_strings = true    # 忽略
log_messages = true      # 忽略
variable_strings = false # 忽略
property_strings = false # 忽略

# 新配置
[extraction]
mode = "thorough"
comments = true
doc_strings = true
strings = true
templates = true
protect_placeholders = true
```

### 命令行兼容

```bash
# 现有命令保持不变
codebase-translator translate

# 新增选项
codebase-translator translate --mode thorough
```

## 7. 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 转义字符处理错误 | 中 | 完善测试用例，添加边界检测 |
| 多语言混合文件 | 低 | 支持语言切换 |
| 未闭合字符串 | 低 | 跳过并记录警告 |
| 性能回退 | 低 | 单次扫描，性能应优于 AST |

## 8. 时间估算

| 阶段 | 预计时间 |
|------|----------|
| 阶段1: 创建扫描器 | 2-3 天 |
| 阶段2: 集成扫描器 | 1-2 天 |
| 阶段3: 简化 Writer | 1 天 |
| 阶段4: 移除旧代码 | 0.5 天 |
| 阶段5: 测试和验证 | 1-2 天 |
| **总计** | **5-8 天** |

## 9. 回滚计划

如果迁移后发现严重问题：

1. 保留旧代码在 `src/parser/_deprecated/` 目录
2. 通过配置选项切换回 tree-sitter 模式
3. 发布修复版本后移除回滚选项
