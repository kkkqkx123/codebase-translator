# Codebase Translate

一个用 Rust 开发的命令行工具，可自动翻译代码库中的注释、文档字符串和错误消息。集成了多种翻译 API（DeepLX、LLM、腾讯云），支持多种编程语言，提供智能增量翻译和 CI/CD 集成功能。

## 核心特性

- **CI/CD 友好**: 配置驱动的设计，无缝集成到自动化工作流
- **智能增量处理**: 基于文件哈希的缓存机制，只处理修改过的文件
- **精确提取**: 根据配置的语言设置，提取包含目标语言字符的文本
- **统一编码**: 完全基于 UTF-8 操作，自动转换非标准编码
- **类型安全**: 利用 Rust 类型系统提供编译时正确性保证

## 快速开始

### 安装

```bash
# 从源代码构建
cargo build --release

# 编译后的二进制文件位于 target/release/translator
```

### 初始化配置

```bash
# 初始化全局配置
translator init --global

# 初始化项目配置
cd your-project
translator init
```

### 配置 API 密钥

编辑全局配置目录下的 `.env` 文件：

```env
# DeepLX
DEEPLX_API_URL=https://api.deeplx.org
DEEPLX_API_KEY=your-api-key-here

# LLM Providers
SILCON_API_KEY=xxx
ZHIPU_API_KEY=xxx

# 腾讯云
TENCENT_SECRET_ID=xxx
TENCENT_SECRET_KEY=xxx
```

### 执行翻译

```bash
# 翻译当前目录到英语
translator translate . --target-lang en
```

## 主要命令

### translate

翻译目录中的文件。

```bash
translator translate [PATH] [OPTIONS]
```

**常用选项**:
- `--target-lang <LANG>`: 目标语言代码（如 `en`, `zh`）
- `--source-langs <LANGS>`: 源语言（逗号分隔）
- `--provider <PROVIDER>`: 翻译提供商（`deeplx`, `llm`, `tencent`）
- `--include <PATTERNS>`: 包含模式（逗号分隔的 glob）
- `--exclude <PATTERNS>`: 排除模式（逗号分隔的 glob）

**示例**:
```bash
# 翻译当前目录到英语
translator translate . --target-lang en

# 使用特定翻译器
translator translate . --provider llm

# 只翻译特定文件类型
translator translate . --include "*.rs,*.py"
```

### init

初始化配置文件。

```bash
translator init [OPTIONS]
```

**选项**:
- `--global`: 初始化全局配置而非项目配置
- `--force, -f`: 强制覆盖现有配置

### cache

管理翻译缓存。

```bash
translator cache [OPTIONS]
```

**选项**:
- `--clear`: 清除所有缓存
- `--detailed`: 显示详细缓存条目

### validate

验证配置文件。

```bash
translator validate
```

### verify

验证提取规则，扫描文件并显示将要提取的内容。

```bash
translator verify [PATH] [OPTIONS]
```

**选项**:
- `--pattern, -P <PATTERN>`: 按模式名称过滤
- `--extension, -E <EXT>`: 按文件扩展名过滤
- `--category, -k <CATEGORY>`: 按类别过滤（comment, docstring, error）
- `--search, -s <TEXT>`: 搜索特定文本
- `--format, -F <FORMAT>`: 输出格式（table, json, csv）

### clean

清理缓存和备份文件以释放磁盘空间。

```bash
translator clean [OPTIONS]
```

**选项**:
- `--cache`: 清理缓存文件
- `--backup`: 清理备份文件
- `--all`: 清理缓存和备份文件
- `--older-than <DAYS>`: 只清理超过 N 天的文件

## 配置

### 全局配置文件

全局配置文件（`translator.toml`）包含 API 设置和提供商配置。

**搜索路径（按优先级）**:
1. `--global-config` 命令行选项指定的目录
2. `TRANSLATOR_CONFIG_HOME` 环境变量指定的目录
3. 可执行文件所在目录
4. 当前工作目录
5. 用户配置目录：`~/.config/codebase-translate/`（Linux/Mac）或 `%APPDATA%/codebase-translate/`（Windows）

**配置结构**:

```toml
# 启用的翻译提供商
enabled_providers = ["deeplx", "llm", "tencent"]

# DeepLX 配置
[deeplx]
api_url = "${DEEPLX_API_URL}"
api_key = "${DEEPLX_API_KEY}"
rate_limit = 10
max_retries = 3

# LLM 配置
[llm.health_check]
enabled = true
interval = 30
timeout = 5

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

### 项目配置文件

项目配置文件（`.translator.toml`）包含项目特定的翻译设置。

**配置结构**:

```toml
[translate]
source_langs = ["AUTO"]
target_lang = "en"
batch_size = 50
concurrency = 5

[include]
patterns = [
    "**/*.rs",
    "**/*.py",
    "**/*.js",
]

[exclude]
patterns = [
    "vendor/**",
    "node_modules/**",
]

[filter]
extract_languages = []
exclude_keywords = ["TODO", "FIXME"]
exclude_patterns = ['https?://[^\s]+']
min_length = 2
max_length = 1000
allow_placeholders = true

[cache]
enabled = true
mode = "local"
directory = ".translator"
format = "binary"

[writer]
preview_only = false
backup = true
backup_dir = ""

[extraction]
comments = true
doc_strings = true
string_literals = true
```

### 环境变量

敏感数据（API 密钥等）应存储在同目录的 `.env` 文件中。

**示例 `.env` 文件**:

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

## 翻译提供商

### DeepLX

基于 DeepL 的免费翻译服务。

- **费用**: 完全免费
- **质量**: 高
- **速度**: 快
- **限制**: 最大 5000 字符/次

详细说明: [DeepLX 翻译器](docs/translator/deeplx.md)

### LLM

支持多种大语言模型提供商（OpenAI、Anthropic、SiliconFlow、智谱等），提供智能路由和负载均衡。

- **费用**: 按使用量收费
- **质量**: 可配置（低-高）
- **速度**: 慢
- **限制**: 依赖模型

详细说明: [LLM 翻译器](docs/translator/llm.md)

### 腾讯云

腾讯云机器翻译服务。

- **费用**: 500 万字符/月免费
- **质量**: 高
- **速度**: 快
- **限制**: 最大 6000 字符/次，5 请求/秒

详细说明: [腾讯云翻译器](docs/translator/tencent.md)

## 支持的语言

### 编程语言

- Rust
- Python
- JavaScript/TypeScript
- Go
- Java
- C/C++
- C#
- 更多...

### 翻译语言

- 英语 (EN)
- 中文 (ZH)
- 日语 (JA)
- 韩语 (KO)
- 德语 (DE)
- 法语 (FR)
- 西班牙语 (ES)
- 意大利语 (IT)
- 葡萄牙语 (PT)
- 俄语 (RU)

## 文档

- [安装指南](docs/user-guide/installation.md) - 详细的安装步骤
- [快速开始](docs/user-guide/quick-start.md) - 5 分钟快速入门
- [配置指南](docs/user-guide/configuration.md) - 详细的配置选项
- [工作流指南](docs/user-guide/workflow.md) - 常见工作流程
- [命令参考](docs/user-guide/cli-commands.md) - 所有命令的详细说明
- [翻译器选择指南](docs/translator/provider-selection.md) - 如何选择合适的翻译器
- [DeepLX 翻译器](docs/translator/deeplx.md) - DeepLX 翻译器详情
- [LLM 翻译器](docs/translator/llm.md) - LLM 翻译器详情
- [腾讯云翻译器](docs/translator/tencent.md) - 腾讯云翻译器详情

## 开发

### 质量验证

```bash
# 代码检查
cargo clippy --all-targets --all-features

# 代码格式化
cargo fmt
```

### 构建

```bash
# 构建发布版本
cargo build --release
```

### 测试

```bash
# 运行所有测试
cargo test --all

# 运行库测试
cargo test --lib -- --nocapture

# 运行特定测试
cargo test <test_name>

# 运行集成测试
cargo test --test <integration_test_file>
```

## 最佳实践

1. **版本控制**: 翻译前提交所有更改，使用 Git 追踪翻译结果
2. **测试验证**: 翻译后运行测试确保代码没有破坏
3. **预演模式**: 使用 `--dry-run` 预览翻译结果
4. **启用缓存**: 使用缓存避免重复翻译
5. **备份文件**: 启用备份功能以便回滚
6. **配置验证**: 使用 `translator validate` 验证配置
7. **日志监控**: 使用 `--log-level debug` 调试问题

## 故障排查

### 常见问题

1. **翻译失败**
   - 检查 API 密钥是否正确
   - 检查网络连接
   - 查看详细日志: `translator translate --log-level debug`

2. **翻译质量不佳**
   - 尝试不同的翻译器
   - 调整过滤规则
   - 使用语言专用提取

3. **性能问题**
   - 减少并发数: `--concurrency 2`
   - 减少批量大小: `--batch-size 20`
   - 查看缓存统计: `translator cache --detailed`

## 示例

### 翻译特定语言

```bash
# 将中文翻译为英语
translator translate . --source-langs zh --target-lang en

# 将多种语言翻译为英语
translator translate . --source-langs "zh,ja,ko" --target-lang en
```

### 使用语言专用提取

```toml
# .translator.toml
[filter]
extract_languages = ["ZH"]
```

```bash
# 只翻译包含中文的文本
translator translate .
```

### CI/CD 集成

```yaml
name: Translate Codebase

on:
  push:
    branches: [ main ]

jobs:
  translate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Codebase Translate
        run: cargo install codebase-translate

      - name: Translate
        env:
          DEEPLX_API_KEY: ${{ secrets.DEEPLX_API_KEY }}
        run: |
          translator translate . --target-lang en --log-level error
```

## 贡献

欢迎贡献！请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 致谢

- [DeepL](https://www.deepl.com/) - 翻译服务
- [OpenAI](https://openai.com/) - LLM API
- [腾讯云](https://cloud.tencent.com/) - 机器翻译服务
- [tree-sitter](https://tree-sitter.github.io/) - 代码解析

## 联系方式

- 问题反馈: [GitHub Issues](https://github.com/your-org/codebase-translate/issues)
- 功能建议: [GitHub Discussions](https://github.com/your-org/codebase-translate/discussions)

---

注意：使用 DeepLX 可能存在法律风险，请自行评估并遵守相关法律法规。建议在生产环境中使用官方的 DeepL API 或其他合法的翻译服务。