# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 5
- **Total Issues**: 5
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 5
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 5

### Warning Type Breakdown

- **warning**: 5 warnings

### Files with Warnings (Top 10)

- `src\translator\llm\routing.rs`: 5 warnings

## Detailed Warning Categorization

### warning: struct `ProviderStats` is never constructed

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\translator\llm\routing.rs`: 5 occurrences

- Line 28: struct `ProviderStats` is never constructed
- Line 39: struct `RouterStats` is never constructed
- Line 90: method `update_effective_weight` is never used
- ... 2 more occurrences in this file

