# Config Module Design

## 概述

Config 模块负责配置管理，包括全局配置、项目配置、环境变量处理和配置哈希计算，为整个翻译系统提供灵活的配置驱动设计。

## 设计目的

1. **配置驱动**：通过配置文件而非代码定义翻译行为，提高灵活性
2. **层次化管理**：区分全局配置和项目配置，实现配置继承和覆盖
3. **环境变量集成**：支持环境变量替换，增强 CI/CD 集成能力
4. **类型安全**：使用 Rust 类型系统确保配置的正确性

## 核心组件

### 1. GlobalConfig

**位置**：`src/config/global.rs`

**职责**：
- 全局翻译服务配置
- 翻译提供商配置（DeepLX, LLM, Tencent）
- 日志配置
- 速率限制配置

**关键配置项**：
```rust
GlobalConfig {
    deeplx: DeepLXConfig,
    llm: LLMGlobalConfig,
    tencent: TencentConfig,
    logging: LoggingConfig,
    limits: RateLimitConfig,
    enabled_providers: Vec<String>,
}
```

**设计要点**：
- 支持多个翻译提供商同时启用
- 提供商配置包含 API 密钥、端点、超时等
- 环境变量支持（${VAR_NAME}）

### 2. ProjectConfig

**位置**：`src/config/project.rs`

**职责**：
- 项目级翻译配置
- 文件过滤规则（include/exclude）
- 提取配置
- 缓存配置
- 写入配置

**关键配置项**：
```rust
ProjectConfig {
    translate: TranslateConfig,
    include: IncludeConfig,
    exclude: ExcludeConfig,
    filter: FilterConfig,
    cache: CacheConfig,
    writer: WriterConfig,
    encoding: EncodingConfig,
    extraction: ExtractionConfig,
}
```

**设计要点**：
- 支持通配符模式匹配文件
- 语言检测和过滤配置
- 提取规则配置（注释、文档字符串、字符串字面量）

### 3. EnvLoader

**位置**：`src/config/env.rs`

**职责**：
- 环境变量加载和替换
- 支持嵌套环境变量引用
- 处理默认值

**关键功能**：
```rust
// 替换环境变量
expand_env_vars("https://${API_HOST}/api") -> "https://api.example.com/api"

// 替换映射中的环境变量
replace_env_vars_in_map(&mut config_map)

// 替换嵌套映射
replace_env_vars_in_nested_map(&mut nested_config)
```

**设计要点**：
- 递归处理嵌套结构
- 支持环境变量未找到时的默认值
- 环境变量格式：${VAR_NAME}

### 4. ConfigLoader

**位置**：`src/config/loader.rs`

**职责**：
- 配置文件加载
- 多格式支持（TOML, YAML, JSON）
- 配置验证和合并

**支持格式**：
- `.toml`：默认格式，人类友好
- `.yaml` / `.yml`：广泛使用
- `.json`：机器友好

**设计要点**：
- 自动检测文件格式
- 配置验证确保有效性
- 支持多个配置文件合并

### 5. Config Hash

**位置**：`src/config/hash.rs`

**职责**：
- 计算配置哈希值
- 用于缓存失效判断
- 仅计算影响翻译结果的配置项

**关键设计**：
```rust
calculate_config_hash(&config) -> String {
    // 只包含影响翻译结果的配置
    // 忽略日志、缓存等不影响结果的配置
}
```

**设计要点**：
- 仅计算关键配置项
- 使用 SHA-256 哈希算法
- 序列化为统一格式后哈希

## 技术选型

### 配置格式
- **TOML**：主要配置格式
  - 人类可读
  - Rust 生态原生支持（serde）
  - 适合复杂嵌套结构

### 序列化库
- **Serde**：序列化/反序列化框架
  - 类型安全
  - 零成本抽象
  - 支持多种格式

### 环境变量
- **Dotenvy**：环境变量加载
  - 支持 .env 文件
  - 类型安全的访问
  - 编译时验证

## 关键设计要点

### 1. 配置层次结构

```
Global Config (全局配置)
  ├── 翻译提供商配置
  ├── 日志配置
  └── 速率限制配置

Project Config (项目配置)
  ├── 翻译配置（源语言、目标语言）
  ├── 文件过滤（include/exclude）
  ├── 提取配置（注释、文档字符串）
  ├── 缓存配置
  └── 写入配置
```

### 2. 配置合并策略

```rust
impl ProjectConfig {
    pub fn merge(&mut self, other: ProjectConfig) {
        // 项目配置覆盖全局配置
        // 非空值才覆盖
    }
}
```

### 3. 环境变量处理

```rust
// 配置文件示例
[deeplx]
api_url = "${DEEPLX_API_URL}"
api_key = "${DEEPLX_API_KEY}"

// 加载后自动替换为实际值
```

**设计原则**：
- 支持嵌套环境变量
- 环境变量未找到时保留原始字符串
- 支持默认值语法：${VAR_NAME:-default}

### 4. 配置验证

```rust
impl LoggingConfig {
    pub fn validate(&self) -> Result<()> {
        if self.output == "file" && self.file.is_none() {
            return Err("File path required for file output");
        }
        Ok(())
    }
}
```

**验证内容**：
- 必填字段检查
- 值范围验证
- 依赖关系检查

### 5. 类型安全配置

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateConfig {
    #[serde(default)]
    pub source_langs: Vec<String>,  // 源语言列表

    #[serde(default = "default_target_lang")]
    pub target_lang: String,  // 目标语言

    #[serde(default)]
    pub concurrency: usize,  // 并发数
}

fn default_target_lang() -> String {
    "en".to_string()
}
```

**优势**：
- 编译时类型检查
- 自动文档生成
- 默认值处理

## 配置示例

### 全局配置 (`.translate/config.toml`)

```toml
[deeplx]
api_url = "https://api-free.deepl.com/v2/translate"
api_key = "${DEEPLX_API_KEY}"
proxy_url = ""
max_retries = 3

[llm]
[[llm.providers]]
id = "openai"
base_url = "https://api.openai.com/v1"
api_keys = ["${OPENAI_API_KEY}"]
model = "gpt-4"
model_list = ["gpt-4", "gpt-3.5-turbo"]

enabled_providers = ["deeplx", "llm"]

[logging]
level = "info"
output = "stdout"
format = "pretty"

[limits]
rate_limit = 60  # requests per minute
split_max_chars = 2000
```

### 项目配置 (`.translate.toml`)

```toml
[translate]
source_langs = ["zh", "ja"]
target_lang = "en"
concurrency = 4
batch_size = 10

[include]
patterns = ["**/*.rs", "**/*.py"]

[exclude]
patterns = ["**/target/**", "**/*.rs.bk"]

[filter]
min_length = 2
max_length = 10000
exclude_keywords = ["TODO", "FIXME"]

[cache]
mode = "binary"
enabled = true
max_size_mb = 100

[writer]
preview_only = false
backup = true
backup_dir = ".translate/backups"

[extraction]
comments = true
doc_strings = true
string_literals = false
```

## 使用示例

```rust
use codebase_translate::config::{ConfigLoader, GlobalConfig, ProjectConfig};

// 加载全局配置
let global_config = ConfigLoader::load_global(".translate/config.toml")?;

// 加载项目配置
let project_config = ConfigLoader::load_project(".translate.toml")?;

// 环境变量已自动替换
let api_key = &global_config.deeplx.api_key;
```

## 扩展性

1. **新的配置格式**：
   - 添加新的配置文件解析器
   - 支持自定义配置格式

2. **配置验证增强**：
   - 更复杂的验证规则
   - 配置依赖关系检查
   - 配置建议和警告

3. **配置热更新**：
   - 监控配置文件变化
   - 动态重新加载配置
   - 配置变更通知

4. **配置管理工具**：
   - 配置生成器
   - 配置验证工具
   - 配置迁移工具