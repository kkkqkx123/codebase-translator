# 配置指南

Codebase Translate 使用两个配置文件：全局配置文件和项目配置文件。

## 配置文件位置

### 全局配置文件 (translator.toml)

全局配置文件存储 API 密钥和翻译器提供者设置。按以下优先级搜索：

1. `--global-config` 命令行选项指定的目录
2. `TRANSLATOR_CONFIG_HOME` 环境变量指定的目录
3. 可执行文件所在目录
4. 当前工作目录
5. 用户配置目录：
   - Linux/Mac: `~/.config/codebase-translate/`
   - Windows: `%APPDATA%/codebase-translate/`

### 项目配置文件 (.translator.toml)

项目配置文件存储项目特定的翻译设置，应放置在要翻译的代码库根目录。

## 配置文件结构

### 全局配置文件 (translator.toml)

```toml
# 启用的翻译器列表
enabled_providers = ["deeplx", "llm", "tencent"]

# DeepLX 配置
[deeplx]
api_url = "${DEEPLX_API_URL}"
api_key = "${DEEPLX_API_KEY}"
proxy_url = "${DEEPLX_PROXY_URL}"
rate_limit = 10
max_retries = 3

# LLM 配置
[llm.health_check]
enabled = true
interval = 30
timeout = 5
failure_threshold = 3
recovery_interval = 60

[[llm.providers]]
id = "silicon"
name = "Siliconflow"
model_list = ["tencent/Hunyuan-MT-7B", "THUDM/GLM-4-9B-0414"]
max_tokens = 8192
temperature = 0.3
base_url = "https://api.siliconflow.cn/v1"
api_keys = ["${SILCON_API_KEY}"]
rate_limit = 40

# 腾讯云配置
[tencent]
secret_id = "${TENCENT_SECRET_ID}"
secret_key = "${TENCENT_SECRET_KEY}"
region = "ap-guangzhou"
rate_limit = 5
max_retries = 3

# 日志配置
[logging]
level = "info"
format = "pretty"
output = "file"
file = "logs/translator.log"
```

### 项目配置文件 (.translator.toml)

```toml
# 翻译语言配置
[translate]
source_langs = ["AUTO"]
target_lang = "en"
batch_size = 50
concurrency = 5

# 包含文件模式
[include]
patterns = [
    "**/*.rs",
    "**/*.py",
    "**/*.js",
    "**/*.ts",
]

# 排除文件模式
[exclude]
patterns = [
    "vendor/**",
    "node_modules/**",
    "*.min.js",
]
respect_gitignore = true

# 过滤配置
[filter]
extract_languages = []
exclude_keywords = ["TODO", "FIXME", "HACK"]
exclude_patterns = [
    'https?://[^\s]+',
    '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}',
]
min_length = 2
max_length = 1000
allow_placeholders = true
detect_code_patterns = true

# 缓存配置
[cache]
enabled = true
mode = "local"
directory = ".translator"
format = "binary"

# 写入配置
[writer]
preview_only = false
backup = true
backup_dir = ""
max_concurrent_writes = 10
preserve_formatting = true

# 提取配置
[extraction]
comments = true
doc_strings = true
string_literals = true

# 自定义提取模式
[[extraction.custom_patterns]]
name = "todo_pattern"
file_extensions = ["js", "ts", "py", "rs"]
category = "other"
regex = 'TODO:\s*(.+)'
group = 1

# 编码检测配置
[encoding]
detect_encodings = ["UTF-8"]
min_confidence = 0.7
convert_to_utf8 = true
```

## 配置说明

### 翻译器配置

#### DeepLX

- `api_url`: API 地址，默认为 `https://api.deeplx.org`
- `api_key`: API 密钥（可选）
- `proxy_url`: 代理地址（可选）
- `rate_limit`: 每秒请求数限制
- `max_retries`: 最大重试次数

#### LLM

- `health_check.enabled`: 是否启用健康检查
- `health_check.interval`: 健康检查间隔（秒）
- `health_check.failure_threshold`: 连续失败次数标记为不可用
- `health_check.recovery_interval`: 恢复检查间隔（秒）

每个 LLM 提供者配置：
- `id`: 提供者唯一标识
- `name`: 提供者名称
- `model_list`: 模型列表（多模型轮询）
- `model`: 单一模型名称（仅在 model_list 为空时生效）
- `max_tokens`: 最大 token 数
- `temperature`: 温度参数 (0.0 - 2.0)
- `base_url`: API 基础 URL
- `api_keys`: API 密钥列表
- `rate_limit`: 每秒请求数限制

#### 腾讯云

- `secret_id`: 腾讯云 Secret ID
- `secret_key`: 腾讯云 Secret Key
- `region`: 区域，默认 `ap-guangzhou`
- `project_id`: 项目 ID
- `endpoint`: API 端点
- `rate_limit`: 速率限制（腾讯云要求不超过 5 次/秒）
- `max_retries`: 最大重试次数

### 翻译配置

- `source_langs`: 源语言列表，AUTO 表示自动检测
- `target_lang`: 目标语言代码
- `batch_size`: 批量翻译大小
- `concurrency`: 并发请求数

### 文件过滤配置

#### 包含模式

- `patterns`: Glob 模式列表，指定需要翻译的文件

#### 排除模式

- `patterns`: Glob 模式列表，指定不需要翻译的文件或目录
- `respect_gitignore`: 是否遵循 .gitignore 文件
- `gitignore_patterns`: 额外的 gitignore 风格模式

### 文本过滤配置

- `extract_languages`: 语言专用提取模式，提取包含指定语言字符的文本
- `exclude_keywords`: 排除关键词列表
- `exclude_patterns`: 排除正则表达式列表
- `include_patterns`: 包含正则表达式列表（优先级高于排除）
- `min_length`: 最小文本长度
- `max_length`: 最大文本长度
- `allow_placeholders`: 是否允许包含占位符的文本
- `detect_code_patterns`: 是否检测并过滤代码模式

### 缓存配置

- `enabled`: 是否启用缓存
- `mode`: 缓存模式（local 或 global）
- `directory`: 缓存目录名
- `format`: 缓存格式（json 或 binary）

### 写入配置

- `preview_only`: 是否仅预览，不实际修改文件
- `backup`: 是否在修改前创建备份文件
- `backup_dir`: 备份目录
- `max_concurrent_writes`: 最大并发写入数
- `preserve_formatting`: 保留原始格式

### 提取配置

- `comments`: 是否提取注释
- `doc_strings`: 是否提取文档字符串
- `string_literals`: 是否提取字符串字面量

### 自定义提取模式

- `name`: 模式名称
- `file_extensions`: 适用的文件扩展名列表
- `category`: 类别（error_handling, output, variables, properties, other）
- `regex`: 正则表达式
- `group`: 捕获组索引

### 编码检测配置

- `detect_encodings`: 要检测的编码列表
- `min_confidence`: 最小置信度阈值 (0-1)
- `convert_to_utf8`: 是否自动转换为 UTF-8

## 环境变量

敏感信息（API 密钥等）应存储在 `.env` 文件中，位于配置文件同目录：

```env
# DeepLX
DEEPLX_API_URL=https://api.deeplx.org
DEEPLX_API_KEY=your-api-key-here

# LLM Providers
SILCON_API_KEY=xxx
ZHIPU_API_KEY=xxx
OPENAI_API_KEY=sk-xxx

# 腾讯云
TENCENT_SECRET_ID=xxx
TENCENT_SECRET_KEY=xxx
```

## 配置优先级

配置按以下优先级加载（从高到低）：

1. 命令行参数
2. 项目配置文件 (`.translator.toml`)
3. 全局配置文件 (`translator.toml`)
4. 默认值

## 配置验证

使用以下命令验证配置：

```bash
translator validate
```

## 最佳实践

1. **安全**: 不要在配置文件中直接写入 API 密钥，使用环境变量占位符
2. **性能**: 根据项目大小调整 `batch_size` 和 `concurrency`
3. **准确性**: 使用 `extract_languages` 精确提取目标语言文本
4. **可靠性**: 启用缓存和备份功能
5. **调试**: 在开发时启用 debug 日志级别