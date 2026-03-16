# 代码库翻译工具架构设计文档

## 1. 概述

### 1.1 设计目标

代码库翻译工具采用分层架构设计，核心设计原则：

- **可扩展性**：翻译器模块支持多API接入（DeepLX、LLM等）
- **可维护性**：清晰的模块边界和接口定义
- **可测试性**：依赖注入和接口抽象便于单元测试
- **性能**：并发处理、缓存机制、批量请求优化

### 1.2 技术栈

- **语言**: Go 1.21+
- **配置**: TOML
- **并发**: Goroutines + Channels
- **限流**: golang.org/x/time/rate
- **HTTP客户端**: github.com/imroc/req/v3

---

## 2. 整体架构

### 2.1 架构分层

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI Layer (cmd/)                          │
│  - 命令行参数解析                                                  │
│  - 配置初始化                                                      │
│  - 应用生命周期管理                                                │
├─────────────────────────────────────────────────────────────────┤
│                      Application Layer (internal/app/)            │
│  - 用例编排 (TranslationUseCase)                                   │
│  - 工作流协调                                                      │
│  - 错误处理与报告                                                  │
├─────────────────────────────────────────────────────────────────┤
│                      Domain Layer (internal/domain/)              │
│  - 实体定义 (File, TranslationUnit, CacheEntry)                    │
│  - 领域服务接口 (Translator, Parser, Cache)                        │
│  - 值对象 (Language, FileType)                                     │
├─────────────────────────────────────────────────────────────────┤
│                    Infrastructure Layer (internal/)               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │   scanner   │ │   parser    │ │  translator │ │    cache    │ │
│  │  (文件扫描)  │ │  (代码解析)  │ │  (翻译实现)  │ │  (缓存管理)  │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                 │
│  │   writer    │ │   config    │ │    encoding │                 │
│  │  (文件写入)  │ │  (配置管理)  │ │  (编码处理)  │                 │
│  └─────────────┘ └─────────────┘ └─────────────┘                 │
├─────────────────────────────────────────────────────────────────┤
│                    Interface Layer (pkg/)                         │
│  - 公共接口和类型定义                                              │
│  - 外部可复用的工具函数                                            │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 模块职责

| 模块 | 职责 | 关键接口 |
|------|------|----------|
| `scanner` | 递归扫描目录，识别代码文件，检测文件编码 | `Scanner` |
| `parser` | 解析代码文件，提取注释和字符串 | `Parser`, `LanguageParser` |
| `translator` | 翻译服务抽象和具体实现 | `Translator`, `BatchTranslator` |
| `cache` | 基于文件哈希的翻译缓存 | `Cache` |
| `writer` | 文件写入，编码转换 | `Writer` |
| `config` | 配置解析和合并 | `Config`, `ConfigLoader` |
| `encoding` | 文件编码检测和转换 | `EncodingDetector`, `Encoder` |

---

## 3. 核心接口设计

### 3.1 翻译器接口 (internal/translator)

翻译器模块采用**策略模式**，支持多种翻译API实现。

```go
// pkg/translator/translator.go
package translator

import "context"

// Translator 翻译器接口
type Translator interface {
    // Translate 单条文本翻译
    Translate(ctx context.Context, req TranslateRequest) (*TranslateResponse, error)
    
    // Name 返回翻译器名称
    Name() string
    
    // Close 关闭翻译器，释放资源
    Close() error
}

// BatchTranslator 批量翻译接口
type BatchTranslator interface {
    Translator
    
    // TranslateBatch 批量翻译
    TranslateBatch(ctx context.Context, reqs []TranslateRequest) ([]TranslateResponse, error)
    
    // SetRateLimit 设置速率限制
    SetRateLimit(requestsPerSecond int, burst int)
}

// TranslateRequest 翻译请求
type TranslateRequest struct {
    Text       string
    SourceLang string // 可选，空字符串表示自动检测
    TargetLang string
}

// TranslateResponse 翻译响应
type TranslateResponse struct {
    OriginalText   string
    TranslatedText string
    SourceLang     string // 实际检测到的源语言
    TargetLang     string
    Alternatives   []string // 备选翻译
}

// TranslatorFactory 翻译器工厂
type TranslatorFactory interface {
    Create(config TranslatorConfig) (Translator, error)
}

// TranslatorConfig 翻译器配置
type TranslatorConfig struct {
    Type       string            // "deeplx", "openai", "anthropic" 等
    APIKey     string
    Endpoint   string            // 自定义API端点
    ProxyURL   string
    Timeout    int               // 秒
    MaxRetries int
    Extra      map[string]interface{} // 各实现特有的配置
}
```

### 3.2 解析器接口 (internal/parser)

```go
// internal/parser/parser.go
package parser

import "context"

// Parser 代码解析器接口
type Parser interface {
    // Parse 解析文件，提取可翻译单元
    Parse(ctx context.Context, file *domain.File) ([]TranslationUnit, error)
    
    // Supports 判断是否支持该文件类型
    Supports(filename string) bool
}

// TranslationUnit 可翻译单元
type TranslationUnit struct {
    ID          string
    Type        UnitType      // Comment, DocString, ErrorMessage
    Content     string        // 原始内容
    StartPos    Position      // 在文件中的起始位置
    EndPos      Position      // 在文件中的结束位置
    Language    string        // 检测到的语言
    ShouldTranslate bool      // 是否需要翻译
}

type UnitType int

const (
    UnitTypeComment UnitType = iota
    UnitTypeDocString
    UnitTypeErrorMessage
)

type Position struct {
    Line   int
    Column int
    Offset int
}
```

### 3.3 缓存接口 (internal/cache)

```go
// internal/cache/cache.go
package cache

import "context"

// Cache 文件哈希缓存接口
type Cache interface {
    // Get 获取缓存的翻译结果
    Get(ctx context.Context, fileHash string) (*CacheEntry, bool)
    
    // Set 设置缓存
    Set(ctx context.Context, entry *CacheEntry) error
    
    // Invalidate 使缓存失效
    Invalidate(ctx context.Context, fileHash string) error
    
    // Clear 清空所有缓存
    Clear(ctx context.Context) error
    
    // Close 关闭缓存
    Close() error
}

// CacheEntry 缓存条目
type CacheEntry struct {
    FileHash        string
    FilePath        string
    LastModified    int64
    TranslationUnits []TranslatedUnit
}

type TranslatedUnit struct {
    UnitID      string
    Original    string
    Translated  string
    SourceLang  string
    TargetLang  string
}
```

### 3.4 扫描器接口 (internal/scanner)

```go
// internal/scanner/scanner.go
package scanner

import "context"

// Scanner 文件扫描器接口
type Scanner interface {
    // Scan 扫描目录，返回符合条件的文件列表
    Scan(ctx context.Context, opts ScanOptions) (<-chan FileInfo, <-chan error)
}

// ScanOptions 扫描选项
type ScanOptions struct {
    RootPath       string
    IncludePatterns []string
    ExcludePatterns []string
    FollowSymlinks bool
}

// FileInfo 文件信息
type FileInfo struct {
    Path         string
    Size         int64
    ModifiedTime int64
    Encoding     string // 检测到的编码
}
```

---

## 4. 翻译器模块详细设计

### 4.1 模块结构

```
internal/translator/
├── interface.go          # 核心接口定义
├── factory.go            # 翻译器工厂
├── deeplx/               # DeepLX 实现
│   ├── client.go         # HTTP客户端
│   ├── translator.go     # Translator接口实现
│   ├── batch.go          # 批量翻译实现
│   ├── types.go          # DeepLX特有类型
│   └── utils.go          # 工具函数
├── llm/                  # LLM API 实现（预留）
│   ├── openai/           # OpenAI API
│   ├── anthropic/        # Claude API
│   └── common/           # LLM通用组件
└── mock/                 # 测试模拟实现
    └── mock_translator.go
```

### 4.2 工厂模式实现

```go
// internal/translator/factory.go
package translator

import "fmt"

// Factory 翻译器工厂
type Factory struct {
    creators map[string]Creator
}

// Creator 翻译器创建函数
type Creator func(config TranslatorConfig) (Translator, error)

// NewFactory 创建工厂实例
func NewFactory() *Factory {
    f := &Factory{
        creators: make(map[string]Creator),
    }
    // 注册内置实现
    f.Register("deeplx", deeplxCreator)
    f.Register("openai", openaiCreator)
    f.Register("anthropic", anthropicCreator)
    return f
}

// Register 注册翻译器创建函数
func (f *Factory) Register(typ string, creator Creator) {
    f.creators[typ] = creator
}

// Create 创建翻译器实例
func (f *Factory) Create(config TranslatorConfig) (Translator, error) {
    creator, ok := f.creators[config.Type]
    if !ok {
        return nil, fmt.Errorf("unsupported translator type: %s", config.Type)
    }
    return creator(config)
}

// SupportedTypes 返回支持的翻译器类型
func (f *Factory) SupportedTypes() []string {
    types := make([]string, 0, len(f.creators))
    for t := range f.creators {
        types = append(types, t)
    }
    return types
}
```

### 4.3 DeepLX 实现适配

```go
// internal/translator/deeplx/translator.go
package deeplx

import (
    "context"
    "github.com/project/translator/internal/translator"
)

// DeepLXTranslator 适配现有实现到统一接口
type DeepLXTranslator struct {
    batchTranslator *BatchTranslator
    config          *Config
}

// NewDeepLXTranslator 创建DeepLX翻译器
func NewDeepLXTranslator(config *Config) *DeepLXTranslator {
    return &DeepLXTranslator{
        batchTranslator: NewBatchTranslator(config, nil),
        config:          config,
    }
}

// Translate 实现Translator接口
func (t *DeepLXTranslator) Translate(ctx context.Context, req translator.TranslateRequest) (*translator.TranslateResponse, error) {
    result, err := t.batchTranslator.translateWithRetry(req.Text, req.SourceLang, req.TargetLang)
    if err != nil {
        return nil, err
    }
    
    return &translator.TranslateResponse{
        OriginalText:   req.Text,
        TranslatedText: result.Data,
        SourceLang:     result.SourceLang,
        TargetLang:     result.TargetLang,
        Alternatives:   result.Alternatives,
    }, nil
}

// TranslateBatch 批量翻译
func (t *DeepLXTranslator) TranslateBatch(ctx context.Context, reqs []translator.TranslateRequest) ([]translator.TranslateResponse, error) {
    // 复用现有的批量翻译逻辑
    return t.batchTranslator.translateBatch(ctx, reqs)
}

// Name 返回翻译器名称
func (t *DeepLXTranslator) Name() string {
    return "deeplx"
}

// Close 关闭翻译器
func (t *DeepLXTranslator) Close() error {
    return nil
}
```

### 4.4 LLM 实现预留

```go
// internal/translator/llm/common/types.go
package common

// LLMConfig LLM翻译器通用配置
type LLMConfig struct {
    APIKey       string
    Model        string
    Endpoint     string
    MaxTokens    int
    Temperature  float64
    SystemPrompt string // 翻译专用系统提示词
}

// PromptTemplate 提示词模板
type PromptTemplate struct {
    System string
    User   string
}

// DefaultTranslationPrompt 默认翻译提示词
var DefaultTranslationPrompt = PromptTemplate{
    System: `You are a professional code translator. Translate the following text from {source_lang} to {target_lang}.
Rules:
1. Preserve markdown formatting
2. Keep code blocks unchanged
3. Maintain special markers like TODO, FIXME, NOTE
4. Handle placeholders like %s, {} correctly
5. Only translate natural language, not code identifiers`,
    User: `Translate: {text}`,
}
```

---

## 5. 数据流设计

### 5.1 翻译流程

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Config    │────▶│   Scanner   │────▶│   Parser    │────▶│   Cache     │
│   Loading   │     │   (并发)     │     │ (语言检测)   │     │  (检查)     │
└─────────────┘     └─────────────┘     └─────────────┘     └──────┬──────┘
                                                                   │
                              ┌────────────────────────────────────┘
                              │ 缓存命中: 跳过
                              ▼ 缓存未命中
                       ┌─────────────┐
                       │  Translator │
                       │  (批量翻译)  │
                       └──────┬──────┘
                              │
                              ▼
                       ┌─────────────┐     ┌─────────────┐
                       │   Writer    │────▶│   Report    │
                       │  (编码转换)  │     │  (统计报告)  │
                       └─────────────┘     └─────────────┘
```

### 5.2 并发模型

```
┌─────────────────────────────────────────────────────────────────┐
│                        Worker Pool Pattern                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────┐    ┌──────────────┐    ┌──────────────────┐      │
│   │  File    │───▶│   Job        │───▶│   Worker Pool    │      │
│   │  Queue   │    │   Channel    │    │   (可配置数量)    │      │
│   └──────────┘    └──────────────┘    └────────┬─────────┘      │
│                                                │                 │
│                       ┌────────────────────────┘                 │
│                       │                                          │
│                       ▼                                          │
│   ┌─────────────────────────────────────────┐                   │
│   │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐       │                   │
│   │  │ W1  │ │ W2  │ │ W3  │ │ W4  │  ...  │  文件处理Worker    │
│   │  └──┬──┘ └──┬──┘ └──┬──┘ └──┬──┘       │                   │
│   └─────┼───────┼───────┼───────┼──────────┘                   │
│         │       │       │       │                               │
│         └───────┴───────┴───────┘                               │
│                   │                                              │
│                   ▼                                              │
│   ┌─────────────────────────────────────────┐                   │
│   │         Translation Batch Queue         │                   │
│   │    (聚合多个文件的翻译请求，批量API调用)   │                   │
│   └─────────────────────────────────────────┘                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. 配置设计

### 6.1 TOML 配置结构

```toml
# 翻译设置
[translate]
provider = "deeplx"           # 翻译服务提供商: deeplx, openai, anthropic
source_lang = "ZH"            # 源语言
source_lang = "AUTO"          # 或自动检测
target_lang = "EN"            # 目标语言

# DeepLX 特定配置
[translate.deeplx]
proxy_url = ""
dl_session = ""
rate_limit = 10               # 每秒请求数

# OpenAI 特定配置（预留）
[translate.openai]
api_key = "${OPENAI_API_KEY}"  # 支持环境变量
model = "gpt-4"
endpoint = "https://api.openai.com/v1"
max_tokens = 2000
temperature = 0.3

# Anthropic 特定配置（预留）
[translate.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-3-sonnet"

# 文件包含/排除规则
[include]
patterns = ["**/*.go", "**/*.py", "**/*.js", "**/*.ts", "**/*.java"]

[exclude]
patterns = [
    "vendor/**",
    "node_modules/**",
    "*.min.js",
    "*_test.go",
    ".git/**"
]

# 缓存配置
[cache]
enabled = true
directory = ".translator-cache"

# 并发配置
[concurrency]
workers = 5                   # 文件处理并发数
batch_size = 50               # 批量翻译大小

# 日志配置
[log]
level = "info"                # debug, info, warn, error
format = "text"               # text, json
```

### 6.2 配置加载优先级

```
┌─────────────────────────────────────────────────────────────┐
│                    配置优先级（从高到低）                      │
├─────────────────────────────────────────────────────────────┤
│  1. 命令行参数 (--source-lang, --target-lang 等)              │
│  2. 环境变量 (CODEBASE_TRANSLATE_*)                           │
│  3. 指定的配置文件 (--config)                                 │
│  4. 默认配置文件 (./translator.toml)                  │
│  5. 内置默认值                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. 错误处理设计

### 7.1 错误类型定义

```go
// pkg/errors/errors.go
package errors

import "errors"

// 错误分类
var (
    // 配置错误
    ErrConfigNotFound   = errors.New("configuration file not found")
    ErrInvalidConfig    = errors.New("invalid configuration")
    
    // 文件错误
    ErrFileNotFound     = errors.New("file not found")
    ErrPermissionDenied = errors.New("permission denied")
    ErrEncodingDetect   = errors.New("encoding detection failed")
    
    // 翻译错误
    ErrTranslationFailed = errors.New("translation failed")
    ErrRateLimited      = errors.New("rate limited")
    ErrAPITimeout       = errors.New("API timeout")
    
    // 缓存错误
    ErrCacheCorrupted   = errors.New("cache corrupted")
)

// TranslationError 翻译错误
type TranslationError struct {
    Code      int    // HTTP状态码或内部错误码
    Message   string
    Retryable bool   // 是否可重试
    Provider  string // 翻译提供商
}

func (e *TranslationError) Error() string {
    return fmt.Sprintf("[%s] %d: %s", e.Provider, e.Code, e.Message)
}
```

### 7.2 错误处理策略

| 错误类型 | 处理策略 | CI模式行为 |
|----------|----------|------------|
| 配置错误 | 立即退出，显示详细错误 | 非零退出码 |
| 单个文件错误 | 记录错误，继续处理其他文件 | 非零退出码 |
| API限流 | 指数退避重试 | 指数退避重试 |
| API错误 | 重试3次后跳过 | 非零退出码 |
| 编码错误 | 尝试其他编码或跳过 | 记录警告 |
| 缓存错误 | 忽略缓存，重新处理 | 记录警告 |

---

## 8. 测试策略

### 8.1 测试分层

```
┌─────────────────────────────────────────────────────────────┐
│                      E2E Tests (tests/e2e/)                  │
│  - 完整工作流测试                                             │
│  - CLI 集成测试                                               │
├─────────────────────────────────────────────────────────────┤
│                   Integration Tests (tests/integration/)      │
│  - 模块间集成测试                                             │
│  - 外部API模拟测试                                            │
├─────────────────────────────────────────────────────────────┤
│                      Unit Tests (各模块内)                     │
│  - 接口实现测试                                               │
│  - 工具函数测试                                               │
│  - Mock 测试                                                  │
└─────────────────────────────────────────────────────────────┘
```

### 8.2 Mock 实现

```go
// internal/translator/mock/mock_translator.go
package mock

// MockTranslator 模拟翻译器，用于测试
type MockTranslator struct {
    translations map[string]string
    delay        time.Duration
    errorRate    float64
}

func (m *MockTranslator) Translate(ctx context.Context, req translator.TranslateRequest) (*translator.TranslateResponse, error) {
    // 模拟延迟
    if m.delay > 0 {
        time.Sleep(m.delay)
    }
    
    // 模拟错误
    if rand.Float64() < m.errorRate {
        return nil, errors.New("mock translation error")
    }
    
    // 返回预设翻译或简单转换
    translated := m.translations[req.Text]
    if translated == "" {
        translated = "[Translated] " + req.Text
    }
    
    return &translator.TranslateResponse{
        OriginalText:   req.Text,
        TranslatedText: translated,
        SourceLang:     req.SourceLang,
        TargetLang:     req.TargetLang,
    }, nil
}
```

---

## 9. 目录结构

```
translator/
├── cmd/
│   └── translator/
│       └── main.go                 # 程序入口
├── internal/
│   ├── app/
│   │   ├── app.go                  # 应用初始化
│   │   └── usecase.go              # 用例编排
│   ├── domain/
│   │   ├── file.go                 # 文件实体
│   │   ├── translation.go          # 翻译实体
│   │   └── language.go             # 语言值对象
│   ├── scanner/
│   │   ├── scanner.go              # 扫描器接口
│   │   └── fs_scanner.go           # 文件系统实现
│   ├── parser/
│   │   ├── parser.go               # 解析器接口
│   │   ├── go_parser.go            # Go语言解析
│   │   ├── python_parser.go        # Python解析
│   │   └── js_parser.go            # JS/TS解析
│   ├── translator/
│   │   ├── interface.go            # 核心接口
│   │   ├── factory.go              # 工厂
│   │   ├── deeplx/                 # DeepLX实现
│   │   │   ├── client.go
│   │   │   ├── translator.go       # 适配器
│   │   │   ├── batch.go
│   │   │   ├── types.go
│   │   │   └── utils.go
│   │   └── llm/                    # LLM实现（预留）
│   │       ├── common/
│   │       ├── openai/
│   │       └── anthropic/
│   ├── cache/
│   │   ├── cache.go                # 缓存接口
│   │   └── file_cache.go           # 文件缓存实现
│   ├── writer/
│   │   ├── writer.go               # 写入器接口
│   │   └── file_writer.go          # 文件写入实现
│   ├── config/
│   │   ├── config.go               # 配置结构
│   │   └── loader.go               # 配置加载
│   ├── encoding/
│   │   ├── detector.go             # 编码检测
│   │   └── converter.go            # 编码转换
│   └── reporter/
│       └── reporter.go             # 报告生成
├── pkg/
│   ├── translator/
│   │   └── types.go                # 公共接口
│   ├── errors/
│   │   └── errors.go               # 公共错误
│   └── utils/
│       └── utils.go                # 公共工具
├── tests/
│   ├── integration/                # 集成测试
│   └── e2e/                        # E2E测试
├── docs/
│   ├── requirements.md             # 需求文档
│   ├── architecture.md             # 本文档
│   └── deeplx.md                   # DeepLX API文档
├── go.mod
├── go.sum
└── README.md
```

---

## 10. 演进计划

### Phase 1: 基础功能 (MVP)
- [x] DeepLX 翻译实现
- [ ] 统一翻译器接口
- [ ] 文件扫描和解析
- [ ] TOML配置支持
- [ ] 基础缓存机制

### Phase 2: 完善功能
- [ ] 多语言解析器 (Python, JS/TS, Java)
- [ ] 编码检测与转换
- [ ] 并发处理优化
- [ ] 完整 CLI 功能

### Phase 3: 扩展功能
- [ ] LLM API 支持 (OpenAI, Anthropic)
- [ ] 插件系统
- [ ] 增量翻译优化
- [ ] 翻译质量控制

### Phase 4: 高级功能
- [ ] Web UI
- [ ] IDE 插件
- [ ] 团队协作功能
- [ ] 翻译记忆库

---

## 11. 附录

### 11.1 术语表

| 术语 | 说明 |
|------|------|
| TranslationUnit | 可翻译的最小文本单元 |
| Provider | 翻译服务提供商 (DeepLX, OpenAI等) |
| CacheEntry | 缓存条目，包含文件哈希和翻译结果 |
| Worker | 并发处理单元 |
| Batch | 批量翻译请求集合 |

### 11.2 参考资料

- [DeepLX GitHub](https://github.com/xixu-me/deeplx)
- [Go Clean Architecture](https://github.com/bxcodec/go-clean-arch)
- [Go Project Layout](https://github.com/golang-standards/project-layout)
