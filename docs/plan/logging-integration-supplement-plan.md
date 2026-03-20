# 日志功能补充计划

## 概述

本文档基于当前项目的实际情况，详细分析了需要补充日志集成的模块，并提供了具体的实施计划。

**当前状态：**
- 日志系统基础完善（使用 tracing 框架）
- 日志覆盖率约 18%（9/50 个模块）
- 核心功能模块（工作流、缓存、写入器）缺少关键日志
- 报告器模块已完整集成，可作为参考模板

**目标：**
- 将日志覆盖率提升至 80% 以上
- 重点补充核心模块的日志
- 统一日志级别和格式规范

---

## 一、高优先级模块（P0 - 立即补充）

### 1.1 工作流模块

#### workflow/executor.rs

**当前状态：** 有 `info` 和 `error` 导入，但无实际日志调用

**需要补充的日志：**

```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn execute(&self) -> Result<WorkflowResult> {
    info!(
        root_path = %self.workflow_config.root_path,
        include_patterns = self.workflow_config.include_patterns.len(),
        exclude_patterns = self.workflow_config.exclude_patterns.len(),
        "Starting translation workflow"
    );

    let start = std::time::Instant::now();

    // 扫描文件
    info!("Scanning files for translation");
    let files = self.scan_files()?;
    info!(
        files_count = files.len(),
        "File scan completed"
    );

    // 处理文件
    debug!(
        files_count = files.len(),
        "Processing files"
    );

    for file in files {
        match self.process_file(&file).await {
            Ok(result) => {
                debug!(
                    file = %file.path,
                    units = result.total_units,
                    "File processed successfully"
                );
            }
            Err(e) => {
                warn!(
                    file = %file.path,
                    error = %e,
                    "Failed to process file, continuing"
                );
            }
        }
    }

    let duration = start.elapsed();
    info!(
        duration_ms = duration.as_millis(),
        files_processed = result.files_processed,
        "Workflow execution completed"
    );

    Ok(result)
}
```

**日志点：**
1. 工作流启动（info）
2. 文件扫描开始和完成（info）
3. 文件处理进度（debug）
4. 文件处理失败（warn）
5. 工作流完成（info，包含耗时）

---

#### workflow/file_processor.rs

**当前状态：** 已有部分日志（debug、info），但不够完整

**需要补充的日志：**

```rust
// 在 process 方法中
pub fn process(&self, file_path: &Path, modified_time: i64) -> Result<FileProcessResult> {
    info!(
        file = %file_path.display(),
        modified_time = modified_time,
        "Processing file"
    );

    let content = std::fs::read(file_path)?;
    let file_hash = calculate_hash(&content);

    // 缓存检查
    debug!(
        file_hash = %file_hash,
        "Checking cache"
    );

    let cached_entry = self.cache.get(&file_hash)?;

    if let Some(entry) = cached_entry {
        if entry.is_valid(modified_time) && entry.is_translated {
            info!(
                file = %file_path.display(),
                "Cache hit - file already translated"
            );
            result.cached_files = 1;
            return Ok(result);
        } else {
            debug!(
                file = %file_path.display(),
                "Cache expired or file modified, re-translating"
            );
        }
    } else {
        debug!(
            file = %file_path.display(),
            "Cache miss"
        );
    }

    // 编码检测
    let encoding_result = self.detector.detect_bytes(&content)?;
    let encoding = encoding_result.encoding;

    if encoding != "UTF-8" {
        info!(
            file = %file_path.display(),
            original_encoding = %encoding,
            "Converting encoding to UTF-8"
        );
    }

    // 解析
    let file = File::new(file_path.to_path_buf(), utf8_content.clone(), "UTF-8");
    let units = self.parser.parse_file(&file)?;
    result.total_units = units.len();

    info!(
        file = %file_path.display(),
        total_units = result.total_units,
        translatable_units = units.iter().filter(|u| u.should_translate).count(),
        "File parsed"
    );

    // 翻译
    if num_to_translate > 0 {
        info!(
            file = %file_path.display(),
            units_to_translate = num_to_translate,
            "Translating units"
        );

        let translated_texts = self.translator.translate_batch(
            &texts,
            &self.project_config.translate.target_lang
        )?;

        info!(
            file = %file_path.display(),
            translated_units = translated_texts.len(),
            "Translation completed"
        );
    }

    // 写入
    if !self.project_config.writer.dry_run {
        info!(
            file = %file_path.display(),
            "Writing file"
        );
        // ... 写入逻辑
        info!(
            file = %file_path.display(),
            "File written successfully"
        );
    } else {
        info!(
            file = %file_path.display(),
            "Dry run mode - skipping file write"
        );
    }

    // 更新缓存
    debug!(
        file = %file_path.display(),
        "Updating cache"
    );
    self.cache.set(&cache_entry)?;

    info!(
        file = %file_path.display(),
        total_units = result.total_units,
        translated_units = result.translated_units,
        "File processing completed"
    );

    Ok(result)
}
```

**日志点：**
1. 文件处理开始（info）
2. 缓存检查（debug/info）
3. 编码转换（info）
4. 解析结果（info）
5. 翻译过程（info）
6. 文件写入（info）
7. 缓存更新（debug）
8. 处理完成（info）

---

### 1.2 缓存模块

#### cache/mod.rs

**当前状态：** 无日志

**需要补充的日志：**

```rust
use tracing::{debug, info};

// 在模块级别
info!(
    cache_type = %cache_type,
    cache_dir = %cache_dir,
    "Cache module initialized"
);
```

---

#### cache/trait.rs

**当前状态：** 有注释说明需要日志，但无实际实现

**需要补充的日志：**

```rust
use tracing::{debug, info, warn, error};

// 实现层面应该记录以下操作
pub trait Cache: Send + Sync {
    fn get(&self, file_hash: &str) -> Result<Option<CacheEntry>> {
        debug!(
            file_hash = %file_hash,
            "Cache: retrieving entry"
        );
        // ... 实现逻辑
    }

    fn set(&self, entry: &CacheEntry) -> Result<()> {
        debug!(
            file_hash = %entry.file_hash,
            file_path = %entry.file_path,
            "Cache: storing entry"
        );
        // ... 实现逻辑
    }

    fn invalidate(&self, file_hash: &str) -> Result<()> {
        debug!(
            file_hash = %file_hash,
            "Cache: invalidating entry"
        );
        // ... 实现逻辑
    }

    fn clear(&self) -> Result<()> {
        info!("Cache: clearing all entries");
        // ... 实现逻辑
    }

    fn cleanup_orphaned(&self, existing_hashes: HashMap<String, bool>) -> Result<usize> {
        info!(
            existing_hashes_count = existing_hashes.len(),
            "Cache: starting orphaned cleanup"
        );
        // ... 实现逻辑
        info!(
            removed_count = count,
            "Cache: orphaned cleanup completed"
        );
        Ok(count)
    }

    fn stats(&self) -> Result<CacheStats> {
        debug!("Cache: retrieving statistics");
        // ... 实现逻辑
    }
}
```

**日志点：**
1. 缓存初始化（info）
2. 缓存读取（debug）
3. 缓存写入（debug）
4. 缓存失效（debug）
5. 缓存清理（info）
6. 孤立文件清理（info）
7. 缓存统计（debug）

---

#### cache/binary.rs

**当前状态：** 有 `debug` 导入，但无实际日志调用

**需要补充的日志：**

```rust
// 在 BinaryCache 实现中添加日志
impl BinaryCache {
    pub fn new(config: CacheConfig, project_path: &str) -> Result<Self> {
        info!(
            cache_dir = %cache_dir,
            cache_type = %config.cache_type,
            "Creating binary cache"
        );
        // ... 初始化逻辑
    }

    fn get(&self, file_hash: &str) -> Result<Option<CacheEntry>> {
        debug!(
            file_hash = %file_hash,
            cache_file = %cache_file.display(),
            "Reading from cache file"
        );
        // ... 读取逻辑
    }

    fn set(&self, entry: &CacheEntry) -> Result<()> {
        debug!(
            file_hash = %entry.file_hash,
            cache_file = %cache_file.display(),
            "Writing to cache file"
        );
        // ... 写入逻辑
    }
}
```

---

### 1.3 写入模块

#### writer/concurrent.rs

**当前状态：** 有 `error` 导入，但只在 panic 时使用

**需要补充的日志：**

```rust
use tracing::{debug, info, warn, error};

pub async fn write_files(&self, files: Vec<(File, Vec<TranslationUnit>)>) -> Vec<WriteResult> {
    info!(
        files_count = files.len(),
        max_concurrent = self.max_concurrent,
        "Starting concurrent file writes"
    );

    // ... 写入逻辑

    for result in results {
        if result.success {
            debug!(
                file = %result.path.display(),
                units_written = result.units_written,
                "File written successfully"
            );
        } else {
            error!(
                file = %result.path.display(),
                error = %result.error.as_ref().unwrap_or(&"unknown".to_string()),
                "Failed to write file"
            );
        }
    }

    let stats = ConcurrentWriteStats::from_results(&results);
    info!(
        total_files = stats.total_files,
        success_count = stats.success_count,
        failure_count = stats.failure_count,
        success_rate = stats.success_rate(),
        "Concurrent write completed"
    );

    results
}
```

**日志点：**
1. 并发写入开始（info）
2. 单个文件写入成功（debug）
3. 单个文件写入失败（error）
4. 并发写入完成（info，含统计）

---

#### writer/file.rs

**当前状态：** 已有较好的日志覆盖（info、debug、warn、error）

**需要补充的日志：**

```rust
// 在 create_backup 方法中
info!(
    file = %file_path.display(),
    backup = %backup_path.display(),
    "Creating backup"
);

// 在 write_file_atomically 方法中
debug!(
    file = %file_path.display(),
    temp_file = %temp_path.display(),
    "Creating temporary file"
);

debug!(
    file = %file_path.display(),
    "Preserving file metadata"
);

debug!(
    file = %file_path.display(),
    "Performing atomic rename"
);

// 在 write_preview 方法中
info!(
    file = %file.path.display(),
    units_count = units.len(),
    "Previewing translations"
);
```

---

### 1.4 工厂模块

#### factory/mod.rs

**当前状态：** 无日志

**需要补充的日志：**

```rust
use tracing::{debug, info};

pub fn create_cache(cache_config: &CacheConfig, project_path: &str) -> Result<Box<dyn Cache>> {
    info!(
        cache_type = %cache_config.cache_type,
        cache_dir = %project_path,
        "Creating cache instance"
    );
    let cache = Box::new(BinaryCache::new(cache_config.clone(), project_path)?);
    debug!("Cache instance created successfully");
    Ok(cache)
}

pub fn create_translator(
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
) -> Result<TranslationService> {
    info!(
        provider = %project_config.translate.provider,
        "Creating translator instance"
    );
    let translator = TranslationService::new(translator_config)?;
    debug!("Translator instance created successfully");
    Ok(translator)
}

pub fn create_parser(project_config: &ProjectConfig) -> Result<ParserCoordinator> {
    info!(
        extract_comments = project_config.extraction.comments,
        extract_docstrings = project_config.extraction.doc_strings,
        extract_strings = project_config.extraction.format_strings,
        "Creating parser coordinator"
    );
    let parser = ParserCoordinator::from_project_config(parser_config, project_config)?;
    debug!("Parser coordinator created successfully");
    Ok(parser)
}

pub fn create_writer(
    project_config: &ProjectConfig,
    project_path: Option<&str>,
) -> Result<FileWriter> {
    info!(
        dry_run = project_config.writer.dry_run,
        backup = project_config.writer.backup,
        "Creating file writer"
    );
    let writer = FileWriter::with_project_path(writer_config, project_path)?;
    debug!("File writer created successfully");
    Ok(writer)
}
```

**日志点：**
1. 创建缓存（info + debug）
2. 创建翻译器（info + debug）
3. 创建解析器（info + debug）
4. 创建写入器（info + debug）

---

## 二、中优先级模块（P1 - 近期补充）

### 2.1 解析器协调器

#### parser/coordinator/coordinator.rs

**当前状态：** 有部分日志（warn、debug），但不够完整

**需要补充的日志：**

```rust
use tracing::{debug, info, warn};

impl ParserCoordinator {
    pub fn new(config: ParserConfig, ...) -> Result<Self> {
        info!(
            tree_sitter_parsers_count = parsers.len(),
            "Creating parser coordinator"
        );
        // ... 初始化逻辑
        debug!("Parser coordinator created successfully");
        Ok(Self { ... })
    }

    pub fn parse_file(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        info!(
            file = %file.path.display(),
            file_size = file.content.len(),
            "Parsing file"
        );

        // 选择解析器
        debug!(
            parser_type = ?parser_type,
            "Selected parser"
        );

        // 解析
        let units = match parser.parse(file) {
            Ok(units) => {
                info!(
                    file = %file.path.display(),
                    total_units = units.len(),
                    "File parsed successfully"
                );
                units
            }
            Err(e) => {
                warn!(
                    file = %file.path.display(),
                    error = %e,
                    "Parser failed, trying fallback"
                );
                return self.fallback_parse(file);
            }
        };

        // 应用提取模式
        debug!(
            custom_patterns_count = self.custom_pattern_matchers.len(),
            state_machine_patterns_count = self.state_machine_matchers.len(),
            "Applying extraction patterns"
        );

        // 过滤
        debug!(
            before_filter = all_units.len(),
            after_filter = units.len(),
            "Applied content filter"
        );

        Ok(units)
    }
}
```

---

### 2.2 核心解析模块

#### parser/core/language_parser.rs

**当前状态：** 有 `debug` 导入，但无实际日志调用

**需要补充的日志：**

```rust
pub fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
    debug!(
        file = %file.path.display(),
        "Parsing with language parser"
    );

    let tree = self.parse_tree(&file.content)?;
    debug!(
        root_node = tree.root_node().kind(),
        "Syntax tree parsed"
    );

    let units = self.extract_units(&tree, &file.content)?;
    debug!(
        units_count = units.len(),
        "Units extracted"
    );

    Ok(units)
}
```

#### parser/core/query_executor.rs

**当前状态：** 无日志

**需要补充的日志：**

```rust
use tracing::{debug, instrument};

#[instrument(skip(self, tree, content))]
pub fn execute<'a>(
    &self,
    query: &Query,
    tree: &'a Tree,
    content: &'a str,
) -> Result<Vec<QueryMatch<'a>>> {
    debug!(
        query_pattern_count = query.pattern_count(),
        "Executing tree-sitter query"
    );

    let mut matches = Vec::new();

    for match_result in cursor.matches(query, tree.root_node(), content.as_bytes()) {
        for capture in match_result.captures {
            let capture_name = self.query.capture_name(capture.index as usize)
                .unwrap_or("unknown");

            debug!(
                capture_name = %capture_name,
                node_kind = capture.node.kind(),
                "Found query capture"
            );

            // ... 处理逻辑
        }
    }

    debug!(
        total_matches = matches.len(),
        "Query execution completed"
    );

    Ok(matches)
}
```

#### parser/core/extractor.rs

**当前状态：** 无日志

**需要补充的日志：**

```rust
use tracing::{debug, instrument};

#[instrument(skip(self, node, content))]
pub fn extract(&self, node: &Node, content: &str) -> Result<ExtractionCandidate> {
    debug!(
        node_type = node.kind(),
        "Extracting candidate from node"
    );

    let text = node.utf8_text(content.as_bytes())?;
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

---

### 2.3 语言解析器

**文件：** `parser/languages/{rust,python,go,java,javascript,typescript,c,cpp,csharp}/parser.rs`

**需要补充的日志：**

```rust
use tracing::{debug, info};

impl LanguageParser {
    #[instrument(skip(self, content))]
    pub fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        info!(
            file = %file.path.display(),
            language = %self.language(),
            "Parsing file with language parser"
        );

        let tree = self.parse_syntax_tree(&file.content)?;
        debug!("Syntax tree parsed");

        let units = self.extract_comments(&tree, &file.content)?;
        debug!(
            comments_count = units.len(),
            "Comments extracted"
        );

        if self.config.extract_docstrings {
            let docstrings = self.extract_docstrings(&tree, &file.content)?;
            debug!(
                docstrings_count = docstrings.len(),
                "Docstrings extracted"
            );
            units.extend(docstrings);
        }

        info!(
            file = %file.path.display(),
            total_units = units.len(),
            "Language parsing completed"
        );

        Ok(units)
    }
}
```

---

### 2.4 过滤和策略模块

#### parser/filter.rs

**当前状态：** 有 `debug` 导入，但无实际日志调用

**需要补充的日志：**

```rust
use tracing::{debug, warn};

impl ContentFilter {
    pub fn should_translate(&self, text: &str, node_type: &NodeType) -> bool {
        debug!(
            text_len = text.len(),
            node_type = ?node_type,
            "Checking if text should be translated"
        );

        // 检查长度
        if text.len() < self.config.min_length {
            debug!(
                text_len = text.len(),
                min_length = self.config.min_length,
                "Text filtered: too short"
            );
            return false;
        }

        if text.len() > self.config.max_length {
            warn!(
                text_len = text.len(),
                max_length = self.config.max_length,
                "Text filtered: too long"
            );
            return false;
        }

        // 检查排除关键词
        for keyword in &self.config.exclude_keywords {
            if text.contains(keyword) {
                debug!(
                    keyword = %keyword,
                    "Text filtered: excluded keyword"
                );
                return false;
            }
        }

        // 检查只包含英文/中文
        if !self.config.allow_english && self.is_english_only(text) {
            debug!("Text filtered: English only");
            return false;
        }

        if !self.config.allow_chinese && self.is_chinese_only(text) {
            debug!("Text filtered: Chinese only");
            return false;
        }

        debug!("Text passed all filters");
        true
    }
}
```

#### parser/strategy.rs

**当前状态：** 有 `debug` 导入，但无实际日志调用

**需要补充的日志：**

```rust
use tracing::{debug};

impl ExtractionStrategy {
    pub fn should_extract(&self, node_type: &NodeType, context: &ExtractionContext) -> bool {
        debug!(
            node_type = ?node_type,
            enabled = self.extract_comments || self.extract_docstrings || self.extract_strings,
            "Checking extraction strategy"
        );

        match node_type {
            NodeType::Comment => self.extract_comments,
            NodeType::DocString => self.extract_docstrings,
            NodeType::StringLiteral => self.extract_strings,
            _ => false,
        }
    }
}
```

---

## 三、低优先级模块（P2 - 可选补充）

### 3.1 查询构建器

**文件：** `parser/queries/{builder,comment_queries,function_queries,string_queries}.rs`

**需要补充的日志：**

```rust
use tracing::{debug};

// 在构建查询时
debug!(
    language = %language,
    patterns_count = patterns.len(),
    "Building tree-sitter query"
);

// 在查询执行时
debug!(
    query_name = %query_name,
    "Executing query"
);
```

---

### 3.2 正则表达式解析器

**文件：** `parser/regex_parsers/{fallback,html,shell,sql}.rs`

**需要补充的日志：**

```rust
use tracing::{debug, warn};

// 回退解析器
warn!(
    file = %file.path.display(),
    reason = "no_tree_sitter_parser",
    "Using fallback regex parser"
);

// 特殊格式解析器
debug!(
    format = %format,
    file = %file.path.display(),
    "Parsing with regex parser"
);
```

---

### 3.3 工具模块

**文件：** `utils/hash.rs`

**当前状态：** 有 `trace` 导入，但无实际日志调用

**需要补充的日志：**

```rust
use tracing::trace;

pub fn calculate_hash(content: &[u8]) -> String {
    trace!(
        content_len = content.len(),
        "Calculating hash"
    );
    // ... 计算逻辑
}
```

---

## 四、实施计划

### 阶段一：核心模块（1-2周）

**目标：** 补充高优先级模块的日志

1. **工作流模块**（2-3天）
   - workflow/executor.rs
   - workflow/file_processor.rs

2. **缓存模块**（2-3天）
   - cache/mod.rs
   - cache/trait.rs
   - cache/binary.rs

3. **写入模块**（2-3天）
   - writer/concurrent.rs
   - writer/file.rs（补充完善）

4. **工厂模块**（1天）
   - factory/mod.rs

**验证方式：**
```bash
# 运行测试
cargo test --lib

# 运行集成测试
cargo test --test workflow_integration_tests
cargo test --test cache_integration_tests
cargo test --test writer_integration_tests

# 验证日志输出
RUST_LOG=debug cargo run -- --config test.toml translate .
```

---

### 阶段二：解析器模块（2-3周）

**目标：** 补充中优先级模块的日志

1. **解析器协调器**（2-3天）
   - parser/coordinator/coordinator.rs

2. **核心解析模块**（3-4天）
   - parser/core/language_parser.rs
   - parser/core/query_executor.rs
   - parser/core/extractor.rs
   - parser/core/string_processor.rs
   - parser/core/position_tracker.rs

3. **语言解析器**（5-7天）
   - parser/languages/rust/parser.rs
   - parser/languages/python/parser.rs
   - parser/languages/go/parser.rs
   - parser/languages/java/parser.rs
   - parser/languages/javascript/parser.rs
   - parser/languages/typescript/parser.rs
   - parser/languages/c/parser.rs
   - parser/languages/cpp/parser.rs
   - parser/languages/csharp/parser.rs

4. **过滤和策略**（1-2天）
   - parser/filter.rs
   - parser/strategy.rs

**验证方式：**
```bash
# 运行解析器测试
cargo test --test parser_integration_tests

# 验证不同语言的解析日志
RUST_LOG=debug cargo run -- --config test.toml translate rust/
RUST_LOG=debug cargo run -- --config test.toml translate python/
```

---

### 阶段三：低优先级模块（可选，1-2周）

**目标：** 补充低优先级模块的日志

1. **查询构建器**（1-2天）
   - parser/queries/*.rs

2. **正则表达式解析器**（1天）
   - parser/regex_parsers/*.rs

3. **工具模块**（0.5天）
   - utils/hash.rs

---

## 五、日志级别规范

### Trace 级别
- 详细的执行流程
- 每个步骤的详细信息
- 仅用于深度调试

**示例：**
```rust
trace!(
    content_len = content.len(),
    "Calculating hash"
);
```

---

### Debug 级别
- 组件初始化
- 关键操作的开始和结束
- 中间结果和状态
- 用于日常开发调试

**示例：**
```rust
debug!(
    file = %file_path.display(),
    total_units = units.len(),
    "File parsed successfully"
);

debug!(
    cache_hits = self.cache_hit_count,
    cache_misses = self.cache_miss_count,
    "Cache statistics"
);
```

---

### Info 级别
- 重要操作的开始和完成
- 文件处理进度
- 统计信息汇总
- 用于了解系统运行状态

**示例：**
```rust
info!(
    files_count = files.len(),
    "Starting file translation"
);

info!(
    total_files = stats.total_files,
    processed_files = stats.processed_files,
    duration_ms = stats.total_duration_ms,
    "Translation workflow completed"
);
```

---

### Warn 级别
- 可恢复的错误
- 降级处理
- 配置警告
- 需要注意但不影响运行的问题

**示例：**
```rust
warn!(
    file = %file_path.display(),
    error = %e,
    "Failed to process file, continuing"
);

warn!(
    encoding = %encoding,
    confidence = result.confidence,
    "Low confidence encoding detection"
);
```

---

### Error 级别
- 严重的错误
- 无法继续的操作
- 需要用户干预的问题

**示例：**
```rust
error!(
    file = %file_path.display(),
    error = %e,
    "Failed to write file"
);

error!(
    provider = %provider,
    error = %e,
    "Translation API call failed"
);
```

---

## 六、结构化日志字段规范

### 常用字段

| 字段名 | 用途 | 示例 |
|--------|------|------|
| `file` | 文件路径 | `file = %path.display()` |
| `file_size` | 文件大小 | `file_size = content.len()` |
| `encoding` | 编码类型 | `encoding = %encoding` |
| `language` | 编程语言 | `language = %self.language()` |
| `units_count` | 翻译单元数量 | `units_count = units.len()` |
| `duration_ms` | 耗时（毫秒） | `duration_ms = elapsed.as_millis()` |
| `error` | 错误信息 | `error = %e` |
| `cache_hits` | 缓存命中数 | `cache_hits = stats.cache_hit_count` |
| `api_calls` | API 调用次数 | `api_calls = stats.api_call_count` |

---

### 日志模板

**文件处理：**
```rust
info!(
    file = %file_path.display(),
    file_size = content.len(),
    "Processing file"
);
```

**操作完成：**
```rust
info!(
    file = %file_path.display(),
    units_count = units.len(),
    duration_ms = elapsed.as_millis(),
    "Operation completed"
);
```

**错误处理：**
```rust
error!(
    file = %file_path.display(),
    error = %e,
    "Operation failed"
);
```

---

## 七、性能考虑

### 避免过度日志

1. **使用条件日志**：在性能敏感的代码路径中，使用条件日志记录
   ```rust
   if log::log_enabled!(log::Level::Debug) {
       debug!(details = expensive_operation(), "Debug info");
   }
   ```

2. **字符串格式化**：使用 `tracing` 的结构化字段，避免不必要的字符串格式化
   ```rust
   // 好
   debug!(file = %path.display(), "Processing file");

   // 避免
   debug!(format!("Processing file: {}", path.display()));
   ```

3. **延迟计算**：使用 `tracing` 的延迟计算特性
   ```rust
   use tracing::field::display;

   debug!(file = display(|| path.display()), "Processing file");
   ```

---

### 异步日志

项目已使用 `tracing_appender` 实现非阻塞日志写入，确保日志不会阻塞主流程：

```rust
// logger/mod.rs 中已实现
let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
let _ = LOG_GUARD.set(Box::new(guard));
```

---

## 八、测试策略

### 单元测试

每个模块的日志功能应包含单元测试：

```rust
#[test]
fn test_logging() {
    // 验证日志导入
    assert!(module_has_logging_import());

    // 验证日志级别使用正确
    assert!(log_levels_are_appropriate());
}
```

---

### 集成测试

运行完整的翻译流程，验证日志输出：

```bash
# 运行所有测试
cargo test --all

# 运行特定集成测试
cargo test --test workflow_integration_tests
cargo test --test cache_integration_tests
cargo test --test logger_integration
```

---

### 日志审查

检查日志输出是否符合预期：

1. **日志级别正确**：错误使用 error，警告使用 warn，信息使用 info
2. **日志信息完整**：包含必要的上下文信息
3. **性能影响可接受**：日志不会显著影响翻译性能

---

## 九、预期成果

### 覆盖率目标

| 阶段 | 模块数量 | 覆盖率 |
|------|---------|--------|
| 当前 | 9/50 | 18% |
| 阶段一 | 17/50 | 34% |
| 阶段二 | 38/50 | 76% |
| 阶段三 | 42/50 | 84% |

### 功能改进

1. **可观测性提升**：用户可以通过日志了解系统的运行状态
2. **调试效率提高**：开发人员可以快速定位问题
3. **性能分析**：通过日志可以分析性能瓶颈
4. **问题追踪**：错误日志可以帮助追踪和修复问题

---

## 十、参考文档

- [tracing 文档](https://docs.rs/tracing/)
- [tracing-subscriber 文档](https://docs.rs/tracing-subscriber/)
- 项目日志需求：`docs/spec/logging-requirements.md`
- 日志集成方案：`docs/logging-integration-plan.md`
- 项目规则：`AGENTS.md`

---

## 版本历史

| 版本 | 日期 | 作者 | 说明 |
|------|------|------|------|
| 1.0 | 2026-03-20 | AI Assistant | 初始版本，基于实际代码分析制定补充计划 |