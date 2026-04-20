# Translator Routing Strategy Improvement

## Current Strategy

- **Rule**: If text contains placeholders (`${...}`), prefer LLM translator
- **Problem**:
  - LLM is expensive and slow
  - LLM (especially translation models) may corrupt placeholders
  - DeepLX/Tencent actually handle placeholders perfectly

## Test Results

### DeepLX with Placeholders

- Input: `不支持的 LLM 提供商：${profile.provider}`
- Output: `Unsupported LLM providers: ${profile.provider}` ✅
- **Result**: Perfect placeholder preservation

### DeepLX with Multiple Placeholders

- Input: `错误：${error}，代码：${code}，消息：${message}`
- Output: `Error: ${error}, Code: ${code}, Message: ${message}` ✅
- **Result**: All placeholders preserved

### LLM (Hunyuan-MT-7B) with Placeholders

- Input: `不支持的 LLM 提供商：${profile.provider}`
- Output: `Unsupported LLM provider: ${profile-provider}` ❌
- **Result**: Placeholder corrupted (`.` → `-`)

### LLM with [[index]] Protection

- Input: `不支持的 LLM 提供商：[[0]]`
- Output: `LLM (Large Language Model) providers that are not supported: [[0]]` ✅
- **Result**: Marker preserved, requires post-processing restoration

## Proposed Strategy

### Priority Order

1. **DeepLX** (first choice)
   - Fast
   - Cheap
   - Perfect placeholder preservation
   - Good quality for technical content

2. **Tencent Cloud** (second choice)
   - Fast
   - Reasonable cost
   - Good placeholder preservation (assumed, needs testing)

3. **LLM** (fallback only)
   - Use only when DeepLX/Tencent unavailable
   - Use placeholder protection mechanism ([[index]] markers)
   - Slower and more expensive

### Implementation

```rust
fn select_translator_internal(&self, has_placeholders: bool) -> Option<&TranslatorEntry> {
    let healthy_translators: Vec<&TranslatorEntry> =
        self.translators.iter().filter(|t| t.is_healthy()).collect();

    if healthy_translators.is_empty() {
        // If no healthy translators, try all translators
        let total = self.translators.len();
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) as usize % total;
        return self.translators.get(index);
    }

    // Priority 1: Try DeepLX first (best for placeholders)
    if let Some(deeplx) = healthy_translators
        .iter()
        .find(|t| t.name.to_lowercase().contains("deeplx"))
    {
        return Some(*deeplx);
    }

    // Priority 2: Try Tencent Cloud
    if let Some(tencent) = healthy_translators
        .iter()
        .find(|t| t.name.to_lowercase().contains("tencent"))
    {
        return Some(*tencent);
    }

    // Priority 3: LLM as fallback (with placeholder protection)
    if let Some(llm) = healthy_translators
        .iter()
        .find(|t| t.name.to_lowercase().contains("llm"))
    {
        return Some(*llm);
    }

    // Default: round-robin
    let index = self.current_index.fetch_add(1, Ordering::Relaxed) as usize % healthy_translators.len();
    healthy_translators.get(index).copied()
}
```

### Benefits

1. **Cost reduction**: Use cheap machine translation APIs instead of expensive LLM
2. **Better quality**: DeepLX preserves placeholders naturally without complex protection
3. **Faster**: Machine translation APIs are much faster than LLM
4. **Simpler**: No need for placeholder protection logic when using DeepLX

### When LLM is Still Useful

1. **Fallback**: When DeepLX/Tencent are unavailable or rate-limited
2. **Complex context**: When translation requires deep understanding of code context
3. **Custom terminology**: When custom system prompts are needed

## Action Items

1. Change translator selection priority (DeepLX > Tencent > LLM)
2. Keep placeholder protection in LLM provider (for fallback scenarios)
3. Test Tencent Cloud with placeholders to confirm behavior
4. Update documentation to reflect new strategy
