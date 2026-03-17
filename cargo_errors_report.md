# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 4
- **Total Issues**: 4
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 4
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 4

### Warning Type Breakdown

- **warning**: 4 warnings

### Files with Warnings (Top 10)

- `src\parser\string_extractor.rs`: 2 warnings
- `src\cache\binary.rs`: 1 warnings
- `src\parser\tree_sitter.rs`: 1 warnings

## Detailed Warning Categorization

### warning: constant `INDEX_ENTRY_SIZE` is never used

**Total Occurrences**: 4  
**Unique Files**: 3

#### `src\parser\string_extractor.rs`: 2 occurrences

- Line 559: use of deprecated constant `parser::string_extractor::tests::test_category_as_str`: This module is not currently used and may be removed in a future version
- Line 568: use of deprecated constant `parser::string_extractor::tests::test_config_category_enabled`: This module is not currently used and may be removed in a future version

#### `src\cache\binary.rs`: 1 occurrences

- Line 20: constant `INDEX_ENTRY_SIZE` is never used

#### `src\parser\tree_sitter.rs`: 1 occurrences

- Line 11: unused import: `FormatInfo`

