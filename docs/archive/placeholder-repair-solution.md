# Solution Analysis: Post-Translation Placeholder Repair

## Problem
LLM translation models (like tencent/Hunyuan-MT-7B) corrupt placeholders:
- Input: `${profile.provider}`
- Output: `${profile-provider}` (dot changed to dash)

## Proposed Solution: Translate First, Fix Later

### Approach
1. **Don't mention placeholders in prompts** - Avoid triggering LLM's "normalization" instinct
2. **Translate with original placeholders** - Send `${...}` as-is to LLM
3. **Post-translation repair** - Detect and fix corrupted placeholders

### Implementation Strategy

#### Step 1: Remove Placeholder Instructions from Prompts
Delete all mentions of placeholders from system/user prompts:
```rust
fn build_system_prompt(&self) -> String {
    r#"You are a professional code comment translator. Translate natural language content to the target language.

Rules:
- Return ONLY the translated text
- Preserve code syntax, URLs, and special characters exactly
- Keep existing formatting in the original text
- Do not add explanations or markdown wrappers"#.to_string()
}
```

#### Step 2: Post-Translation Placeholder Repair
After receiving translation, repair any corrupted placeholders:

```rust
pub fn repair_placeholders(translated: &str, original: &str) -> String {
    // Extract all ${...} patterns from original
    let original_placeholders = extract_placeholders(original);
    
    // Extract all ${...} patterns from translated
    let translated_placeholders = extract_placeholders(translated);
    
    // For each original placeholder, find its corrupted version in translation
    let mut result = translated.to_string();
    for (orig_idx, orig_ph) in original_placeholders.iter().enumerate() {
        if orig_idx < translated_placeholders.len() {
            let trans_ph = &translated_placeholders[orig_idx];
            // Replace corrupted placeholder with original
            result = result.replace(trans_ph, orig_ph);
        }
    }
    
    result
}

fn extract_placeholders(text: &str) -> Vec<String> {
    // Find all ${...} patterns, handling nested braces
    let mut placeholders = Vec::new();
    let mut chars = text.chars().peekable();
    let mut i = 0;
    
    while i < text.len() {
        if text[i..].starts_with("${") {
            let start = i;
            let mut depth = 1;
            let mut j = i + 2;
            while j < text.len() && depth > 0 {
                if text[j..].starts_with('{') {
                    depth += 1;
                } else if text[j..].starts_with('}') {
                    depth -= 1;
                }
                if depth > 0 {
                    j += 1;
                }
            }
            if depth == 0 {
                placeholders.push(text[start..=j].to_string());
                i = j + 1;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    
    placeholders
}
```

### Advantages
1. **No cost increase** - Same number of API calls
2. **Works with any LLM** - Even pure translation models
3. **Simple and reliable** - Direct string replacement
4. **No complex prompt engineering** - Less room for error

### Potential Issues & Solutions

#### Issue 1: Placeholder order might change
**Solution:** Match by position (first in original → first in translation)

#### Issue 2: LLM might remove placeholder entirely
**Solution:** Detect missing placeholders and log warning, but don't fail

#### Issue 3: LLM might add extra placeholders
**Solution:** Only repair placeholders that exist in original

#### Issue 4: Variable name translation (e.g., `provider` → `提供者`)
**Solution:** This is actually CORRECT behavior - we only want to repair syntax (dots, braces), not prevent translation of variable names if that's intentional

Wait... this reveals a fundamental problem:

## Critical Analysis: What Should NOT Be Translated?

### Case 1: Code Variables (Should NOT translate)
- Original: `${profile.provider}`
- Expected: `${profile.provider}` (unchanged)
- LLM output: `${profile-provider}` (corrupted syntax)

### Case 2: Natural Language with Placeholders (Should translate around it)
- Original: `错误：${error}`
- Expected: `Error: ${error}`
- LLM output: `错误：${error}` (if translating to English, this is wrong)

The issue is: **How does LLM know what `${...}` contains?**
- If it's a code variable → Keep unchanged
- If it's natural language → Translate the surrounding text only

### Reality Check
Looking at actual use cases:
- `${profile.provider}` - This is a code variable path
- `${error}` - This is a code variable
- `${user.name}` - This is a code variable path

**All `${...}` patterns in code comments are code variables, not translatable content!**

Therefore, the LLM should NOT be translating the content inside `${...}` at all.

## Conclusion

The post-translation repair approach CAN work, but we need to:

1. **Remove all placeholder instructions from prompts** (to avoid triggering LLM normalization)
2. **Implement smart repair logic** that:
   - Extracts original placeholders
   - Finds corresponding (possibly corrupted) placeholders in translation
   - Replaces them with originals
3. **Accept that some corruption is inevitable** with pure translation models
4. **Consider model choice carefully** - code models vs translation models

The key insight: **Don't fight the LLM, just fix its mistakes afterwards.**
