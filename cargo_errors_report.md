# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 8
- **Total Issues**: 8
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 1
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 8

### Warning Type Breakdown

- **warning**: 8 warnings

### Files with Warnings (Top 10)

- `src\cache\binary.rs`: 8 warnings

## Detailed Warning Categorization

### warning: call to `.clone()` on a reference in this situation does nothing: help: remove this redundant call

**Total Occurrences**: 8  
**Unique Files**: 1

#### `src\cache\binary.rs`: 8 occurrences

- Line 614: call to `.clone()` on a reference in this situation does nothing: help: remove this redundant call
- Line 623: call to `.clone()` on a reference in this situation does nothing: help: remove this redundant call
- Line 659: call to `.clone()` on a reference in this situation does nothing: help: remove this redundant call
- ... 5 more occurrences in this file

