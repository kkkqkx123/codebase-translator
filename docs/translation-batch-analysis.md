# 翻译批处理与Parser协调机制分析

## 一、翻译批处理实现分析

### 1.1 架构层级

项目采用**三层翻译架构**：

```
TranslationService (同步接口层)
    ↓
BatchTranslator (批处理/限流层)
    ↓
MultiTranslator (负载均衡/故障转移层)
    ↓
TranslatorImpl (具体实现层: DeepLX/LLM/Tencent)
```

### 1.2 BatchTranslator 核心功能

**文件位置**: `src/translator/batch.rs`

| 功能 | 实现方式 |
|------|----------|
| **速率限制** | 使用 `governor` crate 实现每秒请求限制 |
| **并发控制** | `Semaphore` 控制并发工作线程数 |
| **重试机制** | 指数退避策略 (1s, 2s, 4s...) |
| **文本分块** | 三级分层策略: 段落 → 句子 → 字符 |

**文本分块策略**:
```rust
// 分层策略确保不超过API限制
fn split_text_hierarchical(&self, text: &str) -> Vec<String> {
    // Level 1: 按段落分割
    // Level 2: 按句子分割 (. ! ? 。！？)
    // Level 3: 按字符分割 (最终保障)
}
```

### 1.3 MultiTranslator 负载均衡

**文件位置**: `src/translator/multi.rs`

- **选择策略**: RoundRobin / Weighted
- **健康检查**: 原子布尔值跟踪每个翻译器健康状态
- **故障转移**: 失败1次立即标记为不健康，自动切换到下一个

---

## 二、Parser与翻译器协调机制

### 2.1 数据流架构

```
文件扫描 → ParserCoordinator.parse_file() → TranslationUnit列表
                                                  ↓
                                        FileProcessor.process()
                                                  ↓
                  ┌──────────────────────────────────────────────┐
                  ↓                                              ↓
        ContentFilter.should_translate()              翻译器批量翻译
                  ↓                                              ↓
        过滤不需要翻译的文本                          translator.translate_batch()
                  ↓                                              ↓
        构建待翻译文本列表                              返回翻译结果
                  └──────────────────────────────────────────────┘
                                                  ↓
                                        写入文件/更新缓存
```

### 2.2 ParserCoordinator 职责

**文件位置**: `src/parser/coordinator/coordinator.rs`

| 阶段 | 处理内容 |
|------|----------|
| **解析** | Tree-sitter 优先 → Regex 回退 |
| **自定义模式** | 应用正则模式和状态机模式 |
| **去重** | 基于(offset, content)去重 |
| **排序** | 按文件位置排序 |
| **过滤** | 应用 `ContentFilter` |

### 2.3 协调关键点

**文件位置**: `src/workflow/file_processor.rs`

当前协调逻辑：

```rust
// 1. 解析文件获取翻译单元
let mut units = self.parser.parse_file(&file)?;
result.total_units = units.len();

// 2. 过滤出需要翻译的单元
let units_to_translate: Vec<_> = units.iter()
    .filter(|u| u.should_translate)
    .collect();

// 3. 提取文本内容
let texts: Vec<String> = units_to_translate
    .iter()
    .map(|u| u.content.clone())
    .collect();

// 4. 批量翻译 (同步调用)
let translated_texts = self.translator
    .translate_batch(&texts, &target_lang)?;

// 5. 回填结果
for unit in units.iter_mut() {
    if unit.should_translate {
        unit.set_translated(translated.clone());
    }
}
```

---

## 三、存在的问题与改进建议

### 3.1 当前问题

| 问题 | 影响 | 位置 |
|------|------|------|
| **同步阻塞** | `file_processor.rs:137` 创建新的tokio运行时，每个文件都阻塞等待翻译 | `file_processor.rs` |
| **粒度粗糙** | 按文件批量翻译，无法跨文件聚合请求 | `file_processor.rs` |
| **重复初始化** | 每次处理文件都重建运行时 | `service.rs` |
| **无预取机制** | 解析和翻译完全串行，无流水线 | 整体架构 |
| **内存累积** | 所有翻译单元同时加载到内存 | `file_processor.rs` |

### 3.2 改进建议

#### 1. 引入异步流水线 (推荐)

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│ 文件扫描     │────→│ 解析队列      │────→│ 解析工作线程  │
│ (Producer)  │     │ (Channel)    │     │ (Workers)   │
└─────────────┘     └──────────────┘     └──────┬──────┘
                                                 ↓
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│ 结果写入     │←────│ 翻译结果队列   │←────│ 批量翻译器    │
│ (Consumer)  │     │ (Channel)    │     │ (Aggregator)│
└─────────────┘     └──────────────┘     └─────────────┘
```

#### 2. 跨文件批量翻译

当前每个文件单独调用 `translate_batch()`，建议：

```rust
// 改进: 累积多个文件的文本统一翻译
struct TranslationBuffer {
    pending_units: Vec<(FileId, UnitId, String)>,
    buffer_size: usize,  // 按字符数或单元数触发
}

impl TranslationBuffer {
    fn push(&mut self, unit: TranslationUnit) -> Option<TranslationTask> {
        // 累积到足够数量后触发批量翻译
        if self.should_flush() {
            return Some(self.flush());
        }
        None
    }
}
```

#### 3. 流式处理大文件

对于大文件，应采用流式解析而非一次性加载：

```rust
// 当前: 一次性解析所有单元
let units = self.parser.parse_file(&file)?;  // 可能占用大量内存

// 改进: 迭代器模式
let unit_stream = self.parser.parse_file_stream(&file)?;
for batch in unit_stream.chunks(100) {
    self.translator.translate_batch(batch).await;
}
```

#### 4. 预解析与翻译并行

```rust
// 当前: 完全串行
for file in files {
    let units = parser.parse(&file)?;           // 阻塞
    let translated = translator.translate(units); // 阻塞
    writer.write(&file, translated).await;      // 阻塞
}

// 改进: 并行流水线
let (parse_tx, parse_rx) = channel::bounded(100);
let (translate_tx, translate_rx) = channel::bounded(100);

// 三个并发任务
spawn(|| scan_and_parse(parse_tx));
spawn(|| parse_and_translate(parse_rx, translate_tx));
spawn(|| translate_and_write(translate_rx));
```

#### 5. 减少运行时创建开销

当前 `TranslationService` 每次创建新运行时，建议复用：

```rust
// 当前
pub fn translate_batch(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>> {
    let runtime = tokio::runtime::Runtime::new()?;  // 每次新建!
    runtime.block_on(async { ... })
}

// 改进: 使用线程本地存储或全局运行时
thread_local! {
    static RUNTIME: Runtime = Runtime::new().expect("Failed to create Tokio runtime");
}
```

---

## 四、总结

当前架构的优点：
- **模块化清晰**：Parser、Translator、Writer 分离
- **多层容错**：MultiTranslator 提供故障转移
- **速率控制**：BatchTranslator 实现限流和重试

主要改进方向：
1. **异步化改造**：消除 `block_on` 阻塞，引入真正的异步流水线
2. **批量聚合**：跨文件累积翻译单元，减少API调用次数
3. **流式处理**：大文件采用迭代器模式，降低内存占用
4. **预取优化**：解析和翻译并行执行，提高吞吐量