# Dynamic Dispatch Usage Documentation

## Overview

This document tracks all instances of dynamic dispatch (`dyn`) usage in the Rust codebase, as required by the project rules.

## Remaining Dynamic Dispatch Usage

### 1. Logger Module - NECESSARY ✅

**Location:** `src/logger/mod.rs`

**Instance 1 (Line 18):**
```rust
pub static LOG_GUARD: OnceLock<Box<dyn std::any::Any + Send + Sync>> = OnceLock::new();
```

**Reason for use:**
- Type erasure required to store different types of log guards
- `Any` trait is the standard Rust pattern for type erasure
- Cannot be eliminated without redesigning the logging system

**Instance 2, 3, 4 (Lines 45, 84, 146):**
```rust
let fmt_layer: Box<dyn tracing_subscriber::Layer<_> + Send + Sync> = match format {
    "json" => Box::new(tracing_subscriber::fmt::layer().json()),
    "compact" => Box::new(tracing_subscriber::fmt::layer().compact()),
    _ => Box::new(tracing_subscriber::fmt::layer().pretty()),
};
```

**Reason for use:**
- External library trait (`tracing_subscriber::Layer`)
- Multiple layer types (json, compact, pretty) from external crate return different concrete types
- Cannot be changed to enum without forking the library
- Runtime configuration requires dynamic selection of layer type

**Status:** Acceptable and necessary

## Summary

- **Total dyn usage:** 4 instances (all in logger module)
- **Necessary:** 4/4 (100%)=
