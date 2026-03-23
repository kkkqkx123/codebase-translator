# Codebase Translate

A command-line tool developed in Rust that automatically translates comments, documentation strings, and error messages within codebases. It integrates multiple translation APIs (DeepLX, LLM, Tencent Cloud), supports various programming languages, and offers intelligent incremental translation alongside CI/CD integration capabilities.

## Features

- **CI/CD-Friendly**: Configuration-driven design for seamless automation workflow integration
- **Intelligent Incremental Processing**: File-hash-based caching mechanism processing only modified files
- **Precise Extraction**: Extracts text containing target language characters based on configured language settings
- **Unified Encoding**: Fully UTF-8-based operation with automatic non-standard encoding conversion
- **Type Safety**: Leverages Rust's type system for compile-time correctness guarantees

## Installation

```bash
cargo build --release
```

The compiled binary will be available at `target/release/translator`.

## Quick Start

1. Initialize configuration:
```bash
translator init
```

2. Set up your API keys in `.env` file (copy from `.env.example`):
```bash
cp .env.example .env
```

3. Translate your codebase:
```bash
translator translate . --target-lang en
```

## Commands

### Translate

Translate files in a directory.

```bash
translator translate [PATH] [OPTIONS]
```

**Options:**
- `--target-lang <LANG>`: Target language code (e.g., `en`, `zh`)
- `--source-langs <LANGS>`: Source languages (comma-separated)
- `--provider <PROVIDER>`: Translation provider (`deeplx`, `llm`, `tencent`)
- `--include <PATTERNS>`: Include patterns (comma-separated globs)
- `--exclude <PATTERNS>`: Exclude patterns (comma-separated globs)

**Examples:**
```bash
# Translate current directory to English
translator translate . --target-lang en

# Translate specific directory with custom patterns
translator translate ./src --target-lang en --include "*.rs,*.py" --exclude "test_*"

# Use specific translation provider
translator translate . --target-lang en --provider llm
```

### Init

Initialize configuration files.

```bash
translator init [OPTIONS]
```

**Options:**
- `--global`: Initialize global config instead of project config
- `--force, -f`: Force overwrite existing config

**Examples:**
```bash
# Initialize project config in current directory
translator init

# Initialize global config
translator init --global

# Force overwrite existing config
translator init --force
```

### Cache

Manage translation cache.

```bash
translator cache [OPTIONS]
```

**Options:**
- `--clear`: Clear all cache
- `--detailed`: Show detailed cache entries

**Examples:**
```bash
# Show cache statistics
translator cache

# Show detailed cache information
translator cache --detailed

# Clear all cache
translator cache --clear
```

### Validate

Validate configuration files.

```bash
translator validate
```

### Verify

Verify extraction rules by scanning files and showing what would be extracted for translation.

```bash
translator verify [PATH] [OPTIONS]
```

**Options:**
- `--pattern, -P <PATTERN>`: Filter by pattern name
- `--extension, -E <EXT>`: Filter by file extension
- `--category, -k <CATEGORY>`: Filter by category (comment, docstring, error)
- `--search, -s <TEXT>`: Search for specific text
- `--format, -F <FORMAT>`: Output format (table, json, csv) - default: table
- `--output, -o <FILE>`: Write results to file
- `--detailed, -d`: Show detailed match information - default: true
- `--show-stats, -S`: Show statistics summary - default: true

**Examples:**
```bash
# Verify extraction in current directory
translator verify

# Verify specific directory
translator verify ./src

# Filter by file extension
translator verify ./src --extension rs

# Search for specific text
translator verify ./src --search "TODO"

# Output to JSON file
translator verify ./src --format json --output results.json
```

### Clean

Clean cache and backup files to free disk space.

```bash
translator clean [OPTIONS]
```

**Options:**
- `--cache`: Clean cache files
- `--backup`: Clean backup files
- `--all`: Clean both cache and backup files
- `--older-than <DAYS>`: Only clean files older than N days
- `--dry-run`: Show what would be deleted without deleting
- `--backup-dir <DIR>`: Custom backup directory path
- `--cache-dir <DIR>`: Custom cache directory path

**Examples:**
```bash
# Clean cache files
translator clean --cache

# Clean backup files
translator clean --backup

# Clean both cache and backup files
translator clean --all

# Clean cache files older than 7 days
translator clean --cache --older-than 7

# Dry run to see what would be deleted
translator clean --all --dry-run
```

## Configuration

### Global Configuration File

The global configuration file (`config.toml` or `translator.toml`) contains API settings and provider configurations.

**Search paths (in priority order):**
1. Directory specified by `--global-config` command-line option
2. Directory specified by `TRANSLATOR_CONFIG_HOME` environment variable
3. Executable directory
4. Current working directory
5. User config directory: `~/.config/codebase-translate/` (Linux/Mac) or `%APPDATA%/codebase-translate/` (Windows)

**Configuration structure:**

```toml
# Enabled translation providers
enabled_providers = ["deeplx", "llm", "tencent"]

# DeepLX configuration
[deeplx]
api_url = "${DEEPLX_API_URL}"
api_key = "${DEEPLX_API_KEY}"
proxy_url = "${DEEPLX_PROXY_URL}"
rate_limit = 5
max_retries = 3

# LLM configuration
[llm.health_check]
enabled = true
interval = 30
timeout = 5
failure_threshold = 3
recovery_interval = 300

[[llm.providers]]
id = "silicon"
name = "Siliconflow"
models = ["tencent/Hunyuan-MT-7B", "THUDM/GLM-4-9B-0414", "Qwen/Qwen2.5-7B-Instruct"]
max_tokens = 4096
temperature = 0.3
weight = 50
base_url = "https://api.siliconflow.cn/v1"
api_keys = ["${SILCON_API_KEY}"]
proxy_url = ""
timeout = 20
rate_limit = 40

[[llm.providers]]
id = "zhipu"
name = "Zhipu"
models = ["glm-4-flash"]
max_tokens = 4096
temperature = 0.3
weight = 20
base_url = "https://open.bigmodel.cn/api/paas/v4"
api_keys = ["${ZHIPU_API_KEY}"]
proxy_url = ""
timeout = 60
rate_limit = 5

# Logging configuration
[logging]
level = "info"
format = "console"
output = "stdout"
file = "translator.log"

# Tencent Cloud configuration
[tencent]
secret_id = "${TENCENT_SECRET_ID}"
secret_key = "${TENCENT_SECRET_KEY}"
region = "ap-guangzhou"
project_id = 0
proxy_url = ""
timeout = 30
rate_limit = 5
max_retries = 3
```

### Project Configuration File

The project configuration file (`.translator.toml`) contains project-specific translation settings.

**Configuration structure:**

```toml
[translate]
source_langs = ["AUTO"]
target_lang = "en"
provider = "deeplx"
batch_size = 50
concurrency = 4

[include]
patterns = ["*.rs", "*.py", "*.js", "*.ts", "*.go", "*.java", "*.c", "*.cpp"]

[exclude]
patterns = ["target/*", "node_modules/*", "*.min.js"]

[filter]
exclude_keywords = ["TODO", "FIXME", "HACK"]
exclude_patterns = []
include_patterns = []
min_length = 2
max_length = 1000
allow_placeholders = true
detect_code_patterns = true

[cache]
cache_dir = ".translator/cache"
cache_type = "binary"
enabled = true

[writer]
dry_run = false
backup = true
backup_dir = ".translator/backup"

[encoding]
detect_encodings = ["utf-8", "gbk", "gb2312", "big5"]
min_confidence = 0.7
convert_to_utf8 = true

[extraction]
comments = true
doc_strings = true
error_messages = true
format_strings = false
custom_patterns = []
```

### Environment Variables

Sensitive data (API keys, secrets) should be stored in a `.env` file in the same directory as the configuration file.

**Example `.env` file:**

```env
# DeepLX
DEEPLX_API_URL=https://api.deeplx.org
DEEPLX_API_KEY=your-api-key-here

# LLM Providers
SILCON_API_KEY=xxx
ZHIPU_API_KEY=xxx
OPENAI_API_KEY=sk-xxx

# Tencent Cloud
TENCENT_SECRET_ID=xxx
TENCENT_SECRET_KEY=xxx
```

## Translation Providers

### DeepLX

Free translation service based on DeepL. No authentication required for local instances.

### LLM

Supports multiple large language model providers (OpenAI, Anthropic, SiliconFlow, Zhipu, etc.) with intelligent routing and load balancing.

### Tencent Cloud

Tencent Cloud Machine Translation Service. 5 million characters free per month.

## Supported Languages

- Rust
- Python
- JavaScript/TypeScript
- Go
- Java
- C/C++
- C#
- And more...

## Development

### Quality Verify

```bash
cargo clippy --all-targets --all-features
cargo fmt --check
```

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test --all
```

## License

MIT
