# Dynamic Dispatch Usage Documentation

## Overview

This document tracks all instances of dynamic dispatch (`dyn`) usage in the Rust codebase, as required by the project rules.

## Current Status

**Total dyn usage:** 32 instances across multiple modules

All instances are necessary and acceptable for the following reasons:
1. **Reporter trait**: Used for progress and statistics reporting across the codebase
2. **External library traits**: Required by `tracing_subscriber::Layer` for logging
3. **Type erasure**: Used for storing log guards with different concrete types

## Detailed Breakdown

### 1. Logger Module - NECESSARY ✅

**Location:** `src/logger/mod.rs`

**Instance 1 (Line 20):**
```rust
pub static LOG_GUARD: OnceLock<Box<dyn std::any::Any + Send + Sync>> = OnceLock::new();
```

**Reason for use:**
- Type erasure required to store different types of log guards
- `Any` trait is the standard Rust pattern for type erasure
- Cannot be eliminated without redesigning the logging system

**Instance 2, 3, 4 (Lines 52, 91, 167):**
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

---

### 2. Reporter Trait - NECESSARY ✅

**Location:** Multiple modules throughout the codebase

**Reason for use:**
- The `Reporter` trait is a core abstraction for progress and statistics reporting
- Enables dependency injection and loose coupling between translation components and reporting
- Allows for different reporter implementations (e.g., for testing, different output formats)
- Runtime configuration and optional reporter require dynamic dispatch
- Essential for the testability and flexibility of the codebase

#### 2.1 Reporter Trait Definition

**Location:** `src/reporter/trait.rs`

**Lines 21:** Trait definition
```rust
pub trait Reporter: Send + Sync {
    // ... trait methods
}
```

**Line 204, 208 (src/reporter/default.rs):** Factory functions returning `Arc<dyn Reporter>`
```rust
pub fn create_reporter() -> Arc<dyn Reporter> {
    Arc::new(DefaultReporter::new())
}

pub fn create_reporter_with_stats(shared_stats: Arc<SharedStats>) -> Arc<dyn Reporter> {
    Arc::new(DefaultReporter::with_shared_stats(shared_stats))
}
```

#### 2.2 Translator Modules

**DeepLX Translator** (`src/translator/deeplx.rs`)
- Line 39: `reporter: Option<Arc<dyn Reporter>>`
- Line 262: `fn set_reporter(&mut self, reporter: Arc<dyn Reporter>)`
- Line 266: `fn reporter(&self) -> Option<Arc<dyn Reporter>>`

**Tencent Cloud Translator** (`src/translator/tencent.rs`)
- Line 78: `reporter: Option<Arc<dyn Reporter>>`
- Line 410: `fn set_reporter(&mut self, reporter: Arc<dyn Reporter>)`
- Line 414: `fn reporter(&self) -> Option<Arc<dyn Reporter>>`

**LLM Multi Translator** (`src/translator/llm/multi_translator.rs`)
- Line 28: `reporter: Option<Arc<dyn Reporter>>`
- Line 325: `fn set_reporter(&mut self, reporter: Arc<dyn Reporter>)`
- Line 335: `fn reporter(&self) -> Option<Arc<dyn Reporter>>`

**LLM Provider** (`src/translator/llm/provider.rs`)
- Line 397: `reporter: Option<Arc<dyn Reporter>>`
- Line 548: `pub fn set_reporter(&mut self, reporter: Arc<dyn Reporter>)`
- Line 553: `pub fn reporter(&self) -> Option<Arc<dyn Reporter>>`
- Line 1161: `fn set_reporter(&mut self, reporter: Arc<dyn Reporter>)`
- Line 1165: `fn reporter(&self) -> Option<Arc<dyn Reporter>>`

**Translator Trait** (`src/translator/trait.rs`)
- Line 78: `fn set_reporter(&mut self, reporter: Arc<dyn Reporter>)`
- Line 81: `fn reporter(&self) -> Option<Arc<dyn Reporter>>`
- Line 207: `fn set_reporter(&mut self, reporter: Arc<dyn Reporter>)`
- Line 215: `fn reporter(&self) -> Option<Arc<dyn Reporter>>`

#### 2.3 Workflow Modules

**Workflow Executor** (`src/workflow/executor.rs`)
- Line 77: `reporter: Option<Arc<dyn Reporter>>`
- Line 113: `pub fn with_reporter(mut self, reporter: Arc<dyn Reporter>) -> Self`

**Workflow Builder** (`src/workflow/builder.rs`)
- Line 35: `reporter: Option<Arc<dyn Reporter>>`
- Line 54: `pub fn with_reporter(mut self, reporter: Arc<dyn Reporter>) -> Self`
- Line 84: `pub fn build_with_reporter(&self) -> Result<(WorkflowComponents, Arc<dyn Reporter>)>`
- Line 129: `pub fn reporter(&self) -> Option<&Arc<dyn Reporter>>`

**File Processor** (`src/workflow/file_processor.rs`)
- Line 88: `reporter: Option<Arc<dyn Reporter>>`
- Line 101: `reporter: Option<Arc<dyn Reporter>>`

**Status:** Acceptable and necessary

## Summary

- **Total dyn usage:** 32 instances
  - Logger module: 4 instances
  - Reporter trait and related: 28 instances
- **Necessary:** 32/32 (100%)
- **Can be eliminated:** 0 instances

## Recommendations

All current uses of dynamic dispatch are justified by external library requirements (tracing_subscriber) or design patterns that require runtime polymorphism (Reporter trait for dependency injection and testability). No changes are recommended.

## Audit Date

2026-06-11