# Workflow Module Design

## 概述

Workflow 模块提供翻译工作流的编排和执行，协调扫描、解析、翻译、写入等各个模块，实现端到端的翻译流程。

## 设计目的

1. **流程编排**：协调各个模块，实现完整的翻译流程
2. **错误处理**：统一的错误处理和恢复机制
3. **进度跟踪**：实时跟踪翻译进度，提供用户反馈
4. **并发控制**：优化并发策略，提高翻译效率

## 核心组件

### 1. TranslationWorkflow

**位置**：`src/workflow/executor.rs`

**职责**：
- 翻译工作流执行器
- 协调各个处理阶段
- 处理错误和重试

**关键功能**：
```rust
pub struct TranslationWorkflow {
    config: WorkflowConfig,
    scanner: Arc<dyn Scanner>,
    parser: Arc<ParserCoordinator>,
    cache: Option<Arc<BinaryCache>>,
    translator: Arc<TranslationService>,
    writer: Arc<FileWriter>,
    reporter: Arc<dyn Reporter>,
}

impl TranslationWorkflow {
    pub async fn execute(&self) -> Result<WorkflowResult> {
        // 阶段 1: 扫描文件
        let entries = self.scan_files().await?;

        // 阶段 2: 处理每个文件
        let mut results = Vec::new();
        for entry in entries {
            match self.process_file(&entry).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    self.reporter.report_error(&entry.path, &e);
                    // 继续处理下一个文件
                }
            }
        }

        // 阶段 3: 生成报告
        let stats = self.reporter.get_stats().ok_or("Stats not available")?;
        let report = self.reporter.final_report(&stats, ReportFormat::Text)?;

        Ok(WorkflowResult {
            success: !self.reporter.has_errors(),
            report,
            stats,
        })
    }

    async fn process_file(&self, entry: &FileEntry) -> Result<FileProcessResult> {
        // 1. 读取文件
        let content = std::fs::read_to_string(&entry.path)?;

        // 2. 检查缓存
        let file_hash = calculate_hash(&content);
        let config_hash = self.config_hash();
        if let Some(cache) = &self.cache {
            if let Some(cache_entry) = cache.get(&file_hash, &config_hash)? {
                self.reporter.report_cache_hit();
                return Ok(FileProcessResult::Cached);
            }
            self.reporter.report_cache_miss();
        }

        // 3. 检测编码并转换为 UTF-8
        let encoding = self.detector.detect_file(&entry.path)?;
        let utf8_content = if encoding.encoding != EncodingType::UTF8 {
            self.encoder.convert_file_to_utf8(&entry.path, &encoding.encoding.to_string())?;
            std::fs::read_to_string(&entry.path)?
        } else {
            content
        };

        // 4. 解析文件，提取翻译单元
        let file = File::new(entry.path.clone(), utf8_content, "utf-8");
        let units = self.parser.parse_file(&file).await?;

        if units.is_empty() {
            return Ok(FileProcessResult::NoContent);
        }

        // 5. 过滤语言
        let units: Vec<_> = units.into_iter()
            .filter(|u| self.language_filter.should_include(&u.content))
            .collect();

        if units.is_empty() {
            return Ok(FileProcessResult::NoMatchingLanguage);
        }

        // 6. 翻译单元
        let translated_units = self.translator
            .translate_units(&units, &self.config.source_lang, &self.config.target_lang)
            .await?;

        // 7. 写入文件
        self.writer.write(&entry.path, &translated_units).await?;

        // 8. 更新缓存
        if let Some(cache) = &self.cache {
            cache.set(&CacheEntry {
                file_hash,
                config_hash,
                timestamp: chrono::Utc::now().timestamp(),
                units: translated_units,
            })?;
        }

        Ok(FileProcessResult::Success(translated_units.len()))
    }
}
```

### 2. WorkflowBuilder

**位置**：`src/workflow/builder.rs`

**职责**：
- 工作流构建器
- 组装各个组件
- 验证配置

**关键功能**：
```rust
pub struct WorkflowBuilder {
    config: Option<WorkflowConfig>,
    scanner: Option<Arc<dyn Scanner>>,
    parser: Option<Arc<ParserCoordinator>>,
    cache: Option<Arc<BinaryCache>>,
    translator: Option<Arc<TranslationService>>,
    writer: Option<Arc<FileWriter>>,
    reporter: Option<Arc<dyn Reporter>>,
}

impl WorkflowBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            scanner: None,
            parser: None,
            cache: None,
            translator: None,
            writer: None,
            reporter: None,
        }
    }

    pub fn with_config(mut self, config: WorkflowConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_scanner(mut self, scanner: Arc<dyn Scanner>) -> Self {
        self.scanner = Some(scanner);
        self
    }

    pub fn with_parser(mut self, parser: Arc<ParserCoordinator>) -> Self {
        self.parser = Some(parser);
        self
    }

    pub fn with_cache(mut self, cache: Option<Arc<BinaryCache>>) -> Self {
        self.cache = cache;
        self
    }

    pub fn with_translator(mut self, translator: Arc<TranslationService>) -> Self {
        self.translator = Some(translator);
        self
    }

    pub fn with_writer(mut self, writer: Arc<FileWriter>) -> Self {
        self.writer = Some(writer);
        self
    }

    pub fn with_reporter(mut self, reporter: Arc<dyn Reporter>) -> Self {
        self.reporter = Some(reporter);
        self
    }

    pub fn build(self) -> Result<TranslationWorkflow> {
        // 验证必要组件
        let config = self.config.ok_or("Config is required")?;
        let scanner = self.scanner.unwrap_or_else(|| Arc::new(FSScanner::new()));
        let parser = self.parser.ok_or("Parser is required")?;
        let translator = self.translator.ok_or("Translator is required")?;
        let writer = self.writer.ok_or("Writer is required")?;
        let reporter = self.reporter.unwrap_or_else(|| {
            Arc::new(DefaultReporter::new(SharedStats::new()))
        });

        Ok(TranslationWorkflow {
            config,
            scanner,
            parser,
            cache: self.cache,
            translator,
            writer,
            reporter,
        })
    }
}
```

### 3. FileProcessor

**位置**：`src/workflow/file_processor.rs`

**职责**：
- 单文件处理器
- 封装文件处理逻辑
- 提供重试机制

**关键功能**：
```rust
pub struct FileProcessor {
    parser: Arc<ParserCoordinator>,
    translator: Arc<TranslationService>,
    writer: Arc<FileWriter>,
    max_retries: usize,
}

impl FileProcessor {
    pub async fn process(&self, file: &File) -> Result<FileProcessResult> {
        let mut attempts = 0;
        let max_attempts = self.max_retries + 1;

        loop {
            attempts += 1;

            match self.try_process(file).await {
                Ok(result) => return Ok(result),
                Err(e) if attempts >= max_attempts => return Err(e),
                Err(e) => {
                    tracing::warn!(
                        file = %file.path.display(),
                        attempt = attempts,
                        error = %e,
                        "File processing failed, retrying"
                    );
                    // 延迟后重试
                    tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                }
            }
        }
    }

    async fn try_process(&self, file: &File) -> Result<FileProcessResult> {
        // 1. 解析文件
        let units = self.parser.parse_file(file).await?;

        // 2. 翻译单元
        let translated_units = self.translator
            .translate_units(&units, "zh", "en")
            .await?;

        // 3. 写入文件
        self.writer.write(&file.path, &translated_units).await?;

        Ok(FileProcessResult::Success(translated_units.len()))
    }
}
```

### 4. WorkflowConfig

**位置**：`src/workflow/executor.rs`

**职责**：
- 工作流配置
- 定义扫描规则和翻译参数

**关键字段**：
```rust
pub struct WorkflowConfig {
    pub root_path: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub follow_symlinks: bool,
    pub respect_gitignore: bool,
    pub gitignore_patterns: Vec<String>,
    pub source_lang: String,
    pub target_lang: String,
    pub concurrency: usize,
    pub max_retries: usize,
}
```

### 5. WorkflowResult

**职责**：
- 工作流执行结果
- 包含统计和报告

**关键字段**：
```rust
pub struct WorkflowResult {
    pub success: bool,
    pub report: String,
    pub stats: TranslationStats,
}
```

### 6. FileProcessResult

**职责**：
- 文件处理结果
- 表示不同的处理状态

**变体**：
```rust
pub enum FileProcessResult {
    Success(usize),           // 成功，翻译了 N 个单元
    Cached,                  // 使用缓存
    NoContent,               // 无可翻译内容
    NoMatchingLanguage,      // 无匹配语言
    Skipped(String),         // 跳过，原因
    Failed(String),          // 失败，错误信息
}
```

## 技术选型

### 异步运行时
- **Tokio**：异步运行时
  - 高性能异步 I/O
  - 调度器
  - 定时器
  - 丰富的生态系统

### 并发控制
- **Tokio Semaphore**：并发限制
- **Arc<RwLock<T>>**：共享状态
- **AtomicUsize**：原子计数器

### 错误处理
- **thiserror**：错误派生宏
- **anyhow**：错误上下文
- **Result<T, E>**：显式错误处理

## 关键设计要点

### 1. 工作流阶段

```
扫描文件 → 检查缓存 → 编码检测 → 解析文件 → 过滤语言
    ↓
翻译单元 → 写入文件 → 更新缓存 → 统计报告
```

### 2. 并发策略

```rust
pub async fn execute_concurrent(&self) -> Result<WorkflowResult> {
    // 扫描文件
    let entries = self.scan_files().await?;

    // 创建信号量
    let semaphore = Arc::new(Semaphore::new(self.config.concurrency));

    // 并发处理文件
    let tasks: Vec<_> = entries.into_iter()
        .map(|entry| {
            let semaphore = Arc::clone(&semaphore);
            let processor = self.clone_processor();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await?;
                processor.process_entry(&entry).await
            })
        })
        .collect();

    // 等待所有任务完成
    let results: Result<Vec<_>, _> = try_join_all(tasks).await?;

    Ok(self.aggregate_results(results))
}
```

### 3. 错误恢复

```rust
async fn process_file_with_recovery(&self, entry: &FileEntry) -> Result<FileProcessResult> {
    match self.process_file(entry).await {
        Ok(result) => Ok(result),
        Err(e) => {
            // 记录错误
            self.reporter.report_error(&entry.path, &e);

            // 尝试恢复
            if self.is_recoverable(&e) {
                self.recover_file(entry, &e).await?;
            }

            Err(e)
        }
    }
}
```

### 4. 进度跟踪

```rust
async fn process_with_progress(&self, entry: &FileEntry) -> Result<FileProcessResult> {
    // 报告开始
    self.reporter.report_file(&entry.path, 0);

    // 处理文件
    let result = self.process_file(entry).await?;

    // 报告完成
    if let FileProcessResult::Success(units) = result {
        self.reporter.report_file(&entry.path, units);
    }

    // 更新进度
    self.reporter.report_progress(
        self.reporter.get_stats().unwrap().processed_files.load(Ordering::Relaxed),
        self.total_files,
    );

    Ok(result)
}
```

### 5. 资源清理

```rust
impl Drop for TranslationWorkflow {
    fn drop(&mut self) {
        // 清理缓存
        if let Some(cache) = &self.cache {
            let _ = cache.close();
        }

        // 关闭翻译器
        tokio::spawn(async move {
            let _ = self.translator.close().await;
        });
    }
}
```

### 6. 配置验证

```rust
impl WorkflowConfig {
    pub fn validate(&self) -> Result<()> {
        if self.root_path.is_empty() {
            return Err("Root path is required".into());
        }

        if self.source_lang.is_empty() || self.target_lang.is_empty() {
            return Err("Source and target languages are required".into());
        }

        if self.concurrency == 0 {
            return Err("Concurrency must be greater than 0".into());
        }

        Ok(())
    }
}
```

## 使用示例

### 构建工作流

```rust
use codebase_translate::workflow::{WorkflowBuilder, WorkflowConfig};

let config = WorkflowConfig {
    root_path: "/workspace".to_string(),
    include_patterns: vec!["**/*.rs".to_string()],
    exclude_patterns: vec!["**/target/**".to_string()],
    follow_symlinks: false,
    respect_gitignore: true,
    gitignore_patterns: vec![],
    source_lang: "zh".to_string(),
    target_lang: "en".to_string(),
    concurrency: 4,
    max_retries: 3,
};

let workflow = WorkflowBuilder::new()
    .with_config(config)
    .with_scanner(Arc::new(FSScanner::new()))
    .with_parser(Arc::new(parser))
    .with_cache(Some(Arc::new(cache)))
    .with_translator(Arc::new(translator))
    .with_writer(Arc::new(writer))
    .with_reporter(Arc::new(reporter))
    .build()?;
```

### 执行工作流

```rust
let result = workflow.execute().await?;

if result.success {
    println!("{}", result.report);
} else {
    eprintln!("Translation failed");
    println!("{}", result.report);
}
```

### 并发执行

```rust
let result = workflow.execute_concurrent().await?;
```

## 性能考量

1. **并发优化**：
   - 信号量控制并发数
   - 避免资源耗尽
   - 合理的默认值

2. **I/O 优化**：
   - 异步 I/O
   - 连接池
   - 批量操作

3. **内存优化**：
   - 流式处理
   - 及时释放
   - 避免克隆

4. **错误处理**：
   - 快速失败
   - 部分成功
   - 详细的错误信息

## 扩展性

1. **新的工作流阶段**：
   - 预处理（代码格式化）
   - 后处理（代码修复）
   - 验证（编译检查）

2. **高级并发**：
   - 动态并发调整
   - 基于负载的调度
   - 优先级队列

3. **插件系统**：
   - 自定义处理器
   - 中间件
   - 钩子函数

4. **分布式执行**：
   - 多机器协作
   - 任务分发
   - 结果聚合