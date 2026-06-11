# Logger Module Design

## 概述

Logger 模块提供结构化日志功能，基于 tracing 框架实现，支持多种输出目标、格式和日志级别，为翻译系统提供完整的可观测性支持。

## 设计目的

1. **结构化日志**：提供结构化的日志输出，便于解析和分析
2. **灵活配置**：支持多种输出目标（stdout、stderr、文件）和格式
3. **性能优化**：使用异步日志确保不影响主流程性能
4. **可观测性**：提供详细的执行过程和错误信息

## 核心组件

### 1. 日志初始化

**位置**：`src/logger/mod.rs`

**职责**：
- 初始化全局日志系统
- 配置日志级别和格式
- 设置输出目标和过滤器

**关键功能**：
```rust
pub fn init(config: &LoggingConfig, project_dir: Option<&Path>) -> Result<()> {
    // 初始化日志系统
    // 只能调用一次
}
```

**初始化流程**：
1. 验证配置有效性
2. 解析日志级别
3. 创建环境过滤器
4. 选择输出目标（stdout/stderr/file）
5. 选择日志格式（pretty/compact/json）
6. 注册全局订阅者

### 2. 日志配置

**LoggingConfig**：
```rust
pub struct LoggingConfig {
    pub level: String,      // 日志级别: trace/debug/info/warn/error
    pub output: String,     // 输出目标: stdout/stderr/file
    pub format: String,     // 格式: pretty/compact/json
    pub file: Option<String>, // 文件路径（output=file时必填）
    pub span_events: bool,  // 是否记录 span 事件
}
```

### 3. 配置解析

**位置**：`src/logger/config.rs`

**关键功能**：
```rust
pub fn parse_level(level: &str) -> Level {
    match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,  // 默认 INFO
    }
}

pub fn validate_config(config: &LoggingConfig) -> Result<()> {
    // 验证配置有效性
    if config.output == "file" && config.file.is_none() {
        return Err("File path required for file output");
    }
    Ok(())
}
```

### 4. 日志格式

**支持的格式**：

1. **Pretty**（默认）：
   ```
   INFO  codebase_translate::scanner: Scanning directory
   │     └─ path=/workspace/src
   ```

2. **Compact**：
   ```
   INFO codebase_translate::scanner: Scanning directory path=/workspace/src
   ```

3. **JSON**：
   ```json
   {"level":"INFO","target":"codebase_translate::scanner","message":"Scanning directory","path":"/workspace/src"}
   ```

### 5. 全局守卫

```rust
pub static LOG_GUARD: OnceLock<Box<dyn Any + Send + Sync>> = OnceLock::new();
```

**目的**：
- 保持日志追加器存活
- 防止日志过早关闭
- 支持测试场景

## 技术选型

### 日志框架
- **tracing**：结构化日志和诊断工具
  - 现代、类型安全的日志 API
  - 异步追踪和跨线程上下文
  - 强大的过滤和采样
  - 与 tokio 深度集成

- **tracing-subscriber**：订阅者实现
  - 提供多种格式化器
  - 支持多种输出目标
  - 环境变量过滤器

- **tracing-appender**：文件日志追加器
  - 非阻塞文件写入
  - 支持滚动日志（可选）
  - 线程安全

## 关键设计要点

### 1. 输出目标

**Stdout**：
```rust
fn init_stdout_logger(filter: EnvFilter, format: &str) -> Result<()> {
    let fmt_layer = match format {
        "json" => Box::new(tracing_subscriber::fmt::layer().json()),
        "compact" => Box::new(tracing_subscriber::fmt::layer().compact()),
        _ => Box::new(tracing_subscriber::fmt::layer().pretty()),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()?;
    Ok(())
}
```

**Stderr**：
```rust
fn init_stderr_logger(filter: EnvFilter, format: &str) -> Result<()> {
    let fmt_layer = match format {
        "json" => Box::new(tracing_subscriber::fmt::layer().json()
            .with_writer(std::io::stderr)),
        "compact" => Box::new(tracing_subscriber::fmt::layer().compact()
            .with_writer(std::io::stderr)),
        _ => Box::new(tracing_subscriber::fmt::layer().pretty()
            .with_writer(std::io::stderr)),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()?;
    Ok(())
}
```

**File**：
```rust
fn init_file_logger(
    config: &LoggingConfig,
    filter: EnvFilter,
    format: &str,
    project_dir: Option<&Path>,
) -> Result<()> {
    // 解析文件路径
    let file_path_str = get_log_file_path(config, project_dir);
    let file_path = Path::new(&file_path_str);

    // 创建日志目录
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 打开日志文件
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path_str)?;

    // 非阻塞写入
    let (non_blocking, guard) = tracing_appender::non_blocking(log_file);
    let _ = LOG_GUARD.set(Box::new(guard));

    // 创建格式化层
    let fmt_layer = match format {
        "json" => Box::new(tracing_subscriber::fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_ansi(false)),
        "compact" => Box::new(tracing_subscriber::fmt::layer()
            .compact()
            .with_writer(non_blocking)
            .with_ansi(false)),
        _ => Box::new(tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)),
    };

    // 注册订阅者
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()?;
    Ok(())
}
```

### 2. 环境过滤器

```rust
let filter = EnvFilter::new(format!("codebase_translate={}", level))
    .add_directive(format!("translator={}", level).parse()?);
```

**设计要点**：
- 主模块：`codebase_translate`
- 翻译模块：`translator`
- 支持不同模块不同级别
- 支持运行时调整

### 3. Span 事件

```rust
let span_events = if config.span_events {
    FmtSpan::CLOSE
} else {
    FmtSpan::NONE
};
```

**Span 事件示例**：
```
INFO  codebase_translate::workflow: Starting translation workflow
│     └─ path=/workspace
INFO  codebase_translate::workflow: Workflow completed in 2.3s
```

### 4. 路径解析

```rust
pub fn get_log_file_path(config: &LoggingConfig, project_dir: Option<&Path>) -> String {
    match &config.file {
        Some(path) => {
            // 如果是相对路径，相对于项目目录
            if Path::new(path).is_relative() {
                if let Some(dir) = project_dir {
                    dir.join(path).to_string_lossy().to_string()
                } else {
                    path.clone()
                }
            } else {
                path.clone()
            }
        }
        None => {
            // 默认路径
            if let Some(dir) = project_dir {
                dir.join(".translate.log").to_string_lossy().to_string()
            } else {
                ".translate.log".to_string()
            }
        }
    }
}
```

### 5. 初始化幂等性

```rust
match tracing_subscriber::registry()
    .with(filter)
    .with(fmt_layer)
    .try_init()
{
    Ok(_) => Ok(()),
    Err(_) => {
        // 日志已初始化，视为成功（便于测试）
        Ok(())
    }
}
```

**设计要点**：
- 允许重复调用
- 测试友好
- 避免重复初始化错误

## 使用示例

### 配置文件

```toml
[logging]
level = "info"
output = "file"
format = "compact"
file = "logs/translate.log"
span_events = true
```

### 初始化

```rust
use codebase_translate::logger::init;
use codebase_translate::config::global::LoggingConfig;

let config = LoggingConfig {
    level: "info".to_string(),
    output: "file".to_string(),
    format: "compact".to_string(),
    file: Some("translate.log".to_string()),
    span_events: true,
};

init(&config, Some(Path::new("/project/path")))?;
```

### 使用日志

```rust
use tracing::{info, debug, warn, error};

info!("Starting translation workflow");
debug!("Processing file: {}", file_path);
warn!("Cache miss for file: {}", file_path);
error!("Translation failed: {}", error);
```

### 结构化字段

```rust
info!(
    file_path = %file_path,
    units_count = units.len(),
    "File processed successfully"
);

error!(
    error = %err,
    file_path = %file_path,
    "Translation failed"
);
```

## 性能考量

1. **非阻塞写入**：
   - 使用 `tracing_appender::non_blocking`
   - 避免阻塞主线程
   - 提高吞吐量

2. **异步日志**：
   - 与 tokio 深度集成
   - 支持异步上下文
   - 线程安全

3. **日志级别过滤**：
   - 编译时优化
   - 运行时过滤
   - 减少不必要的日志

4. **缓冲优化**：
   - 批量写入
   - 减少系统调用
   - 提高 I/O 性能

## 测试支持

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_level_valid() {
        assert_eq!(parse_level("trace"), tracing::Level::TRACE);
        assert_eq!(parse_level("debug"), tracing::Level::DEBUG);
        assert_eq!(parse_level("info"), tracing::Level::INFO);
    }

    #[test]
    fn test_validate_config_stdout() {
        let config = create_test_config("info", "stdout", "pretty", None);
        assert!(validate_config(&config).is_ok());
    }
}
```

**测试要点**：
- 使用 `--test-threads=1` 避免并发初始化
- 验证配置解析
- 测试不同的输出目标

## 扩展性

1. **新的输出目标**：
   - 网络日志（syslog, ELK）
   - 数据库日志
   - 消息队列

2. **新的格式**：
   - 自定义格式化器
   - 彩色输出
   - 模板化格式

3. **高级功能**：
   - 日志采样
   - 日志过滤规则
   - 日志聚合

4. **监控集成**：
   - Metrics 导出
   - 分布式追踪
   - 告警集成