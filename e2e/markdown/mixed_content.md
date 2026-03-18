# Mixed Language Markdown Document

This is an English-only paragraph. It should NOT be extracted when source language is zh and target is en.

这是一个纯中文的段落。当源语言为zh，目标语言为en时，应该被提取。

## Section with Mixed Content

This section contains both English and 中文内容. Since it has Chinese characters, it should be extracted.

### Pure English Subsection

All content here is in English. No translation needed for zh to en.

### 纯中文小节

这里的内容都是中文。需要被提取和翻译。

## Code Examples

```rust
// This English comment should NOT be extracted
fn main() {
    println!("Hello World");  // English - not extracted
    println!("你好世界");      // Chinese - extracted
}
```

```python
# 这是一个中文注释，应该被提取
def hello():
    print("Hello")  # English - not extracted
    print("你好")   # Chinese - extracted
```

## Lists

English items (not extracted):
- First item
- Second item
- Third item

中文项目（应该被提取）：
- 第一项
- 第二项
- 第三项

Mixed items (extracted because of Chinese):
- First 项目
- Second item
- 第三项

## Blockquotes

> This is an English quote. Not extracted.

> 这是一个中文引用。应该被提取。

> Mixed quote: Hello 世界. Extracted due to Chinese.

## Tables

| English Header | Another English |
|----------------|-----------------|
| English cell   | English value   |

| 中文标题 | 英文Header |
|----------|------------|
| 中文内容 | English    |

## Links and References

[English Link](https://example.com)

[中文链接](https://example.com)

## Emphasis

*English italic* - not extracted

**English bold** - not extracted

*中文斜体* - extracted

**中文粗体** - extracted

***Mixed 混合 emphasis*** - extracted
