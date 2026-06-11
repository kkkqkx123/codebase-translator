# Reporter Module Design

## 概述

Reporter 模块提供进度跟踪和统计报告功能，支持多种报告格式（文本、JSON），实时显示翻译进度，收集详细的翻译统计信息。

## 设计目的

1. **进度跟踪**：实时显示翻译进度，提供用户反馈
2. **统计收集**：收集详细的翻译统计信息（文件数、单元数、API 调用等）
3. **多格式报告**：支持文本和 JSON 格式输出
4. **错误记录**：记录错误和跳过的文件，便于问题排查

## 核心组件

### 1. Reporter Trait

**位置**：`src/reporter/trait.rs`

**职责**：
- 定义报告器接口
- 提供统一的方法签名

**关键方法**：
```rust
pub trait Reporter: Send + Sync {
    // 进度跟踪
    fn report_total_files(&self, count: usize);
    fn report_file(&self, path: &Path, units: usize);
    fn report_progress(&self, current: usize, total: usize);

    // 事件记录
    fn report_error(&self, path: &Path, error: &TranslateError);
    fn report_skipped(&self, path: &Path);
    fn report_api_call(&self, count: usize);
    fn report_cache_hit(&self);
    fn report_cache_miss(&self);

    // 翻译器统计
    fn report_translator_call(
        &self,
        translator_type: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    );

    fn report_llm_provider_call(
        &self,
        provider_id: &str,
        provider_name: &str,
        model: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    );

    // 报告生成
    fn final_report(
        &self,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<String, TranslateError>;

    fn save_report(
        &self,
        path: &Path,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<(), TranslateError>;

    // 查询方法
    fn get_stats(&self) -> Option<TranslationStats>;
    fn has_errors(&self) -> bool;
    fn get_progress(&self) -> f64;

    // 完成
    fn finalize(&self, stats: &TranslationStats);
}
```

### 2. TranslationStats

**位置**：`src/reporter/stats.rs`

**职责**：
- 收集翻译统计信息
- 线程安全的计数器

**关键字段**：
```rust
pub struct TranslationStats {
    // 文件统计
    pub total_files: AtomicUsize,
    pub processed_files: AtomicUsize,
    pub skipped_files: AtomicUsize,
    pub failed_files: AtomicUsize,

    // 翻译单元统计
    pub total_units: AtomicUsize,
    pub translated_units: AtomicUsize,
    pub cached_units: AtomicUsize,
    pub failed_units: AtomicUsize,

    // API 调用统计
    pub total_api_calls: AtomicUsize,
    pub total_chars: AtomicUsize,
    pub total_latency_ms: AtomicU64,

    // 缓存统计
    pub cache_hits: AtomicUsize,
    pub cache_misses: AtomicUsize,

    // 翻译器统计
    pub translator_stats: Arc<RwLock<HashMap<String, TranslatorStats>>>,

    // LLM 提供商统计
    pub llm_provider_stats: Arc<RwLock<HashMap<String, LLMProviderStats>>>,

    // 错误记录
    pub errors: Arc<Mutex<Vec<ErrorRecord>>>,
}
```

**线程安全**：
```rust
pub struct SharedStats {
    stats: Arc<TranslationStats>,
}

impl SharedStats {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(TranslationStats::new()),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            stats: Arc::clone(&self.stats),
        }
    }
}
```

### 3. DefaultReporter

**位置**：`src/reporter/default.rs`

**职责**：
- 默认报告器实现
- 实时进度显示
- 统计信息收集

**关键功能**：
```rust
pub struct DefaultReporter {
    stats: Arc<TranslationStats>,
    progress_tracker: Arc<ProgressTracker>,
    event_logger: Arc<EventLogger>,
}
```

**实现特点**：
- 实时进度更新
- 彩色输出（可选）
- 错误高亮显示
- 详细的统计报告

### 4. ProgressTracker

**位置**：`src/reporter/progress.rs`

**职责**：
- 跟踪翻译进度
- 计算进度百分比
- 提供进度查询

**关键功能**：
```rust
pub struct ProgressTracker {
    total_files: AtomicUsize,
    processed_files: AtomicUsize,
}

impl ProgressTracker {
    pub fn update(&self, processed: usize) {
        self.processed_files.fetch_add(processed, Ordering::Relaxed);
    }

    pub fn get_progress(&self) -> f64 {
        let total = self.total_files.load(Ordering::Relaxed);
        let processed = self.processed_files.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            (processed as f64 / total as f64) * 100.0
        }
    }
}
```

### 5. EventLogger

**位置**：`src/reporter/logger.rs`

**职责**：
- 记录事件日志
- 分类记录不同类型的事件

**关键功能**：
```rust
pub struct EventLogger {
    events: Arc<Mutex<Vec<LogEvent>>>,
}

pub enum LogEvent {
    FileProcessed { path: PathBuf, units: usize },
    FileSkipped { path: PathBuf, reason: String },
    Error { path: PathBuf, error: String },
    ApiCall { count: usize, chars: usize },
    CacheHit,
    CacheMiss,
    TranslatorCall {
        translator_type: String,
        latency_ms: u64,
        success: bool,
        chars: usize,
    },
}
```

### 6. ReportGenerator

**位置**：`src/reporter/generator.rs`

**职责**：
- 生成格式化的报告
- 支持多种输出格式

**支持的格式**：
```rust
pub enum ReportFormat {
    Text,   // 人类可读的文本格式
    Json,   // 机器可读的 JSON 格式
}
```

**文本报告示例**：
```
Translation Summary
===================

Files:
  Total: 100
  Processed: 95
  Skipped: 3
  Failed: 2

Translation Units:
  Total: 5000
  Translated: 4500
  Cached: 400
  Failed: 100

API Calls:
  Total: 100
  Characters: 250000
  Average Latency: 245ms

Cache:
  Hits: 400
  Misses: 4600
  Hit Rate: 8.00%

Translators:
  deeplx: 60 calls, avg 200ms, 100% success
  llm: 40 calls, avg 300ms, 100% success

Errors:
  - src/file1.rs: Translation timeout
  - src/file2.rs: API rate limit exceeded
```

**JSON 报告示例**：
```json
{
  "files": {
    "total": 100,
    "processed": 95,
    "skipped": 3,
    "failed": 2
  },
  "units": {
    "total": 5000,
    "translated": 4500,
    "cached": 400,
    "failed": 100
  },
  "api_calls": {
    "total": 100,
    "characters": 250000,
    "average_latency_ms": 245
  },
  "cache": {
    "hits": 400,
    "misses": 4600,
    "hit_rate": 0.08
  },
  "translators": {
    "deeplx": {
      "calls": 60,
      "avg_latency_ms": 200,
      "success_rate": 1.0
    },
    "llm": {
      "calls": 40,
      "avg_latency_ms": 300,
      "success_rate": 1.0
    }
  },
  "errors": [
    {
      "path": "src/file1.rs",
      "error": "Translation timeout"
    },
    {
      "path": "src/file2.rs",
      "error": "API rate limit exceeded"
    }
  ]
}
```

## 技术选型

### 并发原语
- **AtomicUsize**：无锁计数器
  - 零开销
  - 线程安全
  - 适合简单计数

- **RwLock**：读写锁
  - 允许多个并发读取
  - 写操作独占
  - 适合读多写少

- **Mutex**：互斥锁
  - 简单易用
  - 适合复杂操作

### 序列化
- **Serde**：JSON 序列化
  - 类型安全
  - 零成本抽象
  - 广泛支持

## 关键设计要点

### 1. 统计收集

```rust
impl TranslationStats {
    pub fn record_file(&self, units: usize) {
        self.processed_files.fetch_add(1, Ordering::Relaxed);
        self.total_units.fetch_add(units, Ordering::Relaxed);
    }

    pub fn record_translation(&self, chars: usize, cached: bool) {
        if cached {
            self.cached_units.fetch_add(1, Ordering::Relaxed);
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.translated_units.fetch_add(1, Ordering::Relaxed);
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_api_call(&self, latency_ms: u64, chars: usize) {
        self.total_api_calls.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
        self.total_chars.fetch_add(chars, Ordering::Relaxed);
    }
}
```

### 2. 翻译器统计

```rust
pub struct TranslatorStats {
    pub calls: AtomicUsize,
    pub success_calls: AtomicUsize,
    pub failed_calls: AtomicUsize,
    pub total_latency_ms: AtomicU64,
    pub total_chars: AtomicUsize,
}

impl TranslationStats {
    pub fn record_translator_call(
        &self,
        translator_type: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        let mut stats = self.translator_stats.write().unwrap();
        let translator_stats = stats.entry(translator_type.to_string())
            .or_insert_with(TranslatorStats::new);

        translator_stats.calls.fetch_add(1, Ordering::Relaxed);
        translator_stats.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
        translator_stats.total_chars.fetch_add(chars, Ordering::Relaxed);

        if success {
            translator_stats.success_calls.fetch_add(1, Ordering::Relaxed);
        } else {
            translator_stats.failed_calls.fetch_add(1, Ordering::Relaxed);
        }
    }
}
```

### 3. 错误记录

```rust
pub struct ErrorRecord {
    pub path: PathBuf,
    pub error: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl TranslationStats {
    pub fn record_error(&self, path: &Path, error: &TranslateError) {
        let record = ErrorRecord {
            path: path.to_path_buf(),
            error: error.to_string(),
            timestamp: chrono::Utc::now(),
        };

        let mut errors = self.errors.lock().unwrap();
        errors.push(record);

        self.failed_files.fetch_add(1, Ordering::Relaxed);
    }
}
```

### 4. 进度计算

```rust
impl ProgressTracker {
    pub fn get_progress(&self) -> f64 {
        let total = self.total_files.load(Ordering::Relaxed);
        let processed = self.processed_files.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            (processed as f64 / total as f64) * 100.0
        }
    }
}
```

### 5. 报告生成

```rust
impl ReportGenerator {
    pub fn generate_text_report(&self, stats: &TranslationStats) -> String {
        let mut report = String::new();

        report.push_str("Translation Summary\n");
        report.push_str("===================\n\n");

        // 文件统计
        report.push_str("Files:\n");
        report.push_str(&format!("  Total: {}\n", stats.total_files.load(Ordering::Relaxed)));
        report.push_str(&format!("  Processed: {}\n", stats.processed_files.load(Ordering::Relaxed)));

        // 翻译单元统计
        report.push_str("\nTranslation Units:\n");
        report.push_str(&format!("  Total: {}\n", stats.total_units.load(Ordering::Relaxed)));

        // API 调用统计
        report.push_str("\nAPI Calls:\n");
        let total_calls = stats.total_api_calls.load(Ordering::Relaxed);
        let total_latency = stats.total_latency_ms.load(Ordering::Relaxed);
        let avg_latency = if total_calls > 0 {
            total_latency as f64 / total_calls as f64
        } else {
            0.0
        };
        report.push_str(&format!("  Total: {}\n", total_calls));
        report.push_str(&format!("  Average Latency: {:.2}ms\n", avg_latency));

        // 缓存统计
        let hits = stats.cache_hits.load(Ordering::Relaxed);
        let misses = stats.cache_misses.load(Ordering::Relaxed);
        let hit_rate = if hits + misses > 0 {
            (hits as f64 / (hits + misses) as f64) * 100.0
        } else {
            0.0
        };
        report.push_str(&format!("  Hit Rate: {:.2}%\n", hit_rate));

        report
    }

    pub fn generate_json_report(&self, stats: &TranslationStats) -> Result<String> {
        let summary = TranslationSummary::from_stats(stats);
        serde_json::to_string_pretty(&summary)
            .map_err(|e| TranslateError::Report(format!("JSON serialization failed: {}", e)))
    }
}
```

## 使用示例

### 创建报告器

```rust
use codebase_translate::reporter::{DefaultReporter, SharedStats};

let stats = SharedStats::new();
let reporter = DefaultReporter::new(stats.clone());
```

### 记录事件

```rust
// 文件处理
reporter.report_file(&path, units.len());
reporter.report_progress(current, total);

// 翻译调用
reporter.report_translator_call(
    "deeplx",
    250,  // latency_ms
    true, // success
    1000, // chars
);

// 错误
reporter.report_error(&path, &error);

// 缓存
reporter.report_cache_hit();
reporter.report_cache_miss();
```

### 生成报告

```rust
// 生成文本报告
let text_report = reporter.final_report(&stats.get()?, ReportFormat::Text)?;
println!("{}", text_report);

// 生成 JSON 报告
let json_report = reporter.final_report(&stats.get()?, ReportFormat::Json)?;

// 保存报告
reporter.save_report(Path::new("report.txt"), &stats.get()?, ReportFormat::Text)?;
reporter.save_report(Path::new("report.json"), &stats.get()?, ReportFormat::Json)?;
```

## 性能考量

1. **无锁计数**：
   - 使用原子类型
   - 避免锁竞争
   - 高并发性能

2. **读写分离**：
   - 统计收集：原子操作
   - 报告生成：读锁
   - 最小化锁争用

3. **延迟计算**：
   - 进度百分比：按需计算
   - 平均值：按需计算
   - 避免不必要的计算

4. **内存效率**：
   - 错误记录：限制大小
   - 事件日志：可选记录
   - 按需分配

## 扩展性

1. **新的报告格式**：
   - Markdown
   - HTML
   - CSV

2. **新的统计指标**：
   - 内存使用
   - CPU 使用
   - 网络流量

3. **高级报告**：
   - 时间线报告
   - 热力图
   - 趋势分析

4. **集成第三方**：
   - Grafana
   - Prometheus
   - ELK Stack