# 日志系统需求文档

## 1. 项目概述

Codebase Translate 是一个命令行工具，用于自动翻译代码库中的注释、文档字符串和错误消息。项目采用分层架构设计，支持多种翻译提供商和智能增量处理。

## 2. 当前日志使用现状

### 2.1 现有日志实现

项目目前使用 Go 标准库的 `log` 包：

```go
logger := log.New(os.Stdout, "", log.LstdFlags)
```

### 2.2 现有日志调用点

- **应用层** ([app.go](e:\project\codebase-translate\internal\app\app.go#L37)): 记录启动信息、扫描进度、错误信息
- **命令行层** ([main.go](e:\project\codebase-translate\cmd\translator\main.go)): 使用 `fmt.Fprintf` 输出错误到 stderr
- **其他模块**: 部分模块使用标准 `fmt.Println` 或 `log.Printf`

### 2.3 现有日志功能

- ✅ 基础日志输出到 stdout
- ✅ 时间戳格式（标准库默认）
- ✅ 支持 verbose 标志（`-verbose`）
- ❌ 缺少日志级别控制
- ❌ 缺少结构化日志
- ❌ 缺少日志文件输出
- ❌ 缺少上下文传递

## 3. 项目特点分析

### 3.1 架构特点

- **CLI 工具**: 主要在命令行环境中运行
- **CI/CD 友好**: 需要支持自动化流程集成
- **分层架构**: CLI 层 → 应用层 → 领域层 → 基础设施层 → 接口层
- **并发处理**: 使用 Goroutines + Channels 进行并发处理

### 3.2 功能特点

- **文件处理**: 扫描、解析、翻译、写入大量文件
- **多提供商支持**: DeepLX、LLM、Tencent Cloud
- **速率限制**: 实现请求速率控制
- **重试机制**: 失败自动重试
- **缓存机制**: 基于文件哈希的增量处理
- **报告生成**: 统计处理结果和错误信息

### 3.3 性能要求

- **低开销**: 日志系统不应显著影响翻译性能
- **高并发**: 支持并发环境下的日志记录
- **零分配**: 优先选择零内存分配的日志库

## 4. 日志需求

### 4.1 功能需求

#### 4.1.1 日志级别

支持以下日志级别（从低到高）：

| 级别 | 用途 | 示例场景 |
|------|------|----------|
| Trace | 最详细的调试信息 | 函数入口/出口、详细参数 |
| Debug | 调试信息 | 缓存命中/未命中、详细处理步骤 |
| Info | 一般信息 | 启动信息、处理进度、配置加载 |
| Warn | 警告信息 | 降级处理、非关键错误 |
| Error | 错误信息 | API 失败、文件读写错误 |
| Fatal | 致命错误 | 配置错误、无法恢复的错误 |

#### 4.1.2 日志格式

**开发环境**（默认）:
- 人类可读的控制台输出
- 彩色显示不同级别
- 包含时间戳、级别、消息、调用位置

**生产环境**:
- JSON 格式输出
- 便于日志聚合和分析（如 ELK、Splunk）
- 结构化字段

#### 4.1.3 输出目标

- **标准输出 (stdout)**: Info 及以上级别
- **标准错误 (stderr)**: Error 及以上级别
- **日志文件**: 可配置的文件路径，支持日志轮转
- **多输出**: 同时输出到多个目标

#### 4.1.4 结构化日志

支持结构化字段：

```go
logger.Info().
    Str("provider", "deeplx").
    Str("file", "example.go").
    Int("segments", 42).
    Dur("duration", 123*time.Millisecond).
    Msg("Translation completed")
```

#### 4.1.5 上下文传递

支持在上下文中传递 logger：

```go
func processFile(ctx context.Context, file *domain.File) {
    logger := zerolog.Ctx(ctx)
    logger.Info().Str("file", file.Path).Msg("Processing file")
}
```

### 4.2 配置需求

#### 4.2.1 配置文件

在 `translator.toml` 中添加日志配置：

```toml
[logging]
level = "info"           # 日志级别: trace, debug, info, warn, error, fatal
format = "console"       # 输出格式: console, json
output = "stdout"        # 输出目标: stdout, stderr, file, both
file = "translator.log"  # 日志文件路径（当 output 为 file 时）
max_size = 100           # 日志文件最大大小 (MB)
max_backups = 3          # 保留的旧日志文件数量
max_age = 7              # 保留旧日志文件的最大天数
compress = true          # 是否压缩旧日志文件
```

#### 4.2.2 命令行参数

- `-verbose`: 设置日志级别为 debug
- `-log-level`: 覆盖配置文件中的日志级别
- `-log-file`: 指定日志文件路径

### 4.3 模块化日志

不同模块支持独立的日志级别：

```toml
[logging.level]
default = "info"
scanner = "debug"
parser = "info"
translator = "warn"
cache = "debug"
writer = "info"
```

### 4.4 性能需求

- **零分配**: 优先选择零内存分配的日志库
- **低延迟**: 日志记录不应阻塞主流程
- **异步写入**: 支持异步日志写入（可选）

### 4.5 集成需求

- **错误处理**: 与现有的错误处理系统集成
- **报告系统**: 与 reporter 模块集成，提供详细的统计信息
- **配置系统**: 与现有的配置系统集成

## 5. 日志库技术选型

### 5.1 候选库对比

| 特性 | Zap | Logrus | Zerolog |
|------|-----|--------|---------|
| **性能** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **零分配** | ✅ | ❌ | ✅ |
| **结构化日志** | ✅ | ✅ | ✅ |
| **JSON 输出** | ✅ | ✅ | ✅ |
| **控制台输出** | ✅ | ✅ | ✅ |
| **日志级别** | ✅ | ✅ | ✅ |
| **上下文支持** | ✅ | ❌ | ✅ |
| **API 易用性** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **社区活跃度** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Benchmark Score** | 94.6 | 68.5 | 92.5 |
| **Code Snippets** | 47 | 24 | 698 |

### 5.2 推荐选择：Zerolog

**选择理由：**

1. **卓越性能**: 零内存分配，性能与 Zap 相当，优于 Logrus
2. **简洁 API**: 链式调用，易于使用，比 Zap 更简洁
3. **丰富的文档**: 698 个代码示例，远超其他库
4. **上下文支持**: 原生支持上下文传递，适合分层架构
5. **灵活配置**: 支持多种输出格式和目标
6. **活跃社区**: Source Reputation: High，Benchmark Score: 92.5

**使用示例：**

```go
import "github.com/rs/zerolog/log"

logger := log.With().
    Str("provider", "deeplx").
    Str("file", "example.go").
    Logger()

logger.Info().
    Int("segments", 42).
    Dur("duration", 123*time.Millisecond).
    Msg("Translation completed")
```

**备选方案：Zap**

如果项目需要更复杂的日志配置（如多输出、自定义编码器），Zap 是一个很好的选择。Zap 在 Benchmark Score 上略高（94.6），但 API 相对复杂。

### 5.3 依赖添加

```go
require (
    github.com/rs/zerolog v1.32.0
)
```

## 6. 实现建议

### 6.1 日志模块结构

```
internal/
└── logger/
    ├── logger.go          # 日志接口定义
    ├── zerolog.go         # Zerolog 实现
    ├── config.go          # 日志配置
    └── context.go         # 上下文工具
```

### 6.2 核心接口

```go
type Logger interface {
    Trace() *Event
    Debug() *Event
    Info() *Event
    Warn() *Event
    Error() *Event
    Fatal() *Event
    
    With() *Context
    Ctx(ctx context.Context) *Logger
}
```

### 6.3 集成步骤

1. **添加依赖**: `go get github.com/rs/zerolog`
2. **创建日志模块**: 实现 Logger 接口
3. **更新配置**: 在 config 包中添加日志配置
4. **替换现有日志**: 逐步替换 `log.Printf` 和 `fmt.Printf`
5. **添加测试**: 编写日志模块的单元测试
6. **更新文档**: 更新 README 和使用文档

### 6.4 迁移策略

- **阶段 1**: 引入 Zerolog，保留现有日志，并行运行
- **阶段 2**: 逐步替换关键模块的日志（app.go、translator）
- **阶段 3**: 替换所有模块的日志
- **阶段 4**: 移除标准库 log 包的依赖

## 7. 日志内容规划

### 7.1 应用层日志

```
[INFO] Starting translation for directory: ./my-project
[INFO] Provider: deeplx, Source: ZH, Target: EN
[INFO] Found 42 files to process
[INFO] Translation completed: 42 files, 1234 segments, 45.2s
```

### 7.2 扫描器日志

```
[DEBUG] Scanning directory: ./my-project
[DEBUG] File matched: example.go (include pattern: *.go)
[DEBUG] File excluded: vendor/lib.go (exclude pattern: vendor/*)
```

### 7.3 解析器日志

```
[DEBUG] Parsing file: example.go
[DEBUG] Extracted 15 translatable segments from example.go
[WARN] No translatable segments found in README.md
```

### 7.4 翻译器日志

```
[INFO] Translating batch: 10 segments (provider: deeplx)
[DEBUG] Rate limit: 10 req/s, burst: 5
[WARN] API request failed: rate limit exceeded, retrying in 1s
[ERROR] Translation failed after 3 retries: timeout
```

### 7.5 缓存日志

```
[DEBUG] Cache hit: abc123def456 (example.go)
[DEBUG] Cache miss: xyz789uvw012 (new file)
[INFO] Cleaned 5 orphaned cache entries
```

### 7.6 写入器日志

```
[INFO] Writing file: example.go (backup: example.go.bak)
[WARN] Preview mode enabled, no files will be modified
[ERROR] Failed to write file: permission denied
```

## 8. 测试需求

### 8.1 单元测试

- 日志配置加载测试
- 日志级别过滤测试
- 结构化日志格式测试
- 上下文传递测试

### 8.2 集成测试

- 日志输出到文件测试
- 日志轮转测试
- 多输出目标测试
- 性能基准测试

### 8.3 性能测试

- 零分配验证
- 高并发日志记录性能
- 大量日志写入性能

## 9. 非功能性需求

### 9.1 可维护性

- 清晰的日志消息
- 一致的日志格式
- 完善的文档和示例

### 9.2 可扩展性

- 支持自定义输出格式
- 支持自定义字段
- 支持插件式扩展

### 9.3 兼容性

- 向后兼容现有日志输出
- 支持 Go 1.24.5+
- 跨平台支持（Windows、Linux、macOS）

## 10. 风险与挑战

### 10.1 性能风险

- **风险**: 日志记录可能影响翻译性能
- **缓解**: 使用零分配日志库，异步写入（可选）

### 10.2 迁移风险

- **风险**: 替换现有日志可能引入 bug
- **缓解**: 分阶段迁移，保留现有日志并行运行

### 10.3 配置复杂性

- **风险**: 过多的配置选项增加用户负担
- **缓解**: 提供合理的默认配置，简化配置项

## 11. 附录

### 11.1 参考文档

- [Zerolog GitHub](https://github.com/rs/zerolog)
- [Zerolog Context7 Documentation](https://context7.com/rs/zerolog)
- [Go Logging Best Practices](https://go.dev/doc/tutorial/add-a-test)

### 11.2 相关文件

- [app.go](e:\project\codebase-translate\internal\app\app.go) - 应用层日志使用
- [main.go](e:\project\codebase-translate\cmd\translator\main.go) - 命令行参数处理
- [config.go](e:\project\codebase-translate\internal\config\config.go) - 配置管理

### 11.3 版本历史

| 版本 | 日期 | 作者 | 说明 |
|------|------|------|------|
| 1.0 | 2026-03-12 | AI Assistant | 初始版本 |