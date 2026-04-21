---
name: "codebase-translate-executor"
description: "Guide LLM to correctly use Codebase Translate build artifacts and configuration files for translation tasks. Invoke when user needs to execute code translation, verify configurations, clean cache, or initialize project configuration."
---

# Codebase Translate Executor Guide

## Project Overview

Codebase Translate is a command-line tool developed in Rust, designed to automatically translate comments, docstrings, and error messages within codebases. It supports multiple translation APIs (DeepLX, LLM, Tencent Cloud), offers intelligent incremental translation, and provides CI/CD integration capabilities.

## Build Artifacts

### Executable

The compiled executable file is named `translator`:

```bash
# Development build
cargo build
# Output path: target/debug/translator.exe

# Release build (Recommended)
cargo build --release
# Output path: target/release/translator.exe
```

### Using the Build Artifact

```bash
# Use the artifact directly
./target/release/translator --help

# Or add it to your PATH and use it
translator --help
```

## Configuration Files

### Configuration Check API

**CRITICAL**: Before creating any configuration, ALWAYS check if configuration already exists to avoid conflicts and environment management chaos.

```bash
# Check if global configuration exists
test -f ~/.config/codebase-translate/config.toml && echo "Global config exists" || echo "Global config not found"

# Check if project configuration exists
test -f .translator.toml && echo "Project config exists" || echo "Project config not found"

# Or use the tool's validate command to check
translator validate
```

**Rules**:

1. If global config exists → Do NOT create new global config, guide user to edit existing one
2. If project config exists → Do NOT create new project config, guide user to edit existing one
3. Only create configs when they don't exist

### Project Configuration File (.translator.toml)

Place this file in the **root directory of the target codebase** (not the root directory of the Codebase Translate tool itself):

```toml
# Translation Language Configuration
[translate]
source_langs = ["AUTO"]       # Source language; AUTO means auto-detect
target_lang = "EN"            # Target language code
batch_size = 50               # Batch translation size
concurrency = 5               # Number of concurrent requests

# Include File Patterns
[include]
patterns = [
    "**/*.rs",
    "**/*.py",
    "**/*.js",
    "**/*.ts",
    "**/*.go",
    # ... other extensions
]

# Exclude File Patterns
[exclude]
patterns = [
    "vendor/**",
    "node_modules/**",
    "third_party/**",
    ".git/**",
    "dist/**",
    "build/**",
]
respect_gitignore = true

# Filter Configuration
[filter]
# Extract text containing only specified language characters
extract_languages = []  # Empty list uses standard filtering; ["ZH"] extracts only Chinese

# Keywords to exclude
exclude_keywords = ["TODO", "FIXME", "NOTE", "XXX", "HACK", "Copyright", "License"]

# Regex patterns to exclude
exclude_patterns = [
    'https?://[^\s]+',           # URLs
    '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}',  # Email addresses
]

# Cache Configuration
[cache]
enabled = true
mode = "local"                # local (project directory) or global (global directory)
directory = ".translator"     # Cache directory name
format = "binary"             # json or binary

# Writer Configuration
[writer]
preview_only = false          # Preview mode only; do not modify files
backup = true                 # Create backups
max_concurrent_writes = 10
preserve_formatting = true

# Extraction Configuration
[extraction]
comments = true               # Extract comments
doc_strings = true            # Extract docstrings
string_literals = true        # Extract string literals

# Encoding Detection
[encoding]
detect_encodings = ["UTF-8"]
min_confidence = 0.7
convert_to_utf8 = true
```

### Global Configuration File

Location: `~/.config/codebase-translate/config.toml`

**IMPORTANT**: API keys and credentials MUST be provided by the user. LLM should only provide example templates, NEVER include actual credentials.

```toml
# ============================================
# Global Configuration Template
# ============================================
# Copy this template and fill in your own credentials
# DO NOT commit this file with real credentials to version control

# Default translation provider
# Options: "deeplx", "llm", "tencent"
provider = "deeplx"

# Enabled providers list
enabled_providers = ["deeplx"]

# --------------------------------------------
# DeepLX Configuration
# --------------------------------------------
[deeplx]
# Required: DeepLX API endpoint URL
api_url = "https://your-deeplx-instance.com/translate"
# Optional: API key if your instance requires authentication
api_key = "your-api-key-here"
# Rate limit (requests per second)
rate_limit = 5
# Max retries on failure
max_retries = 3

# --------------------------------------------
# Tencent Cloud Configuration
# --------------------------------------------
[tencent]
# Required: Tencent Cloud Secret ID
secret_id = "your-secret-id-here"
# Required: Tencent Cloud Secret Key
secret_key = "your-secret-key-here"
# Region for Tencent Cloud API
region = "ap-beijing"
# Rate limit (requests per second)
rate_limit = 5
# Max retries on failure
max_retries = 3

# --------------------------------------------
# LLM Configuration
# --------------------------------------------
[llm]

[[llm.providers]]
# Provider identifier
id = "openai"
# Base URL for the API
base_url = "https://api.openai.com/v1"
# API keys (can specify multiple for rotation)
api_keys = ["your-api-key-here"]
# Model name
model = "gpt-4o-mini"
# Max tokens per request
max_tokens = 4096
# Rate limit (requests per second)
rate_limit = 10
# Timeout in seconds
timeout = 60
# Temperature for generation
temperature = 0.3

# Example: Additional LLM provider (optional)
[[llm.providers]]
id = "silicon"
base_url = "https://api.siliconflow.cn/v1"
api_keys = ["your-silicon-api-key"]
model = "Qwen/Qwen2.5-7B-Instruct"
max_tokens = 4096
rate_limit = 20
timeout = 60
temperature = 0.3

# --------------------------------------------
# Logging Configuration
# --------------------------------------------
[logging]
# Log level: trace, debug, info, warn, error
level = "info"
# Output: stderr, stdout, file
output = "stderr"
# Format: compact, pretty, json
format = "compact"
# Log file path (required if output = "file")
# file = "/path/to/translator.log"

# --------------------------------------------
# Rate Limiting Configuration
# --------------------------------------------
[limits]
# Global rate limit across all providers
global_rate_limit = 100
# Burst size for rate limiting
burst_size = 10
```

#### Environment Variable Support

All configuration values can be overridden via environment variables:

```bash
# Provider selection
export TRANSLATOR_PROVIDER=deeplx

# DeepLX
export DEEPLX_API_URL=https://api.example.com/translate
export DEEPLX_API_KEY=your-key

# Tencent
export TENCENT_SECRET_ID=your-id
export TENCENT_SECRET_KEY=your-key

# LLM (format: TRANSLATOR_LLM_<PROVIDER_ID>_<KEY>)
export TRANSLATOR_LLM_OPENAI_API_KEY=your-key
export TRANSLATOR_LLM_OPENAI_BASE_URL=https://api.openai.com/v1
export TRANSLATOR_LLM_OPENAI_MODEL=gpt-4o-mini

# Logging
export TRANSLATOR_LOG_LEVEL=debug
export TRANSLATOR_LOG_OUTPUT=stderr
```

## Core Commands

### 1. Initialize Configuration

```bash
# Initialize project configuration (Execute in the root directory of the target codebase)
translator init

# Initialize global configuration
translator init --global

# Force overwrite existing configuration (USE WITH CAUTION)
translator init --force
```

### 2. Validate Configuration

```bash
# Validate current configuration
translator validate

# Validate specific configuration file
translator validate --config /path/to/.translator.toml
```

### 3. Verify Extraction Rules

```bash
# Verify what will be extracted in the current directory
translator verify

# Verify specific directory
translator verify ./src

# Filter by extension
translator verify ./src --extension rs

# Search for specific text
translator verify ./src --search "TODO"

# Output in JSON format
translator verify ./src --format json --output results.json
```

### 4. Execute Translation

```bash
# Translate current directory
translator translate

# Translate specific directory
translator translate ./src

# Specify target language and provider
translator translate ./src --target-lang en --provider deeplx

# Include only specific files
translator translate ./src --include "*.py"

# Exclude specific files
translator translate ./src --exclude "*test*.py"

# Dry-run mode (preview without modifying)
translator translate --dry-run

# Debug mode
translator translate --log-level debug
```

### 5. Cache Management

```bash
# View cache statistics
translator cache

# View detailed cache info
translator cache --detailed

# Clear cache
translator cache --clear
```

### 6. Cleanup Files

```bash
# Clean cache files
translator clean --cache

# Clean backup files
translator clean --backup

# Clean everything
translator clean --all

# Clean files older than 7 days
translator clean --all --older-than 7

# Dry run
translator clean --all --dry-run
```

## Typical Workflows

### First-time Translation of a Codebase

```bash
# 1. Navigate to the target codebase
cd /path/to/target-project

# 2. Check if configuration already exists
if test -f .translator.toml; then
    echo "Project config exists, will use existing config"
else
    # Initialize project configuration
    translator init
fi

# 3. Edit configuration (adjust as needed)
# nano .translator.toml

# 4. Validate configuration
translator validate

# 5. Verify extraction content
translator verify --format table

# 6. Dry-run preview
translator translate --dry-run

# 7. Execute translation
translator translate
```

### Daily Incremental Translation

```bash
# Direct translation (cache handles increments automatically)
translator translate

# Check cache status
translator cache --detailed

# Periodically clean old backups
translator clean --backup --older-than 30
```

## Output Files

### Report Files

After translation completes, reports are generated in the `.translator/` directory:

```
.translator/
├── cache/                    # Cache data
├── report_20250421_102930.txt  # Translation report
└── backup/                   # Backup files (if enabled)
```

### Sample Report Content

```
Translation Report
==================
Start Time: 2025-04-21 10:29:30
End Time: 2025-04-21 10:30:45
Duration: 75 seconds

Files Processed: 42
Files Modified: 15
Files Skipped (cache): 27

Translation Statistics:
  Total texts: 156
  Translated: 42
  Cached: 114

Provider: DeepLX
Source Languages: AUTO
Target Language: EN
```

## Common Issues

### 1. Configuration File Not Found

Ensure you are running commands from the **root directory of the target codebase**, or specify the configuration file path:

```bash
translator translate /path/to/project --config /path/to/project/.translator.toml
```

### 2. Translation Provider Configuration Error

Run the validation command to check the configuration:

```bash
translator validate
```

### 3. Cache Not Working

Check if caching is enabled in the configuration:

```toml
[cache]
enabled = true
mode = "local"
```

### 4. Files Not Being Translated

Use the `verify` command to check extraction rules:

```bash
translator verify --detailed
```

## Best Practices

1. **Before first use**: Always run `translator verify` to check extraction content.
2. **Check existing configs**: Use Configuration Check API before creating new configs.
3. **Production environment**: Use `--dry-run` to preview changes.
4. **CI/CD Integration**: Drive operations via configuration files rather than command-line arguments.
5. **Backup Strategy**: Enable `writer.backup = true`.
6. **Cache Management**: Regularly clean up expired caches and backups.
7. **API Security**: Never commit real API keys to version control; use environment variables.
