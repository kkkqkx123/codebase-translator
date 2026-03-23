# LLM Translator Requirements Specification

## Introduction

This document specifies the requirements for integrating LLM (Large Language Model) based translation capabilities into the Codebase Translate tool. The LLM translator provides an alternative to the existing DeepLX translator, enabling users to leverage OpenAI-compatible APIs (such as OpenAI, Azure OpenAI, or self-hosted models) for translating code comments, documentation strings, and error messages.

The LLM translator is designed to work with any API endpoint that follows the OpenAI chat completions format, providing flexibility for users who prefer using their own API keys or self-hosted language models.

---

## Requirement 1: OpenAI-Compatible API Integration

**User Story:** As a user, I want to use OpenAI-compatible LLM APIs for translation, so that I can leverage high-quality language models for more accurate and context-aware translations of technical content.

**Acceptance Criteria:**

1.1 The system SHALL support non-streaming chat completion API calls to OpenAI-compatible endpoints

1.2 The system SHALL use the standard `/chat/completions` endpoint path

1.3 The system SHALL send requests in JSON format following the OpenAI API specification

1.4 The system SHALL handle API responses in JSON format and extract translated text from the `choices[0].message.content` field

1.5 The system SHALL support HTTP POST requests with proper Content-Type headers

1.6 The system SHALL implement proper error handling for HTTP error codes (4xx, 5xx)

1.7 The system SHALL support configurable base URLs to accommodate different providers (OpenAI, Azure, self-hosted)

---

## Requirement 2: Global Configuration for LLM Settings

**User Story:** As a user, I want to configure LLM API settings globally, so that I can store sensitive information like API keys securely and separately from project-specific translation settings.

**Acceptance Criteria:**

2.1 The system SHALL support a global configuration file stored in the user's config directory (e.g., `~/.config/translator/config.toml` or `%APPDATA%/translator/config.toml`)

2.2 The global configuration SHALL include the following LLM-specific settings:

2.2.1 The system SHALL support `base_url` configuration for specifying the API endpoint base URL

2.2.2 The system SHALL support `api_key` configuration for authentication

2.2.3 The system SHALL support `model` configuration for specifying the model name (e.g., "gpt-3.5-turbo", "gpt-4")

2.2.4 The system SHALL support `proxy_url` configuration for HTTP proxy settings

2.2.5 The system SHALL support `timeout` configuration for request timeout in seconds

2.2.6 The system SHALL support `rate_limit` configuration for requests per second limiting

2.2.7 The system SHALL support `max_tokens` configuration for maximum output tokens per request

2.2.8 The system SHALL support `temperature` configuration for controlling output randomness (0.0 - 2.0)

2.3 The system SHALL support environment variable overrides for all global configuration options:

2.3.1 The system SHALL read `LLM_BASE_URL` environment variable for base_url

2.3.2 The system SHALL read `LLM_API_KEY` environment variable for api_key

2.3.3 The system SHALL read `LLM_MODEL` environment variable for model

2.3.4 The system SHALL read `LLM_PROXY_URL` environment variable for proxy_url

2.3.5 The system SHALL read `LLM_TIMEOUT` environment variable for timeout

2.3.6 The system SHALL read `LLM_RATE_LIMIT` environment variable for rate_limit

2.3.7 The system SHALL read `LLM_MAX_TOKENS` environment variable for max_tokens

2.4 The system SHALL provide a command to generate a sample global configuration file

---

## Requirement 3: Custom HTTP Headers Support

**User Story:** As a user, I want to specify custom HTTP headers for LLM API requests, so that I can accommodate providers that require additional headers for authentication or routing.

**Acceptance Criteria:**

3.1 The system SHALL support an `extra_headers` configuration section in the global config

3.2 The system SHALL include custom headers in all API requests

3.3 Custom headers SHALL be able to override default headers (except critical security headers)

3.4 The system SHALL support any string-based header name and value pairs

3.5 The system SHALL properly escape and encode header values according to HTTP standards

---

## Requirement 4: Provider-Specific Parameters

**User Story:** As a user, I want to pass provider-specific parameters to the LLM API, so that I can use advanced features like reasoning mode or other vendor-specific options.

**Acceptance Criteria:**

4.1 The system SHALL support an `extra_params` configuration section for provider-specific parameters

4.2 The system SHALL pass extra parameters in the request body to the API

4.3 The system SHALL support boolean, string, integer, and float parameter types

4.4 The system SHALL document common provider-specific parameters (e.g., `reasoning` for reasoning mode)

4.5 Extra parameters SHALL be merged with standard parameters without conflicts

---

## Requirement 5: Translation Prompt Engineering

**User Story:** As a user, I want the LLM translator to use optimized prompts for code translation, so that technical content is translated accurately while preserving code structure and formatting.

**Acceptance Criteria:**

5.1 The system SHALL use a system prompt that instructs the model to act as a professional translator for code comments and technical documentation

5.2 The system prompt SHALL instruct the model to:

5.2.1 Translate only natural language content and preserve all code syntax

5.2.2 Maintain markdown formatting if present

5.2.3 Preserve TODO, FIXME, NOTE, and similar markers

5.2.4 Keep code examples and fenced code blocks unchanged

5.2.5 Return ONLY the translated text without explanations

5.2.6 Skip translation for segments that appear to be code rather than natural language

5.3 The system SHALL include source and target language information in the user prompt

5.4 The system SHALL handle "AUTO" source language by instructing the model to auto-detect

---

## Requirement 6: Batch Translation with Rate Limiting

**User Story:** As a user, I want to translate multiple text segments efficiently, so that I can process large codebases while respecting API rate limits.

**Acceptance Criteria:**

6.1 The system SHALL support batch translation of multiple text segments

6.2 The system SHALL implement configurable rate limiting based on the global configuration

6.3 The system SHALL use a token bucket algorithm for rate limiting

6.4 The system SHALL support concurrent workers for parallel translation (default: 5)

6.5 The system SHALL limit maximum batch size to prevent API timeouts (default: 50 items)

6.6 The system SHALL process large batches in chunks automatically

6.7 The system SHALL provide progress feedback during batch translation

---

## Requirement 7: Error Handling and Retry Logic

**User Story:** As a user, I want robust error handling and automatic retry, so that transient failures don't cause the entire translation job to fail.

**Acceptance Criteria:**

7.1 The system SHALL implement exponential backoff retry for failed requests

7.2 The system SHALL distinguish between retryable errors (5xx, 429) and non-retryable errors (4xx except 429)

7.3 The system SHALL respect the configured maximum retry count (default: 3)

7.4 The system SHALL implement a base delay of 1 second with exponential increase

7.5 The system SHALL add jitter to retry delays to avoid thundering herd

7.6 The system SHALL provide clear error messages for API failures

7.7 The system SHALL include the HTTP status code in error messages

---

## Requirement 8: Configuration Validation

**User Story:** As a user, I want clear validation of my LLM configuration, so that I can identify and fix configuration errors before attempting translation.

**Acceptance Criteria:**

8.1 The system SHALL validate that `base_url` is provided when provider is set to "llm"

8.2 The system SHALL validate that `model` is provided when provider is set to "llm"

8.3 The system SHALL validate that `rate_limit` is a positive integer

8.4 The system SHALL validate that `temperature` is between 0.0 and 2.0

8.5 The system SHALL validate that `timeout` is a positive integer

8.6 The system SHALL provide clear error messages for validation failures

8.7 The system SHALL fail fast on configuration errors before starting translation

---

## Requirement 9: Backward Compatibility

**User Story:** As an existing user, I want my current configuration to continue working, so that I can upgrade without breaking my existing setup.

**Acceptance Criteria:**

9.1 The system SHALL maintain backward compatibility with existing single-file configuration

9.2 The system SHALL support migration from old configuration format to new separated format

9.3 The system SHALL provide a migration command or automatic migration

9.4 The system SHALL continue to support the "deeplx" provider without changes

9.5 The system SHALL mark old configuration structures as deprecated but functional

---

## Requirement 10: Security and Privacy

**User Story:** As a user, I want my API keys and sensitive data to be handled securely, so that my credentials are not exposed.

**Acceptance Criteria:**

10.1 The system SHALL store global configuration files with restricted permissions (0600 on Unix)

10.2 The system SHALL NOT log API keys in error messages or logs

10.3 The system SHALL support reading API keys from environment variables

10.4 The system SHALL use HTTPS by default for API communication

10.5 The system SHALL validate SSL certificates (with option to disable for self-hosted instances)

10.6 The system SHALL NOT include API keys in generated sample configuration files

---

## Configuration Examples

### Global Configuration (`~/.config/translator/config.toml`)

```toml
# Provider selection: "deeplx" or "llm"
provider = "llm"

[deeplx]
proxy_url = ""
dl_session = ""
rate_limit = 10

[llm]
base_url = "https://api.openai.com/v1"
api_key = "sk-..."
model = "gpt-3.5-turbo"
proxy_url = ""
timeout = 60
rate_limit = 10
max_tokens = 4096
temperature = 0.3

[llm.extra_headers]
# X-Custom-Header = "value"

[llm.extra_params]
# reasoning = true
```

### Project Configuration (`translator.toml` in target codebase)

```toml
[translate]
source_lang = "AUTO"
target_lang = "EN"

[include]
patterns = [
    "**/*.go",
    "**/*.py",
    "**/*.js",
]

[exclude]
patterns = [
    "vendor/**",
    "node_modules/**",
]

[cache]
enabled = true
directory = ".translator-cache"

[writer]
dry_run = false
backup = false
```

---

## API Request/Response Format

### Request

```json
{
  "model": "gpt-3.5-turbo",
  "messages": [
    {
      "role": "system",
      "content": "You are a professional translator..."
    },
    {
      "role": "user",
      "content": "Translate the following text from ZH to EN:\n\n原文内容"
    }
  ],
  "max_tokens": 4096,
  "temperature": 0.3,
  "stream": false
}
```

### Response

```json
{
  "id": "chatcmpl-...",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "gpt-3.5-turbo",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Translated content"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 100,
    "completion_tokens": 50,
    "total_tokens": 150
  }
}
```
