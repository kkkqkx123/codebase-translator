# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 0
- **Total Issues**: 2
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 0
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0609]**: 1 errors
- **error[E0614]**: 1 errors

### Files with Errors (Top 10)

- `tests\config_validation.rs`: 1 errors
- `tests\writer_integration\file_writer_tests.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0609]: no field `writer_dry_run` on type `&ProjectConfigSummary`: unknown field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\config_validation.rs`: 1 occurrences

- Line 1321: no field `writer_dry_run` on type `&ProjectConfigSummary`: unknown field

### error[E0614]: type `codebase_translate::writer::WriterConfig` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\writer_integration\file_writer_tests.rs`: 1 occurrences

- Line 283: type `codebase_translate::writer::WriterConfig` cannot be dereferenced: can't be dereferenced

