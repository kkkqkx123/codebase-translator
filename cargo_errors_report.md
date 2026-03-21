# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 10
- **Total Warnings**: 0
- **Total Issues**: 10
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 10

## Error Statistics

**Total Errors**: 10

### Error Type Breakdown

- **error[E0432]**: 10 errors

### Files with Errors (Top 10)

- `src\parser\languages\cpp\parser.rs`: 1 errors
- `src\parser\languages\csharp\parser.rs`: 1 errors
- `src\parser\languages\java\parser.rs`: 1 errors
- `src\parser\languages\javascript\parser.rs`: 1 errors
- `src\parser\languages\python\parser.rs`: 1 errors
- `src\parser\languages\go\parser.rs`: 1 errors
- `src\parser\engine\tree_sitter.rs`: 1 errors
- `src\parser\languages\c\parser.rs`: 1 errors
- `src\parser\languages\rust\parser.rs`: 1 errors
- `src\parser\languages\typescript\parser.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0432]: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

**Total Occurrences**: 10  
**Unique Files**: 10

#### `src\parser\engine\tree_sitter.rs`: 1 occurrences

- Line 808: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

#### `src\parser\languages\cpp\parser.rs`: 1 occurrences

- Line 474: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

#### `src\parser\languages\csharp\parser.rs`: 1 occurrences

- Line 454: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

#### `src\parser\languages\go\parser.rs`: 1 occurrences

- Line 384: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

#### `src\parser\languages\java\parser.rs`: 1 occurrences

- Line 379: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

#### `src\parser\languages\javascript\parser.rs`: 1 occurrences

- Line 455: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

#### `src\parser\languages\c\parser.rs`: 1 occurrences

- Line 432: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

#### `src\parser\languages\rust\parser.rs`: 1 occurrences

- Line 429: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

#### `src\parser\languages\typescript\parser.rs`: 1 occurrences

- Line 434: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

#### `src\parser\languages\python\parser.rs`: 1 occurrences

- Line 586: unresolved import `crate::parser::abstraction::strategy::ConfigBasedStrategy`: no `ConfigBasedStrategy` in `parser::abstraction::strategy`

