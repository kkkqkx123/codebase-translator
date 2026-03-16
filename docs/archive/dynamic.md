# Dynamic Dispatch Usage Documentation

## Overview

This document tracks all instances of dynamic dispatch (`dyn`) usage in the Rust codebase, as required by the project rules.

## Remaining Dynamic Dispatch Usage

### 1. Logger Module - NECESSARY ✅

**Location:** `src/logger/mod.rs`

```rust
LOG_GUARD: OnceLock<Box<dyn std::any::Any + Send + Sync>>
Box<dyn tracing_subscriber::Layer<_> + Send + Sync>
```

**Reason for use:**
- External library trait (`tracing_subscriber::Layer`)
- Multiple layer types (json, compact, pretty) from external crate
- Cannot be changed to enum without forking the library

**Status:** Acceptable and necessary
