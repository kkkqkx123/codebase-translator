# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 17
- **Total Issues**: 17
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 15
- **Files with Issues**: 9

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 17

### Warning Type Breakdown

- **warning**: 17 warnings

### Files with Warnings (Top 10)

- `src\parser\core\language_parser.rs`: 6 warnings
- `src\parser\string_extractor.rs`: 3 warnings
- `src\translator\service.rs`: 2 warnings
- `src\translator\llm\provider.rs`: 1 warnings
- `src\parser\coordinator\tests.rs`: 1 warnings
- `src\parser\regex\state_machine.rs`: 1 warnings
- `src\parser\languages\tests\strategy_integration_tests.rs`: 1 warnings
- `src\parser\filter.rs`: 1 warnings
- `src\parser\function_patterns.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `super::*`

**Total Occurrences**: 17  
**Unique Files**: 9

#### `src\parser\core\language_parser.rs`: 6 occurrences

- Line 10: unused import: `Position`
- Line 11: unused import: `QueryMatch`
- Line 18: unused import: `crate::parser::Parser`
- ... 3 more occurrences in this file

#### `src\parser\string_extractor.rs`: 3 occurrences

- Line 420: unused variable: `file_path`: help: if this is intentional, prefix it with an underscore: `_file_path`
- Line 437: unused variable: `matches`: help: if this is intentional, prefix it with an underscore: `_matches`
- Line 455: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\translator\service.rs`: 2 occurrences

- Line 191: unused import: `super::*`
- Line 192: unused imports: `DeepLXConfig` and `ProviderType`

#### `src\parser\coordinator\tests.rs`: 1 occurrences

- Line 7: unused import: `crate::parser::Parser`

#### `src\parser\function_patterns.rs`: 1 occurrences

- Line 429: field `registry` is never read

#### `src\translator\llm\provider.rs`: 1 occurrences

- Line 281: method `close` is never used

#### `src\parser\filter.rs`: 1 occurrences

- Line 360: unused import: `Lang`

#### `src\parser\languages\tests\strategy_integration_tests.rs`: 1 occurrences

- Line 38: function `create_default_strategy` is never used

#### `src\parser\regex\state_machine.rs`: 1 occurrences

- Line 159: unused variable: `match_start`: help: if this is intentional, prefix it with an underscore: `_match_start`

