# Translator Module Design

## 概述

Translator 模块提供翻译服务，支持多个翻译提供商（DeepLX, LLM, Tencent），实现负载均衡、失败重试和速率限制，提供批量翻译和并发处理能力。

## 设计目的

1. **多提供商支持**：集成多个翻译服务，提高可用性和灵活性
2. **负载均衡**：在多个提供商之间分配请求，提高性能
3. **失败重试**：自动重试失败的请求，提高可靠性
4. **速率限制**：遵守 API 速率限制，避免被封禁
5. **并发处理**：批量并发翻译，提高效率

## 核心组件

### 1. Translator Trait

**位置**：`src/translator/trait.rs`

**职责**：
- 定义翻译器接口
- 提供统一的翻译方法

**关键方法**：
```rust
#[async_trait]
pub trait Translator: Send + Sync {
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>>;

    async fn translate_single(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String>;

    fn name(&self) -> &str;
    async fn is_available(&self) -> bool;
    fn supported_source_langs(&self) -> Vec<&str>;
    fn supported_target_langs(&self) -> Vec<&str>;
    fn max_input_chars(&self) -> usize;
    fn can_handle(&self, text_len: usize) -> bool;
    async fn close(&self) -> Result<()>;
    fn set_reporter(&mut self, reporter: Arc<dyn Reporter>);
    fn reporter(&self) -> Option<Arc<dyn Reporter>>;
}
```

### 2. TranslatorImpl

**职责**：
- 静态分发翻译器实现
- 统一不同翻译器的接口

**关键类型**：
```rust
pub enum TranslatorImpl {
    DeepLX(DeepLXTranslator),
    LLM(MultiProviderTranslator),
    Tencent(TencentTranslator),
}

#[async_trait]
impl Translator for TranslatorImpl {
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        match self {
            Self::DeepLX(t) => t.translate(texts, source_lang, target_lang).await,
            Self::LLM(t) => t.translate(texts, source_lang, target_lang).await,
            Self::Tencent(t) => t.translate(texts, source_lang, target_lang).await,
        }
    }
}
```

**优势**：
- 静态分发，零开销
- 编译时类型检查
- 更好的性能

### 3. BatchTranslator

**位置**：`src/translator/batch.rs`

**职责**：
- 批量翻译协调器
- 负载均衡和失败重试
- 速率限制和并发控制

**关键功能**：
```rust
pub struct BatchTranslator {
    translators: Vec<Arc<TranslatorEntry>>,
    rate_limiter: Option<Arc<RateLimiter<NotKeyed>>>,
    semaphore: Arc<Semaphore>,
    max_retries: usize,
    options: BatchOptions,
    shared_stats: Option<Arc<SharedStats>>,
}

struct TranslatorEntry {
    translator: Arc<TranslatorImpl>,
    name: String,
    healthy: AtomicU64,  // 1=healthy, 0=unhealthy
    failure_count: AtomicU64,
}

impl BatchTranslator {
    pub async fn translate_batch(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        // 1. 按大小分组
        let groups = self.group_by_size(texts, self.options.batch_size);

        // 2. 并发翻译各组
        let mut results = Vec::with_capacity(texts.len());
        for group in groups {
            let group_results = self.translate_group(&group, source_lang, target_lang).await?;
            results.extend(group_results);
        }

        Ok(results)
    }

    async fn translate_group(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let mut attempts = 0;
        let max_attempts = self.max_retries + 1;

        loop {
            attempts += 1;

            // 1. 选择健康的翻译器
            let translator = self.select_healthy_translator().await?;

            // 2. 等待速率限制
            self.wait_rate_limit().await;

            // 3. 获取信号量（并发控制）
            let _permit = self.semaphore.acquire().await?;

            // 4. 执行翻译
            match translator.translate(texts, source_lang, target_lang).await {
                Ok(result) => {
                    translator.mark_healthy();
                    return Ok(result);
                }
                Err(e) => {
                    translator.mark_unhealthy();
                    if attempts >= max_attempts {
                        return Err(e);
                    }
                    // 延迟后重试
                    tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                }
            }
        }
    }
}
```

**负载均衡策略**：
- 轮询（Round Robin）
- 最少连接（Least Connections）
- 健康检查优先

### 4. DeepLXTranslator

**位置**：`src/translator/deeplx.rs`

**职责**：
- DeepLX 翻译服务实现
- 免费的 DeepL 翻译

**关键配置**：
```rust
pub struct DeepLXConfig {
    pub api_url: String,
    pub api_key: String,
    pub proxy_url: String,
    pub max_retries: usize,
}
```

**API 请求**：
```rust
async fn translate(
    &self,
    texts: &[String],
    source_lang: &str,
    target_lang: &str,
) -> Result<Vec<String>> {
    let request = TranslateRequest {
        text: texts.to_vec(),
        source_lang: source_lang.to_string(),
        target_lang: target_lang.to_string(),
    };

    let response = self.client
        .post(&self.api_url)
        .header("Authorization", format!("Bearer {}", self.api_key))
        .json(&request)
        .send()
        .await?;

    let result: TranslateResponse = response.json().await?;
    Ok(result.translations)
}
```

### 5. MultiProviderTranslator

**位置**：`src/translator/llm/`

**职责**：
- LLM 多提供商翻译器
- 支持多个 LLM 提供商（OpenAI, Anthropic 等）
- 提供商健康检查和自动切换

**关键功能**：
```rust
pub struct MultiProviderTranslator {
    providers: Vec<Arc<LLMProvider>>,
    stats: Arc<RwLock<HashMap<String, ProviderStats>>>,
    selection_strategy: SelectionStrategy,
}

pub enum SelectionStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    Random,
}

impl MultiProviderTranslator {
    pub async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        // 1. 选择提供商
        let provider = self.select_provider().await?;

        // 2. 构建提示词
        let prompt = self.build_prompt(texts, source_lang, target_lang);

        // 3. 调用 LLM API
        let response = provider.complete(&prompt).await?;

        // 4. 解析响应
        self.parse_response(response, texts.len())
    }
}
```

### 6. TencentTranslator

**位置**：`src/translator/tencent.rs`

**职责**：
- 腾讯云机器翻译实现
- 支持术语库和句子库

**关键配置**：
```rust
pub struct TencentConfig {
    pub secret_id: String,
    pub secret_key: String,
    pub region: String,
    pub project_id: i64,
    pub proxy_url: String,
    pub timeout: Duration,
    pub max_retries: usize,
    pub untranslated_text: bool,
    pub term_repo_id_list: Vec<i64>,
    pub sent_repo_id_list: Vec<i64>,
}
```

### 7. TranslationService

**位置**：`src/translator/service.rs`

**职责**：
- 高级翻译服务
- 智能分批和合并
- 统计收集

**关键功能**：
```rust
pub struct TranslationService {
    batch_translator: Arc<BatchTranslator>,
}

impl TranslationService {
    pub async fn translate_units(
        &self,
        units: &[TranslationUnit],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<TranslationUnit>> {
        // 1. 提取文本
        let texts: Vec<String> = units.iter()
            .map(|u| u.content.clone())
            .collect();

        // 2. 批量翻译
        let translations = self.batch_translator
            .translate_batch(&texts, source_lang, target_lang)
            .await?;

        // 3. 更新翻译单元
        let mut result = Vec::new();
        for (unit, translation) in units.iter().zip(translations) {
            let mut translated = unit.clone();
            translated.translation = Some(translation);
            result.push(translated);
        }

        Ok(result)
    }
}
```

## 技术选型

### HTTP 客户端
- **reqwest**：异步 HTTP 客户端
  - 异步/同步 API
  - 连接池
  - 代理支持
  - TLS 支持

### 速率限制
- **governor**：基于令牌桶的速率限制
  - 高性能
  - 灵活的配置
  - 异步友好

### 并发控制
- **Tokio Semaphore**：并发限制
  - 异步信号量
  - 公平调度
  - 零成本抽象

### 重试逻辑
- **自定义重试**：指数退避
  - 可配置的重试次数
  - 动态延迟
  - 失败策略

## 关键设计要点

### 1. 批量处理

```rust
fn group_by_size(&self, texts: &[String], max_size: usize) -> Vec<Vec<String>> {
    let mut groups = Vec::new();
    let mut current_group = Vec::new();
    let mut current_size = 0;

    for text in texts {
        if current_size + text.len() > max_size && !current_group.is_empty() {
            groups.push(std::mem::take(&mut current_group));
            current_size = 0;
        }
        current_group.push(text.clone());
        current_size += text.len();
    }

    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}
```

### 2. 速率限制

```rust
async fn wait_rate_limit(&self) {
    if let Some(limiter) = &self.rate_limiter {
        limiter.until_ready().await;
    }
}
```

### 3. 健康检查

```rust
impl TranslatorEntry {
    fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Relaxed) == 1
    }

    fn mark_healthy(&self) {
        self.healthy.store(1, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
    }

    fn mark_unhealthy(&self) {
        self.healthy.store(0, Ordering::Relaxed);
        self.failure_count.fetch_add(1, Ordering::Relaxed);
    }
}
```

### 4. 提供商选择

```rust
async fn select_healthy_translator(&self) -> Result<&Arc<TranslatorImpl>> {
    let healthy_translators: Vec<_> = self.translators
        .iter()
        .filter(|t| t.is_healthy())
        .collect();

    if healthy_translators.is_empty() {
        return Err(TranslateError::Translator(
            "No healthy translators available".to_string(),
        ));
    }

    // 简单的轮询策略
    let index = self.next_translator.fetch_add(1, Ordering::Relaxed) % healthy_translators.len();
    Ok(healthy_translators[index])
}
```

### 5. 错误处理

```rust
pub enum TranslateError {
    Network(String),
    Api(String),
    RateLimit(String),
    Auth(String),
    Timeout(String),
    InvalidInput(String),
    ProviderUnavailable(String),
}
```

### 6. 统计收集

```rust
impl BatchTranslator {
    async fn translate_with_stats(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let start = Instant::now();

        match self.translate_batch(texts, source_lang, target_lang).await {
            Ok(result) => {
                let latency = start.elapsed().as_millis() as u64;
                let chars: usize = texts.iter().map(|t| t.len()).sum();

                if let Some(stats) = &self.shared_stats {
                    stats.record_translator_call(
                        translator_name,
                        latency,
                        true,
                        chars,
                    );
                }

                Ok(result)
            }
            Err(e) => {
                if let Some(stats) = &self.shared_stats {
                    stats.record_translator_call(
                        translator_name,
                        0,
                        false,
                        0,
                    );
                }

                Err(e)
            }
        }
    }
}
```

## 使用示例

### 创建翻译服务

```rust
use codebase_translate::translator::{
    create_translation_service,
    GlobalConfig,
    ProjectConfig,
};

let global_config = GlobalConfig::from_file(".translate/config.toml")?;
let project_config = ProjectConfig::from_file(".translate.toml")?;

let service = create_translation_service(&global_config, &project_config)?;
```

### 批量翻译

```rust
let texts = vec![
    "Hello, world!".to_string(),
    "How are you?".to_string(),
];

let translations = service.translate_batch(&texts, "en", "zh").await?;
```

### 翻译单元

```rust
let units = vec![
    TranslationUnit {
        content: "Hello, world!".to_string(),
        ..
    },
    TranslationUnit {
        content: "How are you?".to_string(),
        ..
    },
];

let translated = service.translate_units(&units, "en", "zh").await?;
```

## 性能考量

1. **并发控制**：
   - 信号量限制并发数
   - 避免过多连接
   - 合理的默认值

2. **批量优化**：
   - 按字符数分组
   - 最小化 API 调用
   - 批量大小自适应

3. **连接池**：
   - 复用 HTTP 连接
   - 减少 TCP 握手
   - 提高吞吐量

4. **内存效率**：
   - 流式处理大文件
   - 避免克隆
   - 使用 Arc 共享

## 扩展性

1. **新的提供商**：
   - Google Translate
   - Microsoft Translator
   - 自定义 API

2. **高级策略**：
   - 基于成本的提供商选择
   - 基于质量的提供商选择
   - A/B 测试

3. **缓存增强**：
   - 翻译结果缓存
   - 分布式缓存
   - 缓存预热

4. **监控和告警**：
   - 性能监控
   - 错误率监控
   - 自动告警