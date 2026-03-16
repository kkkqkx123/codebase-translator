# 代码库翻译工具需求文档

## 简介

代码库翻译工具是一个基于 Go 语言开发的命令行应用程序，旨在帮助开发者自动翻译整个代码库中的注释、文档字符串和错误信息等内容。该工具通过集成 DeepLX 翻译 API，提供免费的、高质量的代码翻译服务，支持多种编程语言和自然语言之间的互译。

该工具设计同时支持**单次任务**和**CI/CD集成**场景，特别适用于 AI 编码过程中混用中英文的情况，可在代码提交前自动统一语言。

该工具的核心价值在于：
- **CI/CD 友好**：支持配置文件驱动，易于集成到自动化流程
- **智能增量**：基于文件哈希的缓存机制，仅处理修改过的文件
- **精准提取**：根据配置的语言种类，只提取包含目标语言字符的文本进行翻译
- **配置优先**：目标代码库内的 TOML 配置文件决定主要行为，CLI 参数仅作补充
- **编码统一**：全程使用 UTF-8，自动处理非标准编码转换

---

## Requirement 1: 代码文件扫描与解析

**User Story:** 作为一个开发者，我希望工具能够自动扫描指定目录下的代码文件，以便批量处理整个代码库的翻译工作。

**Acceptance Criteria:**
1.1 The system SHALL support scanning specified directory recursively for code files
1.2 The system SHALL support common programming languages including Go, Python, JavaScript, TypeScript, Java, C++, C#, and Rust
1.3 The system SHALL allow users to specify file extensions to include or exclude via configuration
1.4 The system SHALL provide progress feedback during file scanning
1.5 The system SHALL handle symbolic links and circular references gracefully
1.6 The system SHALL default to scanning current working directory when no path is specified

---

## Requirement 2: 基于语言检测的智能文本提取

**User Story:** 作为一个开发者，我希望工具能够根据配置的语言种类，智能提取代码中包含特定语言字符的段落，以便只翻译需要翻译的内容，避免无关代码增加 API 调用时间。

**Acceptance Criteria:**
2.1 The system SHALL identify single-line comments (e.g., `//` in Go, `#` in Python)
2.2 The system SHALL identify multi-line comments (e.g., `/* */` in Go, `""" """` in Python)
2.3 The system SHALL identify documentation comments (e.g., Go doc comments, JSDoc, JavaDoc)
2.4 The system SHALL identify error message strings in code (e.g., `throw new Error("...")`, `fmt.Errorf("...")`)
2.5 The system SHALL extract text segments containing characters matching the configured source language pattern
2.6 The system SHALL support UTF-8 character range detection for Chinese characters (\u4e00-\u9fff)
2.7 The system SHALL skip English-only segments when source language is configured to Chinese (as English cannot be reliably isolated)
2.8 The system SHALL preserve code structure and syntax during translation
2.9 The system SHALL handle nested comments correctly where applicable

---

## Requirement 3: DeepLX API 集成

**User Story:** 作为一个开发者，我希望工具能够集成 DeepLX 翻译 API，以便免费获取高质量的翻译服务。

**Acceptance Criteria:**
3.1 The system SHALL integrate with embedded DeepLX API.
3.2 The system SHALL support automatic source language detection (AUTO mode)
3.3 The system SHALL allow users to specify source and target languages via configuration file
3.4 The system SHALL support all language codes provided by DeepLX (EN, ZH, JA, KO, DE, FR, ES, etc.)
3.5 The system SHALL handle API errors gracefully with meaningful error messages
3.6 The system SHALL implement rate limiting to respect DeepLX API constraints
3.7 The system SHALL retry failed requests with exponential backoff
3.8 The system SHALL batch multiple text segments into single API calls where possible

---

## Requirement 4: 文件哈希缓存机制

**User Story:** 作为一个开发者，我希望工具能够通过文件哈希缓存已翻译且未修改的文件，以便在重复运行时仅处理需要更新的文件，提高 CI/CD 场景下的执行效率。

**Acceptance Criteria:**
4.1 The system SHALL compute file content hash (e.g., SHA-256) for each processed file
4.2 The system SHALL store file hash-to-translation-result mapping in cache
4.3 The system SHALL skip files whose hash matches the cached hash (unchanged files)
4.4 The system SHALL re-translate files whose hash differs from cache (modified files)
4.5 The system SHALL store cache data in the target codebase directory (e.g., `.translator-cache/`)
4.6 The system SHALL provide `--clear-cache` flag to clear the file hash cache
4.7 The system SHALL handle cache corruption gracefully by re-processing affected files
4.8 The system SHALL update cache entries after successful translation

---

## Requirement 5: TOML 配置文件支持

**User Story:** 作为一个开发者，我希望工具能够使用 TOML 配置文件定义翻译行为，以便将翻译规则与代码库一起版本控制，并在 CI/CD 中保持一致的行为。

**Acceptance Criteria:**
5.1 The system SHALL support TOML format configuration files
5.2 The system SHALL look for configuration file named `translator.toml` in the target codebase root directory
5.3 The system SHALL allow specifying configuration file path via `--config` CLI flag
5.4 The system SHALL merge CLI arguments with configuration file settings (CLI takes precedence over config file)
5.5 The system SHALL validate configuration file and report errors clearly
5.6 The system SHALL provide `--init` flag to generate a sample TOML configuration file
5.7 The system SHALL support the following TOML configuration sections:
    - `[translate]`: source_lang, target_lang
    - `[include]`: patterns for files to include
    - `[exclude]`: patterns for files to exclude
    - `[cache]`: enable/disable, cache directory location

---

## Requirement 6: 命令行界面

**User Story:** 作为一个开发者，我希望工具提供简洁的命令行界面，以便在单次任务中快速执行翻译。

**Acceptance Criteria:**
6.1 The system SHALL accept target codebase path as a positional argument (defaults to current directory)
6.2 The system SHALL provide `--config` flag to specify TOML configuration file path
6.3 The system SHALL provide `--target-lang` flag to override target language from config
6.4 The system SHALL provide `--source-lang` flag to override source language from config
6.5 The system SHALL provide `--dry-run` flag to preview changes without modifying files
6.6 The system SHALL provide `--verbose` flag for detailed logging
6.7 The system SHALL provide `--clear-cache` flag to clear the file hash cache
6.8 The system SHALL display help information with `--help` flag
6.9 The system SHALL display version information with `--version` flag
6.10 The system SHALL provide `--ci` flag for CI/CD mode (stricter error handling, no interactive prompts)

---

## Requirement 7: 编码处理

**User Story:** 作为一个开发者，我希望工具能够统一使用 UTF-8 编码处理文件，以便正确处理各种语言的文本内容。

**Acceptance Criteria:**
7.1 The system SHALL use UTF-8 encoding as the internal processing standard
7.2 The system SHALL detect file encoding before reading
7.3 The system SHALL convert non-UTF-8 encoded files to UTF-8 before processing
7.4 The system SHALL convert files back to original encoding after translation (if different from UTF-8)
7.5 The system SHALL log encoding conversion operations when in verbose mode
7.6 The system SHALL handle encoding detection failures gracefully with clear error messages

---

## Requirement 8: 文件输出与备份

**User Story:** 作为一个开发者，我希望工具能够安全地输出翻译后的文件，以便在需要时可以恢复原始内容。

**Acceptance Criteria:**
8.1 The system SHALL perform in-place modification of source files by default
8.2 The system SHALL preserve original file permissions and timestamps
8.3 The system SHALL maintain original file encoding (converting back from UTF-8 if necessary)
8.4 The system SHALL generate a translation report summarizing changes made
8.5 The system SHALL handle write permission errors gracefully with clear error messages
8.6 The system SHALL support `--dry-run` mode to preview changes without modifying files

---

## Requirement 9: CI/CD 集成支持

**User Story:** 作为一个开发者，我希望工具能够在 CI/CD 环境中稳定运行，以便在代码提交前自动统一代码库语言。

**Acceptance Criteria:**
9.1 The system SHALL provide `--ci` flag for non-interactive mode
9.2 The system SHALL exit with non-zero code on errors in CI mode
9.3 The system SHALL suppress progress spinners and interactive elements in CI mode
9.4 The system SHALL output machine-readable logs in CI mode (JSON format optional)
9.5 The system SHALL fail fast on configuration errors in CI mode
9.6 The system SHALL support environment variables for configuration overrides (e.g., `DEEPLX_TARGET_LANG`)

---

## Requirement 10: 并发处理

**User Story:** 作为一个开发者，我希望工具能够并发处理多个文件和翻译请求，以便提高大型代码库的处理速度。

**Acceptance Criteria:**
10.1 The system SHALL process multiple files concurrently using goroutines
10.2 The system SHALL limit concurrent API requests to avoid rate limiting
10.3 The system SHALL provide `--workers` flag to configure concurrency level
10.4 The system SHALL maintain thread-safe access to shared resources
10.5 The system SHALL gracefully handle concurrent API errors

---

## Requirement 11: 日志与报告

**User Story:** 作为一个开发者，我希望工具能够生成详细的日志和翻译报告，以便了解翻译过程和结果。

**Acceptance Criteria:**
11.1 The system SHALL log translation progress with file names
11.2 The system SHALL generate a summary report including: total files processed, files skipped (cached), files translated, API calls made, errors encountered
11.3 The system SHALL support different log levels (debug, info, warn, error)
11.4 The system SHALL display translation statistics upon completion
11.5 The system SHALL list any files that failed to translate

---

## Requirement 12: 翻译质量控制

**User Story:** 作为一个开发者，我希望工具能够确保翻译质量，以便翻译后的代码注释仍然准确和有用。

**Acceptance Criteria:**
12.1 The system SHALL preserve markdown formatting in documentation comments
12.2 The system SHALL preserve code examples within comments (fenced code blocks)
12.3 The system SHALL preserve special markers like TODO, FIXME, NOTE, etc.
12.4 The system SHALL handle placeholders and format specifiers correctly (e.g., `%s`, `{}`)
12.5 The system SHALL skip translation for segments that appear to be code rather than natural language

---

## 技术架构概述

### 核心模块

1. **Scanner Module**: 负责递归扫描目录，识别支持的代码文件，检测文件编码
2. **Parser Module**: 负责解析代码文件，提取注释、文档字符串和错误信息中包含目标语言字符的段落
3. **Translator Module**: 负责调用 DeepLX API 进行翻译，支持批量请求
4. **Cache Module**: 负责管理基于文件哈希的翻译缓存（存储在目标代码库目录）
5. **Writer Module**: 负责生成翻译后的文件，处理编码转换
6. **Config Module**: 负责解析 TOML 配置文件，合并 CLI 参数
7. **CLI Module**: 负责处理命令行参数和用户交互

### 配置优先级

配置值按以下优先级从高到低应用：
1. 命令行参数（最高优先级）
2. 环境变量
3. TOML 配置文件
4. 默认值（最低优先级）

### 缓存存储位置

文件哈希缓存存储在目标代码库目录下的 `.translator-cache/` 目录中，便于：
- 与代码库一起版本控制（可选）
- CI/CD 环境中复用缓存
- 不同项目使用独立的缓存

### 支持的编程语言

| 语言 | 单行注释 | 多行注释 | 文档注释 | 错误信息示例 |
|------|----------|----------|----------|--------------|
| Go | `//` | `/* */` | `//` (Go doc) | `fmt.Errorf()`, `errors.New()` |
| Python | `#` | `""" """` | `""" """` (Docstring) | `raise Exception()`, `logger.error()` |
| JavaScript/TypeScript | `//` | `/* */` | `/** */` (JSDoc) | `throw new Error()`, `console.error()` |
| Java | `//` | `/* */` | `/** */` (JavaDoc) | `throw new Exception()`, `logger.error()` |
| C/C++ | `//` | `/* */` | `/** */` (Doxygen) | `fprintf(stderr, ...)`, `std::cerr` |
| C# | `//` | `/* */` | `///` (XML Doc) | `throw new Exception()`, `Console.Error` |
| Rust | `//` | `/* */` | `///` or `//!` (Rustdoc) | `panic!()`, `eprintln!()` |

### 示例 TOML 配置

```toml
[translate]
source_lang = "ZH"      # 源语言：中文（只提取包含中文字符的段落）
target_lang = "EN"      # 目标语言：英文

[include]
patterns = ["**/*.go", "**/*.py", "**/*.js", "**/*.ts"]

[exclude]
patterns = ["vendor/**", "node_modules/**", "*.min.js", "*_test.go"]

[cache]
enabled = true
directory = ".translator-cache"
```

### 典型使用场景

**单次任务：**
```bash
translator ./my-project --config ./my-project/translator.toml --dry-run
```

**CI/CD 集成：**
```bash
translator ./my-project --ci --verbose
```

---

## 参考资料

- [DeepLX API 文档](./deeplx.md)
- [DeepLX GitHub](https://github.com/xixu-me/deeplx)
