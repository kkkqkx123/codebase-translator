# 多供应商多模型轮询架构设计方案

## 1. 设计目标

为 Codebase Translate 工具设计一套多供应商、多模型轮询机制，实现：

- **负载均衡**: 在多个 LLM 供应商/模型之间均匀分配翻译请求
- **故障转移**: 当某个供应商不可用时自动切换到备用供应商
- **灵活配置**: 支持多种轮询策略（轮询、加权、优先级）
- **向后兼容**: 保持现有单供应商配置的兼容性

## 2. 当前架构分析

### 2.1 现有配置结构

```
GlobalConfig (全局配置)
├── Provider: string          # "deeplx" 或 "llm"
├── DeepLX: DeepLXGlobalConfig
└── LLM: LLMGlobalConfig      # 单一 LLM 配置
    ├── BaseURL: string
    ├── APIKey: string
    ├── Model: string
    └── ...
```

### 2.2 现有翻译器结构

```
internal/translator/
├── llm/
│   ├── translate.go          # Translator 实现
│   ├── types.go              # Config 定义
│   └── batch.go              # 批量翻译包装
├── common/
│   ├── batch.go              # BatchTranslator 通用实现
│   └── types.go              # 通用类型定义
└── factory.go                # 翻译器工厂
```

### 2.3 现有问题

1. **单点故障**: 只能配置一个 LLM 供应商，一旦失效整个翻译流程中断
2. **无负载均衡**: 无法利用多个供应商的配额和性能优势
3. **扩展性差**: 新增供应商需要修改多处代码

## 3. 设计方案

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    MultiProviderManager                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              Provider Pool (供应商池)                    │    │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │    │
│  │  │Provider1│  │Provider2│  │Provider3│  │Provider4│    │    │
│  │  │ OpenAI  │  │ Azure   │  │Claude   │  │Local LLM│    │    │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘    │    │
│  │       └─────────────┴─────────────┴─────────────┘       │    │
│  │                         │                               │    │
│  │              ┌──────────┴──────────┐                    │    │
│  │              │   Router (路由器)    │                    │    │
│  │              │  - 轮询策略          │                    │    │
│  │              │  - 健康检查          │                    │    │
│  │              │  - 故障转移          │                    │    │
│  │              └──────────┬──────────┘                    │    │
│  └─────────────────────────┼───────────────────────────────┘    │
└────────────────────────────┼────────────────────────────────────┘
                             │
                    ┌────────┴────────┐
                    │  BatchTranslator  │
                    │  (复用现有实现)   │
                    └─────────────────┘
```

### 3.2 配置结构设计

#### 3.2.1 新全局配置结构

```toml
# 全局配置文件 ~/.config/translator/config.toml

# 主提供商选择（向后兼容）
provider = "llm"

# DeepLX 配置（向后兼容）
[deeplx]
proxy_url = ""
dl_session = ""
rate_limit = 10

# LLM 多供应商配置
[llm]
# 轮询策略: round_robin(轮询), weighted(加权), priority(优先级)
rotation_strategy = "round_robin"

# 健康检查配置
[llm.health_check]
enabled = true
interval = 30           # 健康检查间隔（秒）
timeout = 5             # 健康检查超时（秒）
failure_threshold = 3   # 连续失败次数标记为不可用

# 供应商列表
[[llm.providers]]
id = "openai-primary"
name = "OpenAI GPT-4"
enabled = true
priority = 1            # 优先级（数值越小优先级越高）
weight = 50             # 权重（用于加权轮询）
base_url = "https://api.openai.com/v1"
api_key = "sk-xxx"
model = "gpt-4"
proxy_url = ""
timeout = 60
rate_limit = 10
max_tokens = 4096
temperature = 0.3
extra_headers = { "X-Custom-Header" = "value" }
extra_params = { "top_p" = 0.9 }

[[llm.providers]]
id = "azure-backup"
name = "Azure OpenAI"
enabled = true
priority = 2
weight = 30
base_url = "https://my-resource.openai.azure.com/openai/deployments/my-deployment"
api_key = "azure-api-key"
model = "gpt-4"
timeout = 60
rate_limit = 5

[[llm.providers]]
id = "local-llm"
name = "Local Ollama"
enabled = true
priority = 3
weight = 20
base_url = "http://localhost:11434/v1"
api_key = ""
model = "llama2"
timeout = 120
rate_limit = 20
```

#### 3.2.2 Go 结构体定义

```go
// internal/config/global.go

// LLMGlobalConfig LLM 全局配置（更新）
type LLMGlobalConfig struct {
    RotationStrategy string                `toml:"rotation_strategy"` // round_robin, weighted, priority
    HealthCheck      HealthCheckConfig     `toml:"health_check"`
    Providers        []LLMProviderConfig   `toml:"providers"`
    
    // 向后兼容字段（单供应商模式）
    BaseURL      string                 `toml:"base_url"`
    APIKey       string                 `toml:"api_key"`
    Model        string                 `toml:"model"`
    ProxyURL     string                 `toml:"proxy_url"`
    Timeout      int                    `toml:"timeout"`
    RateLimit    int                    `toml:"rate_limit"`
    MaxTokens    int                    `toml:"max_tokens"`
    Temperature  float64                `toml:"temperature"`
    ExtraHeaders map[string]string      `toml:"extra_headers"`
    ExtraParams  map[string]interface{} `toml:"extra_params"`
}

// HealthCheckConfig 健康检查配置
type HealthCheckConfig struct {
    Enabled          bool `toml:"enabled"`
    Interval         int  `toml:"interval"`          // 秒
    Timeout          int  `toml:"timeout"`           // 秒
    FailureThreshold int  `toml:"failure_threshold"` // 连续失败次数
}

// LLMProviderConfig 单个 LLM 供应商配置
type LLMProviderConfig struct {
    ID           string                 `toml:"id"`
    Name         string                 `toml:"name"`
    Enabled      bool                   `toml:"enabled"`
    Priority     int                    `toml:"priority"`     // 优先级，数值越小优先级越高
    Weight       int                    `toml:"weight"`       // 权重，用于加权轮询
    BaseURL      string                 `toml:"base_url"`
    APIKey       string                 `toml:"api_key"`
    Model        string                 `toml:"model"`
    ProxyURL     string                 `toml:"proxy_url"`
    Timeout      int                    `toml:"timeout"`
    RateLimit    int                    `toml:"rate_limit"`
    MaxTokens    int                    `toml:"max_tokens"`
    Temperature  float64                `toml:"temperature"`
    ExtraHeaders map[string]string      `toml:"extra_headers"`
    ExtraParams  map[string]interface{} `toml:"extra_params"`
}
```

### 3.3 核心组件设计

#### 3.3.1 Provider 接口

```go
// internal/translator/llm/provider.go

package llm

import (
    "context"
    "translator/internal/translator/common"
)

// Provider 定义单个 LLM 供应商接口
type Provider interface {
    // ID 返回供应商唯一标识
    ID() string
    
    // Name 返回供应商名称
    Name() string
    
    // Translate 执行翻译
    Translate(ctx context.Context, text, sourceLang, targetLang string) (*common.TranslateResponse, error)
    
    // HealthCheck 执行健康检查
    HealthCheck(ctx context.Context) error
    
    // IsHealthy 返回供应商健康状态
    IsHealthy() bool
    
    // GetWeight 返回权重（用于加权轮询）
    GetWeight() int
    
    // GetPriority 返回优先级
    GetPriority() int
    
    // Close 关闭供应商连接
    Close() error
}

// BaseProvider 基础供应商实现
type BaseProvider struct {
    config        *ProviderConfig
    translator    *Translator
    healthy       bool
    failureCount  int
    lastCheckTime int64
}
```

#### 3.3.2 ProviderPool 供应商池

```go
// internal/translator/llm/pool.go

package llm

import (
    "context"
    "sync"
    "sync/atomic"
    "time"
)

// RotationStrategy 轮询策略类型
type RotationStrategy string

const (
    RoundRobin RotationStrategy = "round_robin"  // 简单轮询
    Weighted   RotationStrategy = "weighted"      // 加权轮询
    Priority   RotationStrategy = "priority"      // 优先级
)

// ProviderPool 管理多个 LLM 供应商
type ProviderPool struct {
    providers []Provider
    strategy  RotationStrategy
    
    // 轮询状态
    currentIndex uint64
    
    // 加权轮询状态
    totalWeight int
    
    // 健康检查
    healthCheckEnabled bool
    healthCheckInterval time.Duration
    healthCheckTimeout  time.Duration
    failureThreshold    int
    
    // 并发控制
    mu sync.RWMutex
}

// NewProviderPool 创建供应商池
func NewProviderPool(providers []Provider, strategy RotationStrategy) *ProviderPool {
    pool := &ProviderPool{
        providers:          providers,
        strategy:           strategy,
        healthCheckEnabled: true,
        healthCheckInterval: 30 * time.Second,
        healthCheckTimeout:  5 * time.Second,
        failureThreshold:    3,
    }
    
    // 计算总权重
    for _, p := range providers {
        pool.totalWeight += p.GetWeight()
    }
    
    return pool
}

// GetProvider 根据策略获取下一个可用供应商
func (p *ProviderPool) GetProvider() (Provider, error) {
    p.mu.RLock()
    defer p.mu.RUnlock()
    
    switch p.strategy {
    case RoundRobin:
        return p.getRoundRobinProvider()
    case Weighted:
        return p.getWeightedProvider()
    case Priority:
        return p.getPriorityProvider()
    default:
        return p.getRoundRobinProvider()
    }
}

// getRoundRobinProvider 简单轮询
func (p *ProviderPool) getRoundRobinProvider() (Provider, error) {
    if len(p.providers) == 0 {
        return nil, ErrNoAvailableProvider
    }
    
    // 获取当前索引并递增
    idx := atomic.AddUint64(&p.currentIndex, 1) - 1
    
    // 遍历所有供应商，找到第一个健康的
    for i := 0; i < len(p.providers); i++ {
        provider := p.providers[(idx+uint64(i))%uint64(len(p.providers))]
        if provider.IsHealthy() {
            return provider, nil
        }
    }
    
    return nil, ErrNoHealthyProvider
}

// getWeightedProvider 加权轮询
func (p *ProviderPool) getWeightedProvider() (Provider, error) {
    if len(p.providers) == 0 || p.totalWeight == 0 {
        return nil, ErrNoAvailableProvider
    }
    
    // 使用当前索引计算权重位置
    idx := atomic.AddUint64(&p.currentIndex, 1) - 1
    targetWeight := int(idx % uint64(p.totalWeight))
    
    currentWeight := 0
    for _, provider := range p.providers {
        if !provider.IsHealthy() {
            continue
        }
        currentWeight += provider.GetWeight()
        if targetWeight < currentWeight {
            return provider, nil
        }
    }
    
    // 如果没有找到（所有都不健康），尝试返回第一个健康的
    for _, provider := range p.providers {
        if provider.IsHealthy() {
            return provider, nil
        }
    }
    
    return nil, ErrNoHealthyProvider
}

// getPriorityProvider 优先级选择
func (p *ProviderPool) getPriorityProvider() (Provider, error) {
    // 按优先级排序，返回第一个健康的
    var bestProvider Provider
    bestPriority := int(^uint(0) >> 1) // MaxInt
    
    for _, provider := range p.providers {
        if provider.IsHealthy() && provider.GetPriority() < bestPriority {
            bestProvider = provider
            bestPriority = provider.GetPriority()
        }
    }
    
    if bestProvider != nil {
        return bestProvider, nil
    }
    
    return nil, ErrNoHealthyProvider
}

// StartHealthCheck 启动健康检查
func (p *ProviderPool) StartHealthCheck(ctx context.Context) {
    if !p.healthCheckEnabled {
        return
    }
    
    ticker := time.NewTicker(p.healthCheckInterval)
    go func() {
        defer ticker.Stop()
        for {
            select {
            case <-ctx.Done():
                return
            case <-ticker.C:
                p.checkAllProviders(ctx)
            }
        }
    }()
}

// checkAllProviders 检查所有供应商健康状态
func (p *ProviderPool) checkAllProviders(ctx context.Context) {
    var wg sync.WaitGroup
    for _, provider := range p.providers {
        wg.Add(1)
        go func(p Provider) {
            defer wg.Done()
            
            checkCtx, cancel := context.WithTimeout(ctx, p.healthCheckTimeout)
            defer cancel()
            
            if err := p.HealthCheck(checkCtx); err != nil {
                // 健康检查失败，增加失败计数
                p.markUnhealthy()
            } else {
                // 健康检查成功，重置状态
                p.markHealthy()
            }
        }(provider)
    }
    wg.Wait()
}
```

#### 3.3.3 MultiProviderTranslator 多供应商翻译器

```go
// internal/translator/llm/multi_translator.go

package llm

import (
    "context"
    "fmt"
    "translator/internal/translator/common"
)

// MultiProviderTranslator 多供应商翻译器
type MultiProviderTranslator struct {
    pool       *ProviderPool
    maxRetries int
}

// NewMultiProviderTranslator 创建多供应商翻译器
func NewMultiProviderTranslator(pool *ProviderPool, maxRetries int) *MultiProviderTranslator {
    return &MultiProviderTranslator{
        pool:       pool,
        maxRetries: maxRetries,
    }
}

// TranslateSingle 实现 SingleTranslator 接口
func (t *MultiProviderTranslator) TranslateSingle(ctx context.Context, text, sourceLang, targetLang string) (*common.TranslateResponse, error) {
    var lastErr error
    
    // 尝试多个供应商（最多 maxRetries 次）
    for attempt := 0; attempt <= t.maxRetries; attempt++ {
        provider, err := t.pool.GetProvider()
        if err != nil {
            return nil, fmt.Errorf("no available provider: %w", err)
        }
        
        result, err := provider.Translate(ctx, text, sourceLang, targetLang)
        if err == nil {
            return result, nil
        }
        
        lastErr = err
        
        // 检查错误是否可重试
        if !common.IsRetryableError(err) {
            return nil, err
        }
        
        // 标记供应商为不健康
        provider.markUnhealthy()
        
        // 如果不是最后一次尝试，等待后重试
        if attempt < t.maxRetries {
            delay := common.CalculateBackoff(attempt)
            time.Sleep(delay)
        }
    }
    
    return nil, fmt.Errorf("all providers failed, last error: %w", lastErr)
}

// Name 返回翻译器名称
func (t *MultiProviderTranslator) Name() string {
    return "llm-multi-provider"
}

// Close 关闭所有供应商连接
func (t *MultiProviderTranslator) Close() error {
    var errs []error
    for _, provider := range t.pool.providers {
        if err := provider.Close(); err != nil {
            errs = append(errs, err)
        }
    }
    if len(errs) > 0 {
        return fmt.Errorf("failed to close providers: %v", errs)
    }
    return nil
}
```

### 3.4 工厂函数更新

```go
// internal/translator/factory.go

func llmCreator(cfg Config) (BatchTranslator, error) {
    // 检查是否为多供应商配置
    if providers, ok := cfg.Extra["providers"].([]llm.ProviderConfig); ok && len(providers) > 0 {
        return createMultiProviderTranslator(providers, cfg)
    }
    
    // 单供应商模式（向后兼容）
    return createSingleProviderTranslator(cfg)
}

func createMultiProviderTranslator(providerConfigs []llm.ProviderConfig, cfg Config) (BatchTranslator, error) {
    // 创建供应商实例
    providers := make([]llm.Provider, 0, len(providerConfigs))
    for _, pc := range providerConfigs {
        if !pc.Enabled {
            continue
        }
        
        translator, err := llm.NewTranslator(&llm.Config{
            BaseURL:      pc.BaseURL,
            APIKey:       pc.APIKey,
            Model:        pc.Model,
            ProxyURL:     pc.ProxyURL,
            Timeout:      pc.Timeout,
            MaxRetries:   1, // 在 provider 层不重试，由 pool 层处理
            MaxTokens:    pc.MaxTokens,
            Temperature:  pc.Temperature,
            ExtraHeaders: pc.ExtraHeaders,
            ExtraParams:  pc.ExtraParams,
        })
        if err != nil {
            return nil, fmt.Errorf("failed to create provider %s: %w", pc.ID, err)
        }
        
        provider := llm.NewBaseProvider(pc, translator)
        providers = append(providers, provider)
    }
    
    if len(providers) == 0 {
        return nil, errors.NewConfigError("no enabled LLM providers", nil)
    }
    
    // 确定轮询策略
    strategy := llm.RoundRobin
    if s, ok := cfg.Extra["rotation_strategy"].(string); ok {
        strategy = llm.RotationStrategy(s)
    }
    
    // 创建供应商池
    pool := llm.NewProviderPool(providers, strategy)
    
    // 启动健康检查
    ctx := context.Background()
    pool.StartHealthCheck(ctx)
    
    // 创建多供应商翻译器
    maxRetries := cfg.MaxRetries
    if maxRetries == 0 {
        maxRetries = 3
    }
    multiTranslator := llm.NewMultiProviderTranslator(pool, maxRetries)
    
    // 包装为 BatchTranslator
    rateLimit := 10
    if rl, ok := cfg.Extra["rate_limit"].(int); ok && rl > 0 {
        rateLimit = rl
    }
    
    opts := &common.BatchOptions{
        RateLimit:  rateLimit,
        Burst:      5,
        Workers:    5,
        MaxRetries: 0, // 由 multiTranslator 处理重试
    }
    
    return common.NewBatchTranslator(multiTranslator, opts), nil
}
```

### 3.5 配置加载更新

```go
// internal/config/global.go

// LoadGlobalConfig 加载全局配置（更新以支持多供应商）
func LoadGlobalConfig(path string) (*GlobalConfig, error) {
    cfg := DefaultGlobalConfig()
    
    // 加载 TOML 文件
    if _, err := os.Stat(path); err == nil {
        if _, err := toml.DecodeFile(path, cfg); err != nil {
            return nil, fmt.Errorf("failed to decode config: %w", err)
        }
    }
    
    // 向后兼容：如果配置了单供应商但未配置多供应商，自动转换
    if len(cfg.LLM.Providers) == 0 && cfg.LLM.BaseURL != "" {
        cfg.LLM.Providers = []LLMProviderConfig{
            {
                ID:           "default",
                Name:         "Default Provider",
                Enabled:      true,
                Priority:     1,
                Weight:       100,
                BaseURL:      cfg.LLM.BaseURL,
                APIKey:       cfg.LLM.APIKey,
                Model:        cfg.LLM.Model,
                ProxyURL:     cfg.LLM.ProxyURL,
                Timeout:      cfg.LLM.Timeout,
                RateLimit:    cfg.LLM.RateLimit,
                MaxTokens:    cfg.LLM.MaxTokens,
                Temperature:  cfg.LLM.Temperature,
                ExtraHeaders: cfg.LLM.ExtraHeaders,
                ExtraParams:  cfg.LLM.ExtraParams,
            },
        }
    }
    
    // 应用环境变量
    cfg.ApplyEnvVars()
    
    return cfg, nil
}

// ApplyEnvVars 应用环境变量（更新以支持多供应商）
func (c *GlobalConfig) ApplyEnvVars() {
    // 原有环境变量处理...
    
    // 多供应商环境变量支持
    // LLM_PROVIDERS_0_BASE_URL, LLM_PROVIDERS_0_API_KEY, etc.
    // 或者使用 JSON 格式：LLM_PROVIDERS_JSON
    if providersJSON := os.Getenv("LLM_PROVIDERS_JSON"); providersJSON != "" {
        var providers []LLMProviderConfig
        if err := json.Unmarshal([]byte(providersJSON), &providers); err == nil {
            c.LLM.Providers = providers
        }
    }
}
```

## 4. 使用示例

### 4.1 单供应商模式（向后兼容）

```toml
# ~/.config/translator/config.toml
provider = "llm"

[llm]
base_url = "https://api.openai.com/v1"
api_key = "sk-xxx"
model = "gpt-4"
rate_limit = 10
```

### 4.2 多供应商轮询模式

```toml
# ~/.config/translator/config.toml
provider = "llm"

[llm]
rotation_strategy = "round_robin"

[llm.health_check]
enabled = true
interval = 30

[[llm.providers]]
id = "openai-1"
name = "OpenAI Primary"
enabled = true
base_url = "https://api.openai.com/v1"
api_key = "sk-xxx"
model = "gpt-4"
rate_limit = 10

[[llm.providers]]
id = "azure-1"
name = "Azure OpenAI"
enabled = true
base_url = "https://my-resource.openai.azure.com/openai/deployments/gpt-4"
api_key = "azure-key"
model = "gpt-4"
rate_limit = 5
```

### 4.3 加权轮询模式

```toml
[llm]
rotation_strategy = "weighted"

[[llm.providers]]
id = "openai"
name = "OpenAI"
enabled = true
weight = 70
base_url = "https://api.openai.com/v1"
api_key = "sk-xxx"
model = "gpt-4"

[[llm.providers]]
id = "local"
name = "Local LLM"
enabled = true
weight = 30
base_url = "http://localhost:11434/v1"
api_key = ""
model = "llama2"
```

### 4.4 优先级模式（主备）

```toml
[llm]
rotation_strategy = "priority"

[[llm.providers]]
id = "primary"
name = "Primary OpenAI"
enabled = true
priority = 1
base_url = "https://api.openai.com/v1"
api_key = "sk-primary"
model = "gpt-4"

[[llm.providers]]
id = "backup"
name = "Backup Azure"
enabled = true
priority = 2
base_url = "https://backup.openai.azure.com/"
api_key = "azure-key"
model = "gpt-4"
```

## 5. 错误处理

### 5.1 新增错误类型

```go
// internal/translator/llm/errors.go

package llm

import "errors"

var (
    ErrNoAvailableProvider = errors.New("no LLM provider available")
    ErrNoHealthyProvider   = errors.New("no healthy LLM provider available")
    ErrProviderNotFound    = errors.New("LLM provider not found")
    ErrAllProvidersFailed  = errors.New("all LLM providers failed")
)
```

### 5.2 故障转移流程

```
1. 获取下一个供应商（根据轮询策略）
2. 执行翻译请求
3. 如果成功，返回结果
4. 如果失败且可重试：
   a. 标记当前供应商为不健康
   b. 等待退避时间
   c. 获取下一个供应商
   d. 重试步骤 2
5. 如果所有供应商都失败，返回错误
```

## 6. 性能考虑

1. **连接池**: 每个供应商维护独立的 HTTP 连接池
2. **健康检查**: 异步后台检查，不影响翻译请求
3. **并发控制**: 每个供应商独立的限流器
4. **缓存**: 健康状态缓存，避免频繁检查

## 7. 向后兼容性

1. 现有单供应商配置完全兼容
2. 自动将单供应商配置转换为单元素 providers 数组
3. 原有环境变量继续有效
4. 原有 API 行为保持不变

## 8. 实现步骤

1. **Phase 1**: 添加多供应商配置结构（config 包）
2. **Phase 2**: 实现 Provider 接口和 BaseProvider
3. **Phase 3**: 实现 ProviderPool 和轮询策略
4. **Phase 4**: 实现 MultiProviderTranslator
5. **Phase 5**: 更新工厂函数和配置加载
6. **Phase 6**: 添加健康检查机制
7. **Phase 7**: 测试和文档
