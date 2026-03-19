# 提取规则验证功能设计方案

## 1. 功能概述

### 1.1 背景

在 codebase-translator 项目中，用户可以通过配置文件定义自定义的提取规则（正则表达式模式、状态机模式）来提取待翻译的文本内容。然而，用户在配置这些规则后，往往需要验证规则是否能正确提取预期内容，以及提取结果是否符合预期。

### 1.2 目标

提供一个 CLI 命令，允许用户：
- 验证配置的提取规则在实际文件上的表现
- 查看规则提取到的所有待翻译内容
- 不执行实际的翻译和写入操作
- 支持按模式名称、文件类型、类别进行过滤
- 提供多种输出格式（表格、JSON、CSV）

### 1.3 使用场景

- **规则调试**：开发或调整提取规则时，快速验证规则是否按预期工作
- **规则评估**：评估提取规则的覆盖范围和准确性
- **规则文档**：生成规则提取结果的示例文档
- **CI/CD 集成**：在 CI/CD 流程中验证提取规则的有效性

## 2. 需求分析

### 2.1 功能需求

#### 2.1.1 核心功能

1. **单文件验证**：支持验证单个文件
2. **目录验证**：支持验证整个目录（递归）
3. **规则应用**：应用所有配置的提取规则（内置规则、自定义正则、状态机）
4. **结果输出**：输出提取到的待翻译内容及其元数据

#### 2.1.2 过滤功能

1. **按模式名称过滤**：只显示特定模式的匹配结果
2. **按文件扩展名过滤**：只显示特定文件类型的匹配结果
3. **按类别过滤**：只显示特定类别的匹配结果（error_handling、output、variables、properties、other）
4. **按文本内容过滤**：只显示包含特定关键词的匹配结果

#### 2.1.3 输出格式

1. **表格格式**：易于阅读的表格输出（默认）
2. **JSON 格式**：结构化数据，便于程序处理
3. **CSV 格式**：表格数据，便于导入电子表格
4. **详细模式**：显示更多元数据（原始匹配内容、正则捕获组等）

#### 2.1.4 统计信息

1. **总体统计**：文件总数、匹配总数
2. **按模式统计**：每个模式的匹配数量
3. **按类别统计**：每个类别的匹配数量
4. **按文件类型统计**：每种文件类型的匹配数量

### 2.2 非功能需求

1. **性能**：大型项目验证应该在合理时间内完成
2. **兼容性**：与现有配置系统、解析器系统完全兼容
3. **可维护性**：代码结构清晰，易于扩展
4. **可测试性**：提供单元测试和集成测试

### 2.3 约束条件

1. 不修改原文件
2. 不执行翻译操作
3. 不写入缓存
4. 不依赖翻译服务

## 3. 架构设计

### 3.1 系统架构

```
CLI 命令层
  └─> verify 命令处理器 (src/commands/verify.rs)
      └─> 配置加载 (ConfigLoader)
      └─> 解析器协调器 (ParserCoordinator)
      └─> 结果收集器 (ResultCollector)
      └─> 结果过滤 (ResultFilter)
      └─> 统计生成器 (StatisticsGenerator)
      └─> 输出格式化器 (OutputFormatter)
```

### 3.2 模块设计

#### 3.2.1 verify 命令模块

**位置**：`src/commands/verify.rs`

**职责**：
- 解析命令行参数
- 加载配置
- 调用解析器提取内容
- 收集和过滤结果
- 生成统计信息
- 格式化输出

**核心函数**：
```rust
pub fn execute_verify_command(args: VerifyArgs) -> Result<VerifyResult>
```

#### 3.2.2 结果收集器

**位置**：`src/commands/verify/collector.rs`

**职责**：
- 从解析器结果中提取元数据
- 识别匹配的模式类型
- 记录位置信息

**数据结构**：
```rust
pub struct VerifyMatch {
    pub file_path: PathBuf,
    pub pattern_name: String,
    pub pattern_type: PatternType,
    pub category: String,
    pub extracted_text: String,
    pub position: Position,
    pub raw_match: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub enum PatternType {
    Builtin,           // 内置解析器（注释、字符串等）
    CustomRegex,       // 自定义正则模式
    StateMachine,      // 状态机模式
}
```

#### 3.2.3 结果过滤器

**位置**：`src/commands/verify/filter.rs`

**职责**：
- 根据用户条件过滤结果
- 支持多种过滤条件

**核心函数**：
```rust
pub fn filter_matches(
    matches: Vec<VerifyMatch>,
    filters: &FilterOptions
) -> Vec<VerifyMatch>
```

#### 3.2.4 统计生成器

**位置**：`src/commands/verify/stats.rs`

**职责**：
- 生成汇总统计信息
- 按不同维度分组统计

**数据结构**：
```rust
pub struct VerifySummary {
    pub total_files: usize,
    pub total_matches: usize,
    pub patterns_used: HashMap<String, usize>,
    pub by_category: HashMap<String, usize>,
    pub by_file_type: HashMap<String, usize>,
    pub by_pattern_type: HashMap<String, usize>,
}
```

#### 3.2.5 输出格式化器

**位置**：`src/commands/verify/output.rs`

**职责**：
- 格式化输出结果
- 支持多种输出格式

**核心函数**：
```rust
pub fn format_output(
    matches: &[VerifyMatch],
    summary: &VerifySummary,
    options: &OutputOptions
) -> Result<String>
```

### 3.3 与现有系统的集成

#### 3.3.1 配置系统集成

```rust
// 复用现有配置加载器
let loader = ConfigLoader::new()
    .with_project_config(config_path)
    .with_global_config(global_config_path);
let (_, project_config) = loader.load()?;
```

#### 3.3.2 解析器系统集成

```rust
// 复用现有解析器协调器
let parser = ParserCoordinator::new(&parser_config, &project_config)?;

// 复用现有解析逻辑
let units = parser.parse_file(&file)?;
```

#### 3.3.3 过滤器系统集成

```rust
// 复用现有内容过滤器
let content_filter = ContentFilter::from_config(&project_config)?;
let should_translate = content_filter.should_translate(&unit.content);
```

## 4. 实现方案

### 4.1 命令行接口设计

```rust
#[derive(Parser, Debug)]
pub struct VerifyArgs {
    /// 目标文件或目录路径
    #[arg(value_name = "PATH")]
    pub path: PathBuf,

    /// 配置文件路径
    #[arg(short = 'c', long = "config")]
    pub config: Option<PathBuf>,

    /// 全局配置文件路径
    #[arg(long = "global-config")]
    pub global_config: Option<PathBuf>,

    /// 显示详细匹配结果
    #[arg(short = 'd', long = "detailed")]
    pub detailed: bool,

    /// 按模式名称过滤
    #[arg(short = 'p', long = "pattern")]
    pub pattern: Option<String>,

    /// 按文件扩展名过滤
    #[arg(short = 'e', long = "extension")]
    pub extension: Option<String>,

    /// 按类别过滤 (error_handling, output, variables, properties, other)
    #[arg(short = 'k', long = "category")]
    pub category: Option<String>,

    /// 按文本内容过滤
    #[arg(short = 's', long = "search")]
    pub search: Option<String>,

    /// 输出格式 (table, json, csv)
    #[arg(short = 'f', long = "format", default_value = "table")]
    pub format: OutputFormat,

    /// 输出文件路径（默认输出到 stdout）
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// 显示统计信息
    #[arg(short = 'S', long = "stats", default_value = "true")]
    pub show_stats: bool,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}
```

### 4.2 主流程实现

```rust
pub fn execute_verify_command(args: VerifyArgs) -> Result<()> {
    // 1. 加载配置
    let loader = ConfigLoader::new();
    let loader = args.config
        .map(|p| loader.with_project_config(&p))
        .unwrap_or(loader);
    let loader = args.global_config
        .map(|p| loader.with_global_config(&p))
        .unwrap_or(loader);
    let (_, project_config) = loader.load()?;

    // 2. 创建解析器
    let parser = create_parser(&project_config)?;

    // 3. 扫描文件
    let files = scan_files(&args.path, &project_config)?;

    // 4. 提取匹配内容
    let mut matches = Vec::new();
    for file_entry in files {
        let file_matches = extract_from_file(&file_entry, &parser)?;
        matches.extend(file_matches);
    }

    // 5. 过滤结果
    let filtered_matches = filter_matches(matches, &args)?;

    // 6. 生成统计信息
    let summary = generate_summary(&filtered_matches, files.len())?;

    // 7. 格式化输出
    let output = format_output(&filtered_matches, &summary, &args)?;

    // 8. 写入输出
    write_output(&output, args.output)?;

    Ok(())
}
```

### 4.3 核心函数实现

#### 4.3.1 文件提取

```rust
fn extract_from_file(
    file_entry: &FileEntry,
    parser: &ParserCoordinator
) -> Result<Vec<VerifyMatch>> {
    let content = std::fs::read_to_string(&file_entry.path)?;
    let file = File::new(
        file_entry.path.clone(),
        content.clone(),
        "UTF-8"
    );

    let units = parser.parse_file(&file)?;

    let mut matches = Vec::new();
    for unit in units {
        let pattern_name = extract_pattern_name(&unit);
        let pattern_type = determine_pattern_type(&unit);
        let category = extract_category(&unit, &project_config);
        let raw_match = extract_raw_match(&unit, &content);

        matches.push(VerifyMatch {
            file_path: file_entry.path.clone(),
            pattern_name,
            pattern_type,
            category,
            extracted_text: unit.content.clone(),
            position: Position {
                line: unit.start_pos.line,
                column: unit.start_pos.column,
            },
            raw_match,
            metadata: extract_metadata(&unit),
        });
    }

    Ok(matches)
}
```

#### 4.3.2 模式名称提取

```rust
fn extract_pattern_name(unit: &TranslationUnit) -> String {
    // 从 TranslationUnit 的 id 字段提取模式名称
    // 格式示例：
    // - "comment://line:15" -> "line_comment"
    // - "custom:todo_pattern:42" -> "todo_pattern"
    // - "state_machine:i18n_with_default:78" -> "i18n_with_default"

    if let Some(prefix) = unit.id.split(':').next() {
        match prefix {
            "custom" => {
                unit.id.split(':').nth(1).unwrap_or("unknown").to_string()
            }
            "state_machine" => {
                unit.id.split(':').nth(1).unwrap_or("unknown").to_string()
            }
            "comment" => "comment".to_string(),
            "string" => "string_literal".to_string(),
            _ => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    }
}
```

#### 4.3.3 模式类型判断

```rust
fn determine_pattern_type(unit: &TranslationUnit) -> PatternType {
    if unit.id.starts_with("custom:") {
        PatternType::CustomRegex
    } else if unit.id.starts_with("state_machine:") {
        PatternType::StateMachine
    } else {
        PatternType::Builtin
    }
}
```

#### 4.3.4 原始匹配提取

```rust
fn extract_raw_match(unit: &TranslationUnit, content: &str) -> Option<String> {
    // 从原始内容中提取匹配的完整文本
    let lines: Vec<&str> = content.lines().collect();
    if unit.start_pos.line < lines.len() {
        let line = lines[unit.start_pos.line];
        if unit.start_pos.column <= line.len() && unit.end_pos.column <= line.len() {
            Some(line[unit.start_pos.column..unit.end_pos.column].to_string())
        } else {
            Some(line.to_string())
        }
    } else {
        None
    }
}
```

### 4.4 输出格式实现

#### 4.4.1 表格格式

```rust
fn format_table(
    matches: &[VerifyMatch],
    summary: &VerifySummary,
    detailed: bool
) -> String {
    use comfy_table::*;

    let mut table = Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);

    if detailed {
        table.set_header(vec![
            "Pattern",
            "Type",
            "Category",
            "File",
            "Line",
            "Extracted Text",
            "Raw Match"
        ]);
    } else {
        table.set_header(vec![
            "Pattern",
            "Type",
            "Category",
            "File",
            "Line",
            "Extracted Text"
        ]);
    }

    for m in matches {
        if detailed {
            table.add_row(vec![
                m.pattern_name.clone(),
                format!("{:?}", m.pattern_type),
                m.category.clone(),
                format_filename(&m.file_path),
                m.position.line.to_string(),
                truncate_text(&m.extracted_text, 40),
                m.raw_match.clone().unwrap_or("-".to_string()),
            ]);
        } else {
            table.add_row(vec![
                m.pattern_name.clone(),
                format!("{:?}", m.pattern_type),
                m.category.clone(),
                format_filename(&m.file_path),
                m.position.line.to_string(),
                truncate_text(&m.extracted_text, 60),
            ]);
        }
    }

    let mut output = table.to_string();

    // 添加统计信息
    output.push_str("\n\n");
    output.push_str("=== Summary ===\n");
    output.push_str(&format!("Total files: {}\n", summary.total_files));
    output.push_str(&format!("Total matches: {}\n", summary.total_matches));
    output.push_str("\nPatterns used:\n");
    for (pattern, count) in &summary.patterns_used {
        output.push_str(&format!("  - {}: {}\n", pattern, count));
    }

    output
}
```

#### 4.4.2 JSON 格式

```rust
fn format_json(
    matches: &[VerifyMatch],
    summary: &VerifySummary
) -> Result<String> {
    let output = serde_json::json!({
        "summary": summary,
        "matches": matches
    });
    Ok(serde_json::to_string_pretty(&output)?)
}
```

#### 4.4.3 CSV 格式

```rust
fn format_csv(
    matches: &[VerifyMatch],
    detailed: bool
) -> Result<String> {
    let mut wtr = csv::Writer::from_writer(vec![]);

    if detailed {
        wtr.write_record(&[
            "pattern", "type", "category", "file", "line", "column",
            "extracted_text", "raw_match"
        ])?;
    } else {
        wtr.write_record(&[
            "pattern", "type", "category", "file", "line", "extracted_text"
        ])?;
    }

    for m in matches {
        if detailed {
            wtr.write_record(&[
                &m.pattern_name,
                &format!("{:?}", m.pattern_type),
                &m.category,
                &m.file_path.display().to_string(),
                &m.position.line.to_string(),
                &m.position.column.to_string(),
                &m.extracted_text,
                &m.raw_match.clone().unwrap_or_default(),
            ])?;
        } else {
            wtr.write_record(&[
                &m.pattern_name,
                &format!("{:?}", m.pattern_type),
                &m.category,
                &m.file_path.display().to_string(),
                &m.position.line.to_string(),
                &m.extracted_text,
            ])?;
        }
    }

    let data = wtr.into_inner()?;
    String::from_utf8(data).map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))
}
```

## 5. 集成到现有 CLI

### 5.1 添加命令变体

在 `src/main.rs` 中：

```rust
#[derive(Subcommand)]
enum Commands {
    /// Translate source files
    Translate {
        // ... 现有参数
    },

    /// Initialize configuration
    Init {
        // ... 现有参数
    },

    /// Cache management
    Cache {
        // ... 现有参数
    },

    /// Validate configuration
    Validate {
        // ... 现有参数
    },

    /// Verify extraction rules (NEW)
    Verify(Box<VerifyArgs>),
}
```

### 5.2 添加命令处理逻辑

```rust
#[tokio::main]
async fn run() -> Result<()> {
    let cli = Cli::parse();

    // ... 配置加载和日志初始化 ...

    match cli.command {
        Some(Commands::Translate(args)) => {
            execute_translate(args).await?;
        }
        Some(Commands::Init(args)) => {
            execute_init(args)?;
        }
        Some(Commands::Cache(args)) => {
            execute_cache(args)?;
        }
        Some(Commands::Validate(args)) => {
            execute_validate(args)?;
        }
        Some(Commands::Verify(args)) => {
            execute_verify_command(*args)?;
        }
        None => {
            // 默认行为：翻译
            execute_translate(TranslateArgs::default()).await?;
        }
    }

    Ok(())
}
```

## 6. 使用示例

### 6.1 基本用法

#### 验证单个文件

```bash
translator verify src/main.rs
```

输出：
```
Pattern              | Type          | Category       | File           | Line | Extracted Text
--------------------|---------------|----------------|----------------|------|---------------------
todo_pattern        | CustomRegex   | other          | src/main.rs    | 15   | Fix this bug
i18n_with_default   | StateMachine  | other          | src/main.rs    | 42   | Welcome message
comment             | Builtin       | other          | src/main.rs    | 78   | This is a comment

=== Summary ===
Total files: 1
Total matches: 3
Patterns used:
  - todo_pattern: 1
  - i18n_with_default: 1
  - comment: 1
```

#### 验证整个目录

```bash
translator verify src/
```

### 6.2 使用过滤条件

#### 按模式名称过滤

```bash
translator verify src/ --pattern todo_pattern
```

#### 按文件扩展名过滤

```bash
translator verify src/ --extension rs
```

#### 按类别过滤

```bash
translator verify src/ --category error_handling
```

#### 按文本内容过滤

```bash
translator verify src/ --search "TODO"
```

#### 组合过滤

```bash
translator verify src/ --pattern todo_pattern --extension rs --search "fix"
```

### 6.3 使用详细模式

```bash
translator verify src/main.rs --detailed
```

输出：
```
Pattern              | Type          | Category       | File           | Line | Extracted Text       | Raw Match
--------------------|---------------|----------------|----------------|------|----------------------|-------------------------
todo_pattern        | CustomRegex   | other          | src/main.rs    | 15   | Fix this bug         | TODO: Fix this bug
i18n_with_default   | StateMachine  | other          | src/main.rs    | 42   | Welcome message      | t("Welcome message")
```

### 6.4 不同输出格式

#### JSON 格式

```bash
translator verify src/ --format json
```

```bash
translator verify src/ --format json --output results.json
```

输出：
```json
{
  "summary": {
    "total_files": 10,
    "total_matches": 45,
    "patterns_used": {
      "todo_pattern": 5,
      "i18n_with_default": 20,
      "comment": 15,
      "string_literal": 5
    },
    "by_category": {
      "other": 30,
      "error_handling": 10,
      "output": 5
    },
    "by_file_type": {
      "rs": 20,
      "js": 15,
      "ts": 10
    },
    "by_pattern_type": {
      "Builtin": 20,
      "CustomRegex": 5,
      "StateMachine": 20
    }
  },
  "matches": [
    {
      "file_path": "/path/to/src/main.rs",
      "pattern_name": "todo_pattern",
      "pattern_type": "CustomRegex",
      "category": "other",
      "extracted_text": "Fix this bug",
      "position": {
        "line": 15,
        "column": 1
      },
      "raw_match": "TODO: Fix this bug",
      "metadata": {}
    }
  ]
}
```

#### CSV 格式

```bash
translator verify src/ --format csv --output results.csv
```

输出：
```csv
pattern,type,category,file,line,extracted_text
todo_pattern,CustomRegex,other,src/main.rs,15,Fix this bug
i18n_with_default,StateMachine,other,src/main.rs,42,Welcome message
comment,Builtin,other,src/main.rs,78,This is a comment
```

### 6.5 使用自定义配置

```bash
translator verify src/ --config .translator.toml
```

```bash
translator verify src/ --config .translator.toml --global-config ~/.config/translator/config.toml
```

## 7. 测试计划

### 7.1 单元测试

#### 7.1.1 结果收集器测试

- 测试从 TranslationUnit 提取模式名称
- 测试判断模式类型
- 测试提取原始匹配内容
- 测试提取元数据

**测试文件**：`src/commands/verify/tests/collector_test.rs`

#### 7.1.2 结果过滤器测试

- 测试按模式名称过滤
- 测试按文件扩展名过滤
- 测试按类别过滤
- 测试按文本内容过滤
- 测试组合过滤

**测试文件**：`src/commands/verify/tests/filter_test.rs`

#### 7.1.3 统计生成器测试

- 测试生成总体统计
- 测试按模式统计
- 测试按类别统计
- 测试按文件类型统计

**测试文件**：`src/commands/verify/tests/stats_test.rs`

#### 7.1.4 输出格式化器测试

- 测试表格格式化
- 测试 JSON 格式化
- 测试 CSV 格式化
- 测试详细模式格式化

**测试文件**：`src/commands/verify/tests/output_test.rs`

### 7.2 集成测试

#### 7.2.1 基本功能测试

- 测试单文件验证
- 测试目录验证
- 测试过滤功能
- 测试不同输出格式

**测试文件**：`tests/verify_integration_tests.rs`

#### 7.2.2 规则类型测试

- 测试内置规则（注释、字符串等）
- 测试自定义正则规则
- 测试状态机规则
- 测试规则组合

**测试文件**：`tests/verify_rule_tests.rs`

#### 7.2.3 边界情况测试

- 测试空文件
- 测试无匹配文件
- 测试大文件
- 测试特殊字符处理

**测试文件**：`tests/verify_edge_case_tests.rs`

### 7.3 端到端测试

#### 7.3.1 完整流程测试

- 测试从命令行参数到输出结果的完整流程
- 测试与现有配置系统的集成
- 测试与现有解析器系统的集成

**测试文件**：`tests/e2e/verify_e2e_tests.rs`

#### 7.3.2 性能测试

- 测试大型项目的验证性能
- 测试内存使用情况

**测试文件**：`tests/e2e/verify_performance_tests.rs`

## 8. 实现步骤

### 阶段 1：基础框架搭建（优先级：高）

1. 创建 `src/commands/verify` 模块
2. 定义核心数据结构（VerifyMatch、VerifySummary 等）
3. 实现 `execute_verify_command` 主函数框架
4. 添加 `Verify` 命令到 CLI
5. 实现基本的单文件验证功能

**预计工作量**：2-3 天

### 阶段 2：结果处理功能（优先级：高）

1. 实现结果收集器（collector.rs）
2. 实现结果过滤器（filter.rs）
3. 实现统计生成器（stats.rs）
4. 实现表格输出格式（output.rs - table）
5. 实现目录扫描功能

**预计工作量**：3-4 天

### 阶段 3：输出格式扩展（优先级：中）

1. 实现 JSON 输出格式
2. 实现 CSV 输出格式
3. 实现详细模式
4. 实现输出到文件功能

**预计工作量**：2-3 天

### 阶段 4：测试和优化（优先级：高）

1. 编写单元测试
2. 编写集成测试
3. 编写端到端测试
4. 性能优化
5. 文档完善

**预计工作量**：3-4 天

### 阶段 5：发布准备（优先级：中）

1. 代码审查
2. 错误处理完善
3. 用户文档更新
4. 示例配置更新

**预计工作量**：1-2 天

**总预计工作量**：11-16 天

## 9. 依赖项

### 9.1 新增依赖

```toml
[dependencies]
# CSV 输出支持
csv = "1.3"

# 更好的表格输出（可选，用于替代 comfy_table）
tabled = "0.15"

# JSON 序列化（已有）
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 9.2 现有依赖复用

- `clap`：命令行参数解析
- `anyhow`：错误处理
- `serde`：序列化/反序列化
- `tokio`：异步运行时（如果需要）
- 现有的配置、解析器、扫描器模块

## 10. 风险和挑战

### 10.1 技术风险

1. **性能问题**：大型项目验证可能耗时较长
   - 缓解措施：实现增量验证、并行处理

2. **内存使用**：大文件或大量匹配可能占用大量内存
   - 缓解措施：流式处理、分批处理

3. **模式识别准确性**：从 TranslationUnit 提取模式名称可能不准确
   - 缓解措施：在 TranslationUnit 中添加更明确的元数据字段

### 10.2 实现挑战

1. **与现有系统的集成**：需要确保与现有配置、解析器系统无缝集成
   - 解决方案：仔细分析现有接口，遵循现有设计模式

2. **结果过滤逻辑**：需要实现灵活且高效的过滤逻辑
   - 解决方案：使用组合模式，支持链式过滤

3. **输出格式兼容性**：确保不同输出格式的一致性和正确性
   - 解决方案：定义统一的中间数据结构，使用测试验证

## 11. 未来扩展

### 11.1 功能扩展

1. **规则对比**：支持对比不同规则版本的提取结果
2. **规则覆盖率分析**：分析规则对不同文件类型的覆盖情况
3. **规则建议**：基于提取结果建议规则优化
4. **可视化报告**：生成交互式的 HTML 报告

### 11.2 集成扩展

1. **CI/CD 集成**：提供专门的 CI/CD 验证命令
2. **IDE 集成**：提供 IDE 插件，支持实时规则验证
3. **版本控制集成**：支持比较不同提交的提取结果

### 11.3 性能优化

1. **并行处理**：支持多线程并行验证
2. **增量验证**：只验证变更的文件
3. **缓存机制**：缓存验证结果，加速重复验证

## 12. 总结

本设计方案提供了一个完整的提取规则验证功能的实现计划，包括：

1. **清晰的功能定义**：明确了验证命令的功能范围和使用场景
2. **详细的架构设计**：说明了模块划分和职责分配
3. **具体的实现方案**：提供了核心代码结构和关键函数实现
4. **完整的使用示例**：展示了各种使用场景和输出格式
5. **系统的测试计划**：覆盖了单元测试、集成测试和端到端测试
6. **明确的实施步骤**：分为 5 个阶段，每个阶段都有明确的目标和预计工作量

该设计充分利用了现有系统的可复用组件，避免了重复开发，同时保持了与现有架构的一致性。实施该方案将为用户提供一个强大而灵活的规则验证工具，大大提升提取规则的调试和优化效率。