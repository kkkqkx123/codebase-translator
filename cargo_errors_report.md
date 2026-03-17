# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 8
- **Total Issues**: 8
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 8
- **Files with Issues**: 7

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 8

### Warning Type Breakdown

- **warning**: 8 warnings

### Files with Warnings (Top 10)

- `src\parser\string_extractor.rs`: 2 warnings
- `src\translator\routing.rs`: 1 warnings
- `src\parser\tree_sitter.rs`: 1 warnings
- `src\parser\coordinator\coordinator.rs`: 1 warnings
- `src\parser\languages\c\parser.rs`: 1 warnings
- `src\translator\llm\multi_translator.rs`: 1 warnings
- `src\translator\llm\routing.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `super::*`

**Total Occurrences**: 8  
**Unique Files**: 7

#### `src\parser\string_extractor.rs`: 2 occurrences

- Line 559: use of deprecated constant `parser::string_extractor::tests::test_category_as_str`: This module is not currently used and may be removed in a future version
- Line 568: use of deprecated constant `parser::string_extractor::tests::test_config_category_enabled`: This module is not currently used and may be removed in a future version

#### `src\translator\routing.rs`: 1 occurrences

- Line 167: unused import: `super::*`

#### `src\translator\llm\routing.rs`: 1 occurrences

- Line 76: field `total_weight` is never read

#### `src\parser\coordinator\coordinator.rs`: 1 occurrences

- Line 165: unused variable: `filename`: help: if this is intentional, prefix it with an underscore: `_filename`

#### `src\parser\languages\c\parser.rs`: 1 occurrences

- Line 47: method `clean_comment_text` is never used

#### `src\translator\llm\multi_translator.rs`: 1 occurrences

- Line 20: field `max_retries` is never read

#### `src\parser\tree_sitter.rs`: 1 occurrences

- Line 11: unused import: `FormatInfo`

