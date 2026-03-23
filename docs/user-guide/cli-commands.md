# CLI Commands Reference

Codebase Translate provides a comprehensive command-line interface for translating code comments and documentation. This document describes all available commands and their options.

## Global Options

These options can be used with any command:

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--config` | `-c` | Path to project configuration file | `.translator.toml` |
| `--global-config` | - | Path to global configuration file | `~/.config/codebase-translate/config.toml` |
| `--log-level` | `-l` | Logging level (trace, debug, info, warn, error) | `info` |
| `--dry-run` | - | Dry run mode - show what would be done without making changes | `false` |

## Commands

### translate

Translate files in a directory. This is the main command for performing translations.

**Usage:**
```bash
translator translate [PATH] [OPTIONS]
```

**Arguments:**

| Argument | Description | Default |
|----------|-------------|---------|
| `PATH` | Directory path to translate | `.` (current directory) |

**Options:**

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--target-lang` | `-t` | Target language for translation | From config |
| `--source-langs` | `-s` | Source languages (comma-separated) | From config |
| `--provider` | `-p` | Translation provider (deeplx, llm, tencent) | From config |
| `--include` | - | Include patterns (comma-separated) | From config |
| `--exclude` | - | Exclude patterns (comma-separated) | From config |

**Examples:**

```bash
# Translate current directory with default settings
translator translate

# Translate specific directory
translator translate ./src

# Translate to English using DeepLX
translator translate ./src --target-lang en --provider deeplx

# Translate only Python files
translator translate ./src --include "*.py"

# Exclude test files
translator translate ./src --exclude "*test*.py"

# Translate multiple source languages to English
translator translate ./src --source-langs "zh,ja,ko" --target-lang en
```

**Output:**

- Translation progress is logged to console
- Detailed translation report is saved to `.translator/report_{timestamp}.txt`
- Cache is automatically updated for translated files

---

### init

Initialize configuration files. Creates default configuration for project or global settings.

**Usage:**
```bash
translator init [OPTIONS]
```

**Options:**

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--global` | - | Initialize global configuration instead of project | `false` |
| `--force` | `-f` | Force overwrite existing configuration | `false` |

**Examples:**

```bash
# Initialize project configuration in current directory
translator init

# Initialize global configuration
translator init --global

# Overwrite existing project configuration
translator init --force

# Overwrite existing global configuration
translator init --global --force
```

**Output:**

- Project config: Creates `.translator.toml` in current directory
- Global config: Creates `~/.config/codebase-translate/config.toml`

---

### cache

Display cache statistics or clear the translation cache.

**Usage:**
```bash
translator cache [OPTIONS]
```

**Options:**

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--clear` | - | Clear all cache entries | `false` |
| `--detailed` | - | Show detailed cache entries | `false` |

**Examples:**

```bash
# Show cache statistics
translator cache

# Show detailed cache entries
translator cache --detailed

# Clear all cache
translator cache --clear
```

**Output:**

```
Cache statistics:
  Total entries: 42
  Total size: 123456 bytes

  Detailed entries:
    - abc123def456: src/main.rs
    - xyz789uvw012: src/lib.rs
```

---

### validate

Validate configuration files to ensure they are correct and complete.

**Usage:**
```bash
translator validate
```

**Examples:**

```bash
# Validate current configuration
translator validate

# Validate with specific config file
translator validate --config .translator.toml
```

**Validation Checks:**

- Target language is not empty
- Translation provider is properly configured
- Required credentials are set for the chosen provider:
  - **DeepLX**: API URL is accessible
  - **LLM**: At least one provider is configured
  - **Tencent**: `secret_id` and `secret_key` are set

**Output:**

```
Validating configuration
Using DeepLX provider at: https://api-free.deepl.com/v2/translate
Configuration is valid
```

---

### verify

Verify extraction rules by scanning files and showing what would be extracted for translation.

**Usage:**
```bash
translator verify [PATH] [OPTIONS]
```

**Arguments:**

| Argument | Description | Default |
|----------|-------------|---------|
| `PATH` | Directory path to verify | `.` (current directory) |

**Options:**

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--pattern` | `-P` | Filter by pattern name | All patterns |
| `--extension` | `-E` | Filter by file extension | All extensions |
| `--category` | `-k` | Filter by category (comment, docstring, error) | All categories |
| `--search` | `-s` | Search for specific text | No filter |
| `--format` | `-F` | Output format (table, json, csv) | `table` |
| `--output` | `-o` | Write results to file | Console output |
| `--detailed` | `-d` | Show detailed match information | `true` |
| `--show-stats` | `-S` | Show statistics summary | `true` |

**Examples:**

```bash
# Verify extraction in current directory
translator verify

# Verify specific directory
translator verify ./src

# Filter by file extension
translator verify ./src --extension rs

# Filter by pattern name
translator verify ./src --pattern "rust_doc_comment"

# Search for specific text
translator verify ./src --search "TODO"

# Output to JSON file
translator verify ./src --format json --output results.json

# Show only matches without statistics
translator verify ./src --show-stats false

# Filter by category
translator verify ./src --category docstring
```

**Output (table format):**

```
File                    | Line | Type      | Pattern              | Content
------------------------|------|-----------|----------------------|------------------------
src/main.rs             | 12   | comment   | rust_line_comment    | This is a comment
src/lib.rs              | 45   | docstring | rust_doc_comment     | Function documentation
src/utils.rs            | 78   | error     | rust_error_macro     | Error message

Statistics:
  Total files scanned: 3
  Total matches found: 15
  Comments: 8
  Docstrings: 5
  Errors: 2
```

---

### clean

Clean cache and backup files to free disk space.

**Usage:**
```bash
translator clean [OPTIONS]
```

**Options:**

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--cache` | - | Clean cache files | `false` |
| `--backup` | - | Clean backup files | `false` |
| `--all` | - | Clean both cache and backup files | `false` |
| `--older-than` | - | Only clean files older than N days | All files |
| `--dry-run` | - | Show what would be deleted without deleting | `false` |
| `--backup-dir` | - | Custom backup directory path | From config |
| `--cache-dir` | - | Custom cache directory path | From config |

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

# Clean with custom directories
translator clean --all --backup-dir ./backups --cache-dir ./cache
```

**Output:**

```
Starting clean operation
  cache: true
  backup: true
  dry_run: false
  older_than_days: 7

Cleaning cache files...
Cache cleared successfully

Cleaning backup files...
Deleted 15 backup files

Clean operation completed
```

---

## Default Behavior

When no command is specified, the tool defaults to translating the current directory:

```bash
translator
```

This is equivalent to:

```bash
translator translate .
```

---

## Common Workflows

### Initial Setup

```bash
# Initialize project configuration
translator init

# Edit configuration if needed
nano .translator.toml

# Validate configuration
translator validate
```

### First Translation

```bash
# Verify what will be extracted
translator verify --format table

# Perform dry run to see changes
translator translate --dry-run

# Execute translation
translator translate
```

### Ongoing Development

```bash
# Translate only modified files (cache is automatic)
translator translate

# Check cache statistics
translator cache --detailed

# Clean old backups periodically
translator clean --backup --older-than 30
```

### Debugging

```bash
# Enable debug logging
translator translate --log-level debug

# Verify extraction rules
translator verify --search "specific text"

# Validate configuration
translator validate
```

---

## Exit Codes

- `0` - Success
- `1` - Error occurred (check error message for details)

---

## Configuration Priority

Configuration is loaded in the following priority order (highest to lowest):

1. Command-line arguments
2. Project configuration file (`.translator.toml`)
3. Global configuration file (`~/.config/codebase-translate/config.toml`)
4. Default values

---

## See Also

- [Configuration Guide](../config/README.md) - Detailed configuration options
- [Translation Providers](../translator/README.md) - Provider-specific settings
- [Cache Management](../cache.md) - Cache internals and best practices