# 日志集成改进方案

## 概述

本文档详细说明了 Codebase Translate 项目中各模块的日志记录现状，并提供了完整的日志补充方案。目标是提高系统的可观测性、可调试性和可维护性。

## 当前状态

### 已集成日志的模块（26个）

| 模块 | 文件 | 日志覆盖 |
|------|------|----------|
| **日志系统** | logger/mod.rs, logger/config.rs | ✅ 完整 |
| **翻译模块** | translator/deeplx.rs, translator/tencent.rs, translator/batch.rs, translator/multi.rs, translator/llm/*.rs | ✅ 完整 |
| **写入模块** | writer/file.rs, writer/concurrent.rs | ✅ 完整 |
| **扫描模块** | scanner/walker.rs, scanner/gitignore.rs | ✅ 完整 |
| **工作流模块** | workflow/executor.rs, workflow/file_processor.rs | ✅ 完整 |
| **编码模块** | encoding/detector.rs, encoding/encoder.rs | ✅ 完整 |
| **解析模块** | parser/regex/*.rs, parser/coordinator/*.rs | ⚠️ 部分 |
| **配置模块** | config/loader.rs | ⚠️ 部分 |
| **缓存模块** | cache/binary.rs, cache/util.rs | ⚠️ 部分 |

### 未集成日志的模块（47+个）

| 模块类别 | 数量 | 优先级 |
|---------|------|--------|
| 报告器模块 | 4 | 🔴 高 |
| 配置模块 | 4 | 🔴 高 |
| 缓存模块 | 2 | 🔴 高 |
| 语言解析器 | 9 | 🟡 中 |
| 解析核心 | 5 | 🟡 中 |
| 解析策略 | 2 | 🟡 中 |
| 核心模块 | 2 | 🟡 中 |
| 工厂模块 | 1 | 🟡 中 |
| 解析查询 | 4 | 🟢 低 |
| 工具模块 | 1 | 🟢 低 |
| 其他解析器 | 4 | 🟢 低 |

**总体覆盖率**: ~36%

---

## 日志补充方案

### 一、高优先级模块（立即补充）

#### 1.1 报告器模块

**文件**: reporter/default.rs, reporter/progress.rs, reporter/stats.rs, reporter/trait.rs

**需要补充的日志**:

##### reporter/default.rs

```rust
use tracing::{debug, info, warn};

// 在 generate_text_report 方法中
info!(
    files = stats.processed_files,
    units = stats.translated_units,
    errors = stats.error_count,
    "Generating translation report"
);

// 在 report_file 方法中
debug!(
    file = %file_path,
    format = ?format,
    "Reporting file"
);

// 在 record_error 方法中
warn!(
    file = %file_path,
    error = %error,
    "Recording error"
);
```

##### reporter/progress.rs

```rust
use tracing::{debug, info};

// 在 update_progress 方法中
debug!(
    current = current,
    total = total,
    file = %file_path,
    "Updating progress"
);

// 在 finish 方法中
info!(
    duration_ms = duration.as_millis(),
    files = processed_files,
    "Progress finished"
);
```

##### reporter/stats.rs

```rust
use tracing::{debug, info};

// 在 record_file_processed 方法中
debug!(
    file = %file_path,
    units = units_count,
    "Recording file processing"
);

// 在 record_translation 方法中
debug!(
    source_len = source.len(),
    target_len = target.len(),
    "Recording translation"
);

// 在 finalize 方法中
info!(
    total_files = self.total_files,
    processed_files = self.processed_files,
    total_units = self.total_units,
    translated_units = self.translated_units,
    api_calls = self.api_call_count,
    cache_hits = self.cache_hit_count,
    cache_misses = self.cache_miss_count,
    "Finalizing statistics"
);
```

##### reporter/trait.rs

```rust
use tracing::debug;

// 在 report_file 方法中
debug!(
    file = %file_path,
    format = ?format,
    "Reporter: reporting file"
);

// 在 report_summary 方法中
debug!(
    format = ?format,
    "Reporter: generating summary"
);
```

---

#### 1.2 配置模块

**文件**: config/global.rs, config/project.rs, config/env.rs, config/mod.rs

**需要补充的日志**:

##### config/global.rs

```rust
use tracing::{debug, info, warn};

// 在 merge 方法中
debug!(
    logging_level = %self.logging.level,
    logging_output = %self.logging.output,
    "Merging global configuration"
);

// 在 validate 方法中
info!(
    deeplx_api_url = %self.deeplx.api_url,
    tencent_region = %self.tencent.region,
    "Validating global configuration"
);

// 在从环境变量加载时
debug!(
    var = "DEEPLX_API_URL",
    has_value = !self.deeplx.api_url.is_empty(),
    "Loading from environment"
);

// 配置警告
if self.deeplx.api_url.is_empty() {
    warn!("DeepLX API URL not configured");
}
```

##### config/project.rs

```rust
use tracing::{debug, info};

// 在 validate 方法中
info!(
    provider = %self.translate.provider,
    cache_type = %self.cache.cache_type,
    "Validating project configuration"
);

// 在从文件加载时
debug!(
    file = %path,
    include_patterns = self.include.patterns.len(),
    exclude_patterns = self.exclude.patterns.len(),
    "Loaded project configuration"
);
```

##### config/env.rs

```rust
use tracing::{debug, warn};

// 在 load_env_file 方法中
debug!(
    file = %path,
    "Loading environment file"
);

// 在 expand_env_vars 方法中
debug!(
    vars_count = env_vars.len(),
    "Expanding environment variables"
);

// 在解析环境变量时
if let Some(value) = std::env::var(key).ok() {
    debug!(key = %key, "Found environment variable");
} else {
    debug!(key = %key, "Environment variable not found");
}
```

##### config/mod.rs

```rust
use tracing::{info, debug};

// 在模块初始化时
info!("Configuration module initialized");

// 在配置合并时
debug!(
    global_config = ?global,
    project_config = ?project,
    "Merging configurations"
);
```

---

#### 1.3 缓存模块

**文件**: cache/mod.rs, cache/trait.rs

**需要补充的日志**:

##### cache/mod.rs

```rust
use tracing::{debug, info, warn};

// 在模块初始化时
info!(
    cache_type = %cache_type,
    cache_dir = %cache_dir,
    "Cache module initialized"
);
```

##### cache/trait.rs

```rust
use tracing::{debug, info, warn, error};

// 在 get 方法中
debug!(
    key = %key,
    "Cache: retrieving entry"
);

// 在 set 方法中
debug!(
    key = %key,
    value_len = value.len(),
    "Cache: storing entry"
);

// 在 remove 方法中
debug!(
    key = %key,
    "Cache: removing entry"
);

// 在 clear 方法中
info!("Cache: clearing all entries");

// 在 stats 方法中
debug!(
    entries = stats.entry_count,
    size_bytes = stats.size_bytes,
    "Cache: statistics"
);

// 在缓存命中时
debug!(key = %key, "Cache hit");

// 在缓存未命中时
debug!(key = %key, "Cache miss");

// 在缓存错误时
error!(
    key = %key,
    error = %e,
    "Cache operation failed"
);
```

---

### 二、中优先级模块（近期补充）

#### 2.1 语言解析器模块

**文件**: parser/languages/{python,java,javascript,typescript,go,rust,c,cpp,csharp}/parser.rs

**需要补充的日志**:

所有语言解析器统一模式：

```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument(skip(self, content))]
pub fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
    let start = std::time::Instant::now();
    
    info!(
        file = %file.path,
        size = file.content.len(),
        "Parsing file"
    );

    // 解析语法树
    let tree = match self.parse_tree(&file.content) {
        Ok(tree) => {
            debug!(
                duration_ms = start.elapsed().as_millis(),
                "Syntax tree parsed successfully"
            );
            tree
        }
        Err(e) => {
            error!(
                file = %file.path,
                error = %e,
                "Failed to parse syntax tree"
            );
            return Err(e);
        }
    };

    // 提取翻译单元
    let units = match self.extract_units(&tree, &file.content) {
        Ok(units) => {
            info!(
                file = %file.path,
                units = units.len(),
                duration_ms = start.elapsed().as_millis(),
                "Extracted translation units"
            );
            units
        }
        Err(e) => {
            warn!(
                file = %file.path,
                error = %e,
                "Failed to extract some units, continuing with partial results"
            );
            return Err(e);
        }
    };

    Ok(units)
}

#[instrument(skip(self, tree, content))]
fn extract_units(&self, tree: &Tree, content: &str) -> Result<Vec<TranslationUnit>> {
    let mut units = Vec::new();
    
    debug!(
        root_node = tree.root_node().kind(),
        "Starting unit extraction"
    );

    // 使用查询提取
    let query = self.queries().comments();
    let matches = self.query_executor().execute(query, tree, content)?;
    
    for match_result in matches {
        debug!(
            capture = %match_result.capture_name,
            text_len = match_result.text.len(),
            "Processing capture"
        );
        
        // 处理匹配结果
        if let Some(unit) = self.process_match(&match_result, content) {
            units.push(unit);
        }
    }

    debug!(
        total_units = units.len(),
        comments = units.iter().filter(|u| u.node_type == NodeType::Comment).count(),
        docstrings = units.iter().filter(|u| u.node_type == NodeType::DocString).count(),
        "Extraction completed"
    );

    Ok(units)
}
```

---

#### 2.2 解析核心模块

**文件**: parser/core/extractor.rs, parser/core/language_parser.rs, parser/core/query_executor.rs, parser/core/string_processor.rs, parser/core/position_tracker.rs

**需要补充的日志**:

##### parser/core/extractor.rs

```rust
use tracing::{debug, info, instrument};

#[instrument(skip(self, node, content))]
pub fn extract(&self, node: &Node, content: &str) -> Result<ExtractionCandidate> {
    let text = node.utf8_text(content.as_bytes())?;
    
    debug!(
        node_type = node.kind(),
        text_len = text.len(),
        "Extracting candidate"
    );

    // 清理文本
    let cleaned = self.clean_text(text);
    
    debug!(
        original_len = text.len(),
        cleaned_len = cleaned.len(),
        "Text cleaned"
    );

    Ok(ExtractionCandidate {
        text: cleaned,
        position: Position::from(node),
        node_type: self.determine_node_type(node),
    })
}
```

##### parser/core/language_parser.rs

```rust
use tracing::{debug, info};

// 在 parse_tree 方法中
debug!(
    content_len = content.len(),
    "Parsing syntax tree"
);

// 在 extract_comments 方法中
debug!(
    query_name = "comments",
    "Extracting comments"
);

// 在 extract_docstrings 方法中
debug!(
    query_name = "docstrings",
    "Extracting docstrings"
);
```

##### parser/core/query_executor.rs

```rust
use tracing::{debug, instrument};

#[instrument(skip(self, tree, content))]
pub fn execute<'a>(
    &self,
    query: &Query,
    tree: &'a Tree,
    content: &'a str,
) -> Result<Vec<QueryMatch<'a>>> {
    let mut cursor = QueryCursor::new();
    let mut matches = Vec::new();
    
    debug!(
        query_pattern_count = query.pattern_count(),
        "Executing query"
    );

    for match_result in cursor.matches(query, tree.root_node(), content.as_bytes()) {
        for capture in match_result.captures {
            let text = capture.node.utf8_text(content.as_bytes())?;
            
            debug!(
                capture_name = %self.query.capture_name(capture.index as usize).unwrap_or("unknown"),
                node_kind = capture.node.kind(),
                text_len = text.len(),
                "Found capture"
            );

            matches.push(QueryMatch {
                capture_name: self.query.capture_name(capture.index as usize)
                    .unwrap_or("unknown")
                    .to_string(),
                text,
                start_pos: Position::from(capture.node.start_position()),
                end_pos: Position::from(capture.node.end_position()),
                node: capture.node,
            });
        }
    }

    debug!(
        total_matches = matches.len(),
        "Query execution completed"
    );

    Ok(matches)
}
```

##### parser/core/string_processor.rs

```rust
use tracing::{debug};

// 在 clean_comment 方法中
debug!(
    original = %text,
    comment_type = ?comment_type,
    "Cleaning comment"
);

// 在 extract_placeholders 方法中
debug!(
    placeholders_found = placeholders.len(),
    "Extracted placeholders"
);

// 在 has_code_patterns 方法中
debug!(
    has_code = result,
    "Checking for code patterns"
);
```

##### parser/core/position_tracker.rs

```rust
use tracing::{debug};

// 在 track 方法中
debug!(
    start = ?start,
    end = ?end,
    "Tracking position"
);
```

---

#### 2.3 解析策略模块

**文件**: parser/strategy.rs, parser/filter.rs

**需要补充的日志**:

##### parser/strategy.rs

```rust
use tracing::{debug, info};

// 在 should_extract 方法中
debug!(
    node_type = ?node_type,
    strategy_enabled = enabled,
    "Checking extraction strategy"
);

// 在 apply_strategy 方法中
info!(
    strategy = ?self,
    "Applying extraction strategy"
);
```

##### parser/filter.rs

```rust
use tracing::{debug, warn};

// 在 should_translate 方法中
debug!(
    text = %text,
    text_len = text.len(),
    "Checking if text should be translated"
);

// 在检查排除关键词时
if self.config.exclude_keywords.iter().any(|kw| text.contains(kw)) {
    debug!(
        text = %text,
        reason = "excluded_keyword",
        "Text filtered out"
    );
    return false;
}

// 在检查最小长度时
if text.len() < self.config.min_length {
    debug!(
        text = %text,
        length = text.len(),
        min_length = self.config.min_length,
        reason = "too_short",
        "Text filtered out"
    );
    return false;
}

// 在过滤警告时
warn!(
    text = %text,
    reason = %reason,
    "Text filtered out"
);
```

---

#### 2.4 核心模块

**文件**: core/models.rs, core/error.rs

**需要补充的日志**:

##### core/models.rs

```rust
use tracing::{debug};

// 在 TranslationUnit::new 方法中
debug!(
    original_len = original.len(),
    node_type = ?node_type,
    "Creating translation unit"
);

// 在 CacheEntry::new 方法中
debug!(
    key = %key,
    value_len = value.len(),
    "Creating cache entry"
);
```

##### core/error.rs

```rust
use tracing::{error};

// 在错误发生时
error!(
    error_type = std::any::type_name::<Self>(),
    error_message = %self,
    "Translation error occurred"
);
```

---

#### 2.5 工厂模块

**文件**: factory/mod.rs

**需要补充的日志**:

```rust
use tracing::{debug, info};

// 在 create_cache 方法中
info!(
    cache_type = %cache_config.cache_type,
    cache_dir = %cache_dir,
    "Creating cache instance"
);

// 在 create_translator 方法中
info!(
    provider = %project_config.translate.provider,
    "Creating translator instance"
);

// 在 create_parser 方法中
info!(
    parser_config = ?parser_config,
    "Creating parser coordinator"
);

// 在 create_writer 方法中
info!(
    dry_run = project_config.writer.dry_run,
    backup = project_config.writer.backup,
    "Creating file writer"
);
```

---

### 三、低优先级模块（可选补充）

#### 3.1 解析查询模块

**文件**: parser/queries/builder.rs, parser/queries/comment_queries.rs, parser/queries/function_queries.rs, parser/queries/string_queries.rs

**需要补充的日志**:

```rust
use tracing::{debug};

// 在构建查询时
debug!(
    language = %language,
    pattern_count = patterns.len(),
    "Building query"
);

// 在查询执行时
debug!(
    query_name = %query_name,
    "Executing query"
);
```

---

#### 3.2 工具模块

**文件**: utils/hash.rs

**需要补充的日志**:

```rust
use tracing::{trace};

// 在 calculate_hash 方法中（仅在 trace 级别）
trace!(
    content_len = content.len(),
    "Calculating hash"
);
```

---

#### 3.3 其他解析器

**文件**: parser/regex_parsers/fallback.rs, parser/regex_parsers/html.rs, parser/regex_parsers/shell.rs, parser/regex_parsers/sql.rs

**需要补充的日志**:

```rust
use tracing::{debug, warn};

// 在回退解析器触发时
warn!(
    file = %file.path,
    reason = "no_tree_sitter_parser",
    "Using fallback parser"
);

// 在解析特殊格式时
debug!(
    format = %format,
    file = %file.path,
    "Parsing special format"
);
```

---

## 日志级别规范

### Trace 级别
- 详细的执行流程
- 每个翻译单元的处理细节
- 查询执行的每个匹配
- 字符串处理的每个步骤

### Debug 级别
- 解析器选择过程
- 提取结果详情
- 配置加载过程
- 缓存操作详情
- 查询执行统计

### Info 级别
- 文件处理开始/完成
- 翻译成功/失败
- 组件创建
- 重要配置变更
- 统计数据汇总

### Warn 级别
- 配置回退
- 解析失败但可继续
- 缓存未命中
- 部分翻译失败
- 使用回退方案

### Error 级别
- 翻译失败
- 文件读写错误
- 配置验证失败
- 网络请求失败
- 严重的解析错误

---

## 结构化日志示例

### 基本日志
```rust
tracing::info!("Processing file");
```

### 带字段的日志
```rust
tracing::info!(
    file = %file_path,
    size = file_size,
    "Processing file"
);
```

### 带调试信息的日志
```rust
tracing::debug!(
    file = %file_path,
    units = extracted_units.len(),
    duration_ms = start.elapsed().as_millis(),
    "Parsed file successfully"
);
```

### 错误日志
```rust
tracing::error!(
    file = %file_path,
    error = %e,
    "Failed to parse file"
);
```

### 性能监控
```rust
let start = std::time::Instant::now();
// ... 操作 ...
tracing::debug!(
    duration_ms = start.elapsed().as_millis(),
    "Operation completed"
);
```

---

## 函数追踪

使用 `#[instrument]` 宏自动记录函数调用：

```rust
use tracing::instrument;

#[instrument(skip(self, content))]
pub fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
    // 函数进入和退出会自动记录
    // 参数会自动记录（除了 skip 的）
    // 返回值会自动记录
}
```

---

## 实施计划

### 阶段一：高优先级（1-2周）
1. ✅ 报告器模块（4个文件）
2. ✅ 配置模块（4个文件）
3. ✅ 缓存模块（2个文件）

### 阶段二：中优先级（2-3周）
4. ✅ 语言解析器模块（9个文件）
5. ✅ 解析核心模块（5个文件）
6. ✅ 解析策略模块（2个文件）
7. ✅ 核心模块（2个文件）
8. ✅ 工厂模块（1个文件）

### 阶段三：低优先级（可选，1-2周）
9. ⭕ 解析查询模块（4个文件）
10. ⭕ 工具模块（1个文件）
11. ⭕ 其他解析器（4个文件）

---

## 验证方法

### 单元测试
每个模块的日志功能应包含单元测试：

```rust
#[test]
fn test_logging() {
    // 验证日志输出
}
```

### 集成测试
运行完整的翻译流程，验证日志输出：

```bash
cargo test --all
RUST_LOG=debug cargo run -- --config test.toml translate .
```

### 日志审查
检查日志输出是否符合预期：
- 日志级别是否正确
- 日志信息是否完整
- 性能影响是否可接受

---

## 性能考虑

### 避免过度日志
- 在性能敏感的代码路径中，使用 `trace` 或 `debug` 级别
- 使用条件日志记录：
  ```rust
  if log::log_enabled!(log::Level::Debug) {
      debug!(details = expensive_operation(), "Debug info");
  }
  ```

### 异步日志
- 使用 `tracing` 的异步支持
- 文件日志使用非阻塞写入（已实现）

### 日志格式
- 生产环境使用 `json` 或 `compact` 格式
- 开发环境使用 `pretty` 格式

---

## 相关文档

- [日志需求规范](./spec/logging-requirements.md)
- [项目规则](../AGENTS.md)
- [代码规范](../docs/archive/unsafe.md)
- [动态分发文档](../docs/archive/dynamic.md)

---

## 版本历史

| 版本 | 日期 | 作者 | 说明 |
|------|------|------|------|
| 1.0 | 2026-03-18 | AI Assistant | 初始版本，完整的日志补充方案 |
