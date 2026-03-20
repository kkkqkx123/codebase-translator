# Reporter 模块翻译器 API 调用统计增强方案

## 1. 当前功能分析

### 1.1 现有统计能力

当前 [TranslationStats](../src/reporter/stats.rs#L18-L38) 结构包含以下统计信息：

- ✅ **时间统计**: 开始时间、结束时间、总耗时、平均速度
- ✅ **文件统计**: 总数、已处理、跳过、失败
- ✅ **翻译单元统计**: 总数、已翻译
- ✅ **缓存统计**: 命中次数、未命中次数、命中率
- ⚠️ **API 调用统计**: 只有 `api_call_count`（总次数），**无法区分翻译器类型**
- ✅ **错误统计**: 错误计数和详细错误记录

### 1.2 翻译器架构分析

项目支持三种翻译器，架构如下：

#### 1.2.1 DeepLX 翻译器
- **文件位置**: [src/translator/deeplx.rs](../src/translator/deeplx.rs)
- **特点**: 单一翻译器，无子分类
- **配置**: [DeepLXConfig](../src/config/global.rs#L642-L659)
  - `api_url`: API 地址
  - `api_key`: API 密钥（可选）
  - `proxy_url`: 代理地址
  - `rate_limit`: 速率限制
  - `max_retries`: 最大重试次数

#### 1.2.2 LLM 翻译器
- **文件位置**: [src/translator/llm/](../src/translator/llm/)
- **特点**: 多 provider 架构，支持多个供应商和模型
- **核心组件**:
  - `LLMTranslator`: 基础翻译器实现
  - `LLMProvider`: Provider 包装器，包含健康跟踪
  - `ProviderRouter`: 基于容量的加权路由
  - `MultiProviderTranslator`: 多 provider 翻译器
  - `ProviderPool`: Provider 池管理
- **配置**: [LLMProviderConfig](../src/config/global.rs#L717-L764)
  - `id`: Provider 唯一标识符
  - `name`: Provider 人类可读名称
  - `weight`: 容量路由权重
  - `base_url`: API 基础 URL
  - `api_keys`: API 密钥列表（支持轮换）
  - `model`: 模型名称（单个，优先）
  - `model_list`: 模型列表（多模型轮换）
  - `max_tokens`: 每次请求最大 token 数
  - `temperature`: 温度参数
  - `proxy_url`: 代理地址
  - `timeout`: 请求超时
  - `rate_limit`: 速率限制
  - `extra_headers`: 额外请求头
  - `extra_params`: 额外参数

#### 1.2.3 LLM Provider 统计
- **文件位置**: [src/translator/llm/provider.rs](../src/translator/llm/provider.rs#L25-L38)
- **现有统计结构** [ProviderStats](../src/translator/llm/provider.rs#L25-L38):
  ```rust
  pub struct ProviderStats {
      pub total_requests: u64,
      pub successful_requests: u64,
      pub failed_requests: u64,
      pub total_tokens: u64,
      pub average_latency_ms: f64,
      pub last_request_time: Option<Instant>,
  }
  ```
- **问题**: 该统计结构已存在，但**未暴露给 reporter 模块**

#### 1.2.4 Tencent 翻译器
- **文件位置**: [src/translator/tencent.rs](../src/translator/tencent.rs)
- **特点**: 单一翻译器，无子分类
- **配置**: [TencentConfig](../src/config/global.rs#L769-L809)
  - `secret_id`: 密钥 ID
  - `secret_key`: 密钥
  - `region`: 区域
  - `project_id`: 项目 ID
  - `endpoint`: 端点
  - `proxy_url`: 代理地址
  - `timeout`: 超时
  - `rate_limit`: 速率限制
  - `max_retries`: 最大重试次数

### 1.3 当前 API 调用统计的问题

在 [src/workflow/file_processor.rs:269](../src/workflow/file_processor.rs#L269) 中：

```rust
if let Some(ref reporter) = self.reporter {
    reporter.report_api_call(1);
}
```

**存在的问题**:
- ❌ 无法区分是哪个翻译器（DeepLX/LLM/Tencent）
- ❌ 对于 LLM，无法区分是哪个 provider
- ❌ 对于 LLM，无法区分是哪个模型
- ❌ 无法追踪每个翻译器的成功率
- ❌ 无法追踪每个翻译器的延迟等性能指标
- ❌ LLM Provider 已有统计但未暴露给 reporter

---

## 2. 需要补充的功能

### 2.1 翻译器级别的 API 调用统计 🔥 高优先级

#### 功能需求
- 按翻译器类型（DeepLX/LLM/Tencent）统计调用次数
- 记录每个翻译器的成功/失败次数
- 记录每个翻译器的平均延迟
- 记录每个翻译器处理的字符数
- 记录最后一次调用时间

#### 数据结构设计

在 `src/reporter/stats.rs` 中添加：

```rust
/// 翻译器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatorStats {
    /// 翻译器类型
    pub translator_type: String,
    /// 总调用次数
    pub total_calls: usize,
    /// 成功调用次数
    pub successful_calls: usize,
    /// 失败调用次数
    pub failed_calls: usize,
    /// 处理的总字符数
    pub total_chars: usize,
    /// 平均延迟（毫秒）
    pub average_latency_ms: f64,
    /// 最后一次调用时间
    pub last_call_time: Option<DateTime<Utc>>,
    /// 最小延迟（毫秒）
    pub min_latency_ms: Option<f64>,
    /// 最大延迟（毫秒）
    pub max_latency_ms: Option<f64>,
}

impl TranslatorStats {
    pub fn new(translator_type: String) -> Self {
        Self {
            translator_type,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            total_chars: 0,
            average_latency_ms: 0.0,
            last_call_time: None,
            min_latency_ms: None,
            max_latency_ms: None,
        }
    }

    pub fn record_call(&mut self, latency_ms: u64, success: bool, chars: usize) {
        self.total_calls += 1;
        self.total_chars += chars;
        self.last_call_time = Some(Utc::now());

        let latency = latency_ms as f64;

        if success {
            self.successful_calls += 1;

            let total_latency = self.average_latency_ms * (self.successful_calls - 1) as f64;
            self.average_latency_ms = (total_latency + latency) / self.successful_calls as f64;

            if let Some(min) = self.min_latency_ms {
                self.min_latency_ms = Some(min.min(latency));
            } else {
                self.min_latency_ms = Some(latency);
            }

            if let Some(max) = self.max_latency_ms {
                self.max_latency_ms = Some(max.max(latency));
            } else {
                self.max_latency_ms = Some(latency);
            }
        } else {
            self.failed_calls += 1;
        }
    }
}
```

### 2.2 LLM Provider 级别的统计 🔥 高优先级

#### 功能需求
- 按 provider id 统计调用次数
- 按 provider model 统计调用次数
- 记录每个 provider 的成功/失败次数
- 记录每个 provider 的平均延迟
- 记录每个 provider 处理的字符数
- 记录最后一次调用时间

#### 数据结构设计

在 `src/reporter/stats.rs` 中添加：

```rust
/// LLM Provider 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMProviderStats {
    /// Provider ID
    pub provider_id: String,
    /// Provider 名称
    pub provider_name: String,
    /// 模型名称
    pub model: String,
    /// 总调用次数
    pub total_calls: usize,
    /// 成功调用次数
    pub successful_calls: usize,
    /// 失败调用次数
    pub failed_calls: usize,
    /// 处理的总字符数
    pub total_chars: usize,
    /// 平均延迟（毫秒）
    pub average_latency_ms: f64,
    /// 最后一次调用时间
    pub last_call_time: Option<DateTime<Utc>>,
    /// 最小延迟（毫秒）
    pub min_latency_ms: Option<f64>,
    /// 最大延迟（毫秒）
    pub max_latency_ms: Option<f64>,
}

impl LLMProviderStats {
    pub fn new(provider_id: String, provider_name: String, model: String) -> Self {
        Self {
            provider_id,
            provider_name,
            model,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            total_chars: 0,
            average_latency_ms: 0.0,
            last_call_time: None,
            min_latency_ms: None,
            max_latency_ms: None,
        }
    }

    pub fn record_call(&mut self, latency_ms: u64, success: bool, chars: usize) {
        self.total_calls += 1;
        self.total_chars += chars;
        self.last_call_time = Some(Utc::now());

        let latency = latency_ms as f64;

        if success {
            self.successful_calls += 1;

            let total_latency = self.average_latency_ms * (self.successful_calls - 1) as f64;
            self.average_latency_ms = (total_latency + latency) / self.successful_calls as f64;

            if let Some(min) = self.min_latency_ms {
                self.min_latency_ms = Some(min.min(latency));
            } else {
                self.min_latency_ms = Some(latency);
            }

            if let Some(max) = self.max_latency_ms {
                self.max_latency_ms = Some(max.max(latency));
            } else {
                self.max_latency_ms = Some(latency);
            }
        } else {
            self.failed_calls += 1;
        }
    }
}
```

### 2.3 扩展 TranslationStats 🔥 高优先级

#### 新增字段

在 `src/reporter/stats.rs` 中修改 `TranslationStats` 结构：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationStats {
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_files: usize,
    pub processed_files: usize,
    pub skipped_files: usize,
    pub failed_files: usize,
    pub total_units: usize,
    pub translated_units: usize,
    pub api_call_count: usize,
    pub error_count: usize,
    pub cache_hit_count: usize,
    pub cache_miss_count: usize,
    pub errors: Vec<ErrorRecord>,
    pub total_duration_ms: u64,
    pub avg_speed_files_per_sec: f64,

    // 新增字段
    /// 翻译器级别统计（按翻译器类型）
    pub translator_stats: HashMap<String, TranslatorStats>,
    /// LLM Provider 统计
    pub llm_provider_stats: HashMap<String, LLMProviderStats>,
}
```

#### 新增方法

```rust
impl TranslationStats {
    // ... 现有方法 ...

    pub fn record_translator_call(
        &mut self,
        translator_type: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        let stats = self
            .translator_stats
            .entry(translator_type.to_string())
            .or_insert_with(|| TranslatorStats::new(translator_type.to_string()));
        stats.record_call(latency_ms, success, chars);
    }

    pub fn record_llm_provider_call(
        &mut self,
        provider_id: &str,
        provider_name: &str,
        model: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        let stats = self
            .llm_provider_stats
            .entry(provider_id.to_string())
            .or_insert_with(|| LLMProviderStats::new(
                provider_id.to_string(),
                provider_name.to_string(),
                model.to_string(),
            ));
        stats.record_call(latency_ms, success, chars);
    }

    pub fn get_translator_stats(&self, translator_type: &str) -> Option<&TranslatorStats> {
        self.translator_stats.get(translator_type)
    }

    pub fn get_llm_provider_stats(&self, provider_id: &str) -> Option<&LLMProviderStats> {
        self.llm_provider_stats.get(provider_id)
    }

    pub fn get_all_translator_stats(&self) -> Vec<&TranslatorStats> {
        self.translator_stats.values().collect()
    }

    pub fn get_all_llm_provider_stats(&self) -> Vec<&LLMProviderStats> {
        self.llm_provider_stats.values().collect()
    }
}
```

### 2.4 扩展 Reporter Trait 🔥 高优先级

#### 新增方法

在 `src/reporter/trait.rs` 中添加：

```rust
pub trait Reporter: Send + Sync {
    // ... 现有方法 ...

    /// Report API call with translator type
    ///
    /// # Arguments
    /// * `translator_type` - 翻译器类型 (deeplx/llm/tencent)
    /// * `count` - 调用次数
    /// * `latency_ms` - 延迟（毫秒）
    /// * `success` - 是否成功
    /// * `chars` - 处理的字符数
    fn report_api_call_by_translator(
        &self,
        translator_type: &str,
        count: usize,
        latency_ms: u64,
        success: bool,
        chars: usize,
    );

    /// Report LLM provider API call
    ///
    /// # Arguments
    /// * `provider_id` - Provider ID
    /// * `provider_name` - Provider 名称
    /// * `model` - 模型名称
    /// * `latency_ms` - 延迟（毫秒）
    /// * `success` - 是否成功
    /// * `chars` - 处理的字符数
    fn report_llm_provider_call(
        &self,
        provider_id: &str,
        provider_name: &str,
        model: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    );
}
```

### 2.5 在翻译器中集成统计 🔥 高优先级

#### 2.5.1 DeepLX 翻译器

修改 `src/translator/deeplx.rs`:

```rust
impl DeepLXTranslator {
    async fn translate_single_internal(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let start_time = std::time::Instant::now();

        // ... 现有翻译逻辑 ...

        let result = self
            .client
            .post(&api_url)
            // ... 请求构建 ...
            .send()
            .await
            .map_err(|e| TranslateError::Http(e.to_string()))?;

        let latency = start_time.elapsed().as_millis() as u64;

        // ... 响应处理 ...

        let response = self
            .client
            .post(&api_url)
            // ... 请求构建 ...
            .send()
            .await
            .map_err(|e| TranslateError::Http(e.to_string()))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| TranslateError::Http(e.to_string()))?;

        if !status.is_success() {
            error!(
                status = %status,
                response_body = %response_text,
                "DeepLX API error"
            );
            return Err(TranslateError::Translation(format!(
                "DeepLX API error: {} - {}",
                status, response_text
            )));
        }

        let deeplx_resp: DeepLXResponse = serde_json::from_str(&response_text).map_err(|e| {
            error!(
                error = %e,
                response_body = %response_text,
                "Failed to parse DeepLX response"
            );
            TranslateError::Parse(format!(
                "Failed to parse DeepLX response: {} - {}",
                e, response_text
            ))
        })?;

        Ok(TranslateResponse {
            original_text: text.to_string(),
            translated_text: deeplx_resp.data,
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            latency_ms: Some(latency),
            translator_type: Some("deeplx".to_string()),
            ..Default::default()
        })
    }
}
```

#### 2.5.2 LLM Provider

修改 `src/translator/llm/provider.rs`:

```rust
impl LLMProvider {
    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let start_time = Instant::now();

        let result = self
            .translator
            .translate_single(text, source_lang, target_lang)
            .await;

        let latency = start_time.elapsed();

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.last_request_time = Some(Instant::now());

        match &result {
            Ok(resp) => {
                stats.successful_requests += 1;
                let total_latency = stats.average_latency_ms * (stats.total_requests - 1) as f64;
                stats.average_latency_ms =
                    (total_latency + latency.as_millis() as f64) / stats.total_requests as f64;
            }
            Err(_) => {
                stats.failed_requests += 1;
            }
        }

        match result {
            Ok(translated_text) => Ok(TranslateResponse {
                original_text: text.to_string(),
                translated_text,
                source_lang: source_lang.to_string(),
                target_lang: target_lang.to_string(),
                latency_ms: Some(latency.as_millis() as u64),
                translator_type: Some("llm".to_string()),
                provider_id: Some(self.id.clone()),
                provider_name: Some(self.translator.name().to_string()),
                model: Some(self.translator.model().to_string()),
                ..Default::default()
            }),
            Err(e) => Err(e),
        }
    }
}
```

#### 2.5.3 Tencent 翻译器

修改 `src/translator/tencent.rs`:

```rust
impl TencentTranslator {
    async fn translate_single_internal(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let start_time = std::time::Instant::now();

        // ... 现有翻译逻辑 ...

        let response = self
            .client
            .post(API_URL)
            // ... 请求构建 ...
            .send()
            .await
            .map_err(|e| TranslateError::Http(e.to_string()))?;

        let latency = start_time.elapsed().as_millis() as u64;

        // ... 响应处理 ...

        Ok(TranslateResponse {
            original_text: text.to_string(),
            translated_text: tencent_resp.response.target_text,
            source_lang: tencent_resp.response.source,
            target_lang: tencent_resp.response.target,
            latency_ms: Some(latency),
            translator_type: Some("tencent".to_string()),
            ..Default::default()
        })
    }
}
```

#### 2.5.4 扩展 TranslateResponse

在 `src/translator/common.rs` 中修改 `TranslateResponse`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateResponse {
    pub original_text: String,
    pub translated_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub alternatives: Vec<String>,

    // 新增字段
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 翻译器类型
    pub translator_type: Option<String>,
    /// Provider ID (仅 LLM)
    pub provider_id: Option<String>,
    /// Provider 名称 (仅 LLM)
    pub provider_name: Option<String>,
    /// 模型名称 (仅 LLM)
    pub model: Option<String>,
}
```

### 2.6 更新报告生成 📊 中优先级

#### 2.6.1 文本报告格式

修改 `src/reporter/default.rs` 中的 `generate_text_report` 方法：

```rust
fn generate_text_report(
    &self,
    stats: &crate::reporter::stats::TranslationStats,
) -> Result<String, TranslateError> {
    // ... 现有报告内容 ...

    report.push_str("API Calls:\n");
    report.push_str(&format!("  Total:      {}\n", stats.api_call_count));

    // 新增：按翻译器类型统计
    if !stats.translator_stats.is_empty() {
        report.push_str("\n  By Translator:\n");
        for (translator_type, translator_stats) in &stats.translator_stats {
            report.push_str(&format!(
                "    {}: {} calls ({} success, {} failed, avg: {:.0}ms, chars: {})\n",
                translator_type,
                translator_stats.total_calls,
                translator_stats.successful_calls,
                translator_stats.failed_calls,
                translator_stats.average_latency_ms,
                translator_stats.total_chars
            ));
        }
    }

    // 新增：LLM Provider 统计
    if !stats.llm_provider_stats.is_empty() {
        report.push_str("\n  LLM Providers:\n");
        for (provider_id, provider_stats) in &stats.llm_provider_stats {
            report.push_str(&format!(
                "    {} ({}):\n",
                provider_stats.provider_name, provider_id
            ));
            report.push_str(&format!(
                "      Model: {}\n",
                provider_stats.model
            ));
            report.push_str(&format!(
                "      Calls: {} ({} success, {} failed)\n",
                provider_stats.total_calls,
                provider_stats.successful_calls,
                provider_stats.failed_calls
            ));
            report.push_str(&format!(
                "      Latency: avg {:.0}ms, min {:.0}ms, max {:.0}ms\n",
                provider_stats.average_latency_ms,
                provider_stats.min_latency_ms.unwrap_or(0.0),
                provider_stats.max_latency_ms.unwrap_or(0.0)
            ));
            report.push_str(&format!(
                "      Chars: {}\n",
                provider_stats.total_chars
            ));
        }
    }

    report.push('\n');
}
```

#### 2.6.2 JSON 报告格式

JSON 报告会自动序列化新增的字段，格式如下：

```json
{
  "start_time": "2024-01-01T00:00:00Z",
  "end_time": "2024-01-01T01:00:00Z",
  "total_files": 100,
  "processed_files": 95,
  "skipped_files": 3,
  "failed_files": 2,
  "total_units": 1000,
  "translated_units": 950,
  "api_call_count": 350,
  "error_count": 2,
  "cache_hit_count": 50,
  "cache_miss_count": 50,
  "total_duration_ms": 3600000,
  "avg_speed_files_per_sec": 0.0264,
  "translator_stats": {
    "deeplx": {
      "translator_type": "deeplx",
      "total_calls": 100,
      "successful_calls": 95,
      "failed_calls": 5,
      "total_chars": 50000,
      "average_latency_ms": 150.5,
      "last_call_time": "2024-01-01T01:00:00Z",
      "min_latency_ms": 100.0,
      "max_latency_ms": 300.0
    },
    "llm": {
      "translator_type": "llm",
      "total_calls": 200,
      "successful_calls": 195,
      "failed_calls": 5,
      "total_chars": 100000,
      "average_latency_ms": 800.2,
      "last_call_time": "2024-01-01T01:00:00Z",
      "min_latency_ms": 500.0,
      "max_latency_ms": 1500.0
    },
    "tencent": {
      "translator_type": "tencent",
      "total_calls": 50,
      "successful_calls": 48,
      "failed_calls": 2,
      "total_chars": 25000,
      "average_latency_ms": 200.3,
      "last_call_time": "2024-01-01T01:00:00Z",
      "min_latency_ms": 150.0,
      "max_latency_ms": 400.0
    }
  },
  "llm_provider_stats": {
    "openai-gpt4": {
      "provider_id": "openai-gpt4",
      "provider_name": "OpenAI GPT-4",
      "model": "gpt-4",
      "total_calls": 120,
      "successful_calls": 118,
      "failed_calls": 2,
      "total_chars": 60000,
      "average_latency_ms": 750.0,
      "last_call_time": "2024-01-01T01:00:00Z",
      "min_latency_ms": 500.0,
      "max_latency_ms": 1200.0
    },
    "anthropic-claude": {
      "provider_id": "anthropic-claude",
      "provider_name": "Anthropic Claude",
      "model": "claude-3-opus",
      "total_calls": 80,
      "successful_calls": 77,
      "failed_calls": 3,
      "total_chars": 40000,
      "average_latency_ms": 870.5,
      "last_call_time": "2024-01-01T01:00:00Z",
      "min_latency_ms": 600.0,
      "max_latency_ms": 1500.0
    }
  }
}
```

### 2.7 性能指标增强 📊 中优先级

#### 2.7.1 延迟分布统计

在 `TranslatorStats` 和 `LLMProviderStats` 中添加：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatorStats {
    // ... 现有字段 ...

    /// P50 延迟（毫秒）
    pub p50_latency_ms: Option<f64>,
    /// P95 延迟（毫秒）
    pub p95_latency_ms: Option<f64>,
    /// P99 延迟（毫秒）
    pub p99_latency_ms: Option<f64>,
    /// 所有延迟记录（用于计算百分位数）
    #[serde(skip)]
    latency_records: Vec<f64>,
}

impl TranslatorStats {
    pub fn record_call(&mut self, latency_ms: u64, success: bool, chars: usize) {
        // ... 现有逻辑 ...

        if success {
            self.latency_records.push(latency);

            if self.latency_records.len() >= 10 {
                self.update_percentiles();
            }
        }
    }

    fn update_percentiles(&mut self) {
        let mut sorted = self.latency_records.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = sorted.len();
        if len > 0 {
            self.p50_latency_ms = Some(sorted[len * 50 / 100]);
            self.p95_latency_ms = Some(sorted[len * 95 / 100]);
            self.p99_latency_ms = Some(sorted[len * 99 / 100]);
        }
    }
}
```

#### 2.7.2 吞吐量统计

在 `TranslationStats` 中添加：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationStats {
    // ... 现有字段 ...

    /// 总吞吐量（字符/秒）
    pub total_throughput_chars_per_sec: f64,
    /// 按翻译器类型的吞吐量
    pub translator_throughput: HashMap<String, f64>,
}

impl TranslationStats {
    pub fn finalize(&mut self) {
        // ... 现有逻辑 ...

        if self.total_duration_ms > 0 {
            let total_chars: usize = self.translator_stats.values()
                .map(|s| s.total_chars)
                .sum();

            self.total_throughput_chars_per_sec =
                (total_chars as f64) / (self.total_duration_ms as f64 / 1000.0);

            for (translator_type, stats) in &self.translator_stats {
                let throughput = (stats.total_chars as f64) / (self.total_duration_ms as f64 / 1000.0);
                self.translator_throughput.insert(translator_type.clone(), throughput);
            }
        }
    }
}
```

#### 2.7.3 错误率统计

在 `TranslatorStats` 和 `LLMProviderStats` 中添加：

```rust
impl TranslatorStats {
    pub fn error_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        (self.failed_calls as f64 / self.total_calls as f64) * 100.0
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        (self.successful_calls as f64 / self.total_calls as f64) * 100.0
    }
}
```

---

## 3. 实现优先级总结

### 🔥 高优先级（立即实现）

1. **扩展 TranslationStats**
   - 添加 `translator_stats: HashMap<String, TranslatorStats>`
   - 添加 `llm_provider_stats: HashMap<String, LLMProviderStats>`
   - 实现 `TranslatorStats` 和 `LLMProviderStats` 结构
   - 实现统计记录方法

2. **扩展 Reporter trait**
   - 添加 `report_api_call_by_translator` 方法
   - 添加 `report_llm_provider_call` 方法
   - 在所有 Reporter 实现中添加对应方法

3. **在翻译器中集成统计**
   - 修改 `DeepLXTranslator` 记录延迟和翻译器类型
   - 修改 `LLMProvider` 记录延迟、provider 信息
   - 修改 `TencentTranslator` 记录延迟和翻译器类型
   - 扩展 `TranslateResponse` 添加统计字段

4. **更新报告生成**
   - 在文本报告中显示按翻译器类型的统计
   - 在文本报告中显示 LLM Provider 详细统计
   - JSON 报告自动包含新字段

### 📊 中优先级（近期实现）

5. **性能指标增强**
   - 添加 P50/P95/P99 延迟统计
   - 添加吞吐量统计
   - 添加错误率/成功率计算
   - 在报告中显示性能指标

6. **实时统计显示**
   - 在进度条中显示当前使用的翻译器
   - 显示翻译器切换事件
   - 显示实时延迟信息

7. **统计导出**
   - 支持将详细统计导出为 CSV
   - 支持将详细统计导出为 JSON
   - 支持自定义导出格式

### 💡 低优先级（未来考虑）

8. **统计可视化**
   - 生成延迟分布图
   - 生成翻译器使用占比图
   - 生成错误率趋势图

9. **历史统计对比**
   - 保存历史统计数据
   - 对比不同运行周期的统计
   - 显示趋势分析

10. **成本估算**
    - 根据翻译器类型和调用次数估算成本
    - 支持 DeepLX、OpenAI、Anthropic 等不同计费模式
    - 生成成本报告

---

## 4. 关键设计原则

### 4.1 向后兼容
- 新增统计字段不影响现有功能
- 保留原有的 `report_api_call` 方法
- 默认实现可以返回空统计

### 4.2 性能优先
- 统计收集不应显著影响翻译性能
- 使用 `Arc<RwLock<>>` 保证线程安全
- 延迟记录使用轻量级数据结构
- 百分位数计算可以采样而非记录所有数据

### 4.3 类型安全
- 使用枚举 `ProviderType` 避免字符串硬编码
- 使用结构体而非 HashMap 表示统计信息
- 提供类型安全的访问方法

### 4.4 可扩展性
- 易于添加新的翻译器类型
- 易于添加新的统计指标
- 支持自定义统计收集器

### 4.5 测试覆盖
- 为所有新增方法编写单元测试
- 测试线程安全性
- 测试统计准确性
- 测试边界情况

---

## 5. 实现步骤

### 阶段 1: 数据结构扩展
1. 在 `src/reporter/stats.rs` 中添加 `TranslatorStats` 结构
2. 在 `src/reporter/stats.rs` 中添加 `LLMProviderStats` 结构
3. 在 `TranslationStats` 中添加新字段
4. 实现统计记录和查询方法
5. 编写单元测试

### 阶段 2: Trait 扩展
1. 在 `src/reporter/trait.rs` 中添加新方法
2. 在 `DefaultReporter` 中实现新方法
3. 在 `ProgressReporter` 中实现新方法（如果存在）
4. 编写集成测试

### 阶段 3: 翻译器集成
1. 扩展 `TranslateResponse` 添加统计字段
2. 修改 `DeepLXTranslator` 记录统计
3. 修改 `LLMProvider` 记录统计
4. 修改 `TencentTranslator` 记录统计
5. 编写翻译器测试

### 阶段 4: 报告生成
1. 修改 `generate_text_report` 显示详细统计
2. 验证 JSON 报告格式
3. 编写报告生成测试

### 阶段 5: 工作流集成
1. 修改 `FileProcessor` 调用新的统计方法
2. 修改 `BatchTranslator` 传递统计信息
3. 端到端测试

### 阶段 6: 性能增强（可选）
1. 实现延迟分布统计
2. 实现吞吐量统计
3. 实现错误率计算
4. 优化性能

---

## 6. 测试计划

### 6.1 单元测试
- `TranslatorStats` 的所有方法
- `LLMProviderStats` 的所有方法
- `TranslationStats` 的新方法
- 统计计算的准确性

### 6.2 集成测试
- Reporter trait 新方法的实现
- 翻译器统计记录的正确性
- 多线程环境下的统计准确性

### 6.3 端到端测试
- 完整翻译流程的统计收集
- 报告生成的正确性
- 不同翻译器组合的统计

### 6.4 性能测试
- 统计收集对翻译性能的影响
- 大量调用时的统计准确性
- 内存使用情况

---

## 7. 预期效果

实施此方案后，用户将能够：

1. **了解翻译器使用情况**: 知道每个翻译器被调用了多少次
2. **评估翻译器性能**: 比较不同翻译器的延迟和成功率
3. **优化翻译器配置**: 根据统计数据调整权重和路由策略
4. **监控翻译成本**: 根据调用次数估算成本
5. **诊断问题**: 通过错误率和延迟信息定位问题
6. **生成详细报告**: 导出详细的统计数据用于分析

---

## 8. 相关文件清单

### 需要修改的文件
- `src/reporter/stats.rs` - 添加新的统计结构
- `src/reporter/trait.rs` - 扩展 Reporter trait
- `src/reporter/default.rs` - 实现新的统计方法
- `src/reporter/progress.rs` - 实现新的统计方法（如果存在）
- `src/translator/common.rs` - 扩展 TranslateResponse
- `src/translator/deeplx.rs` - 集成统计记录
- `src/translator/llm/provider.rs` - 集成统计记录
- `src/translator/tencent.rs` - 集成统计记录
- `src/workflow/file_processor.rs` - 使用新的统计方法
- `src/translator/batch.rs` - 传递统计信息

### 需要添加的测试文件
- `tests/reporter_stats_tests.rs` - 统计结构测试
- `tests/reporter_integration_tests.rs` - Reporter 集成测试（已存在，需扩展）
- `tests/translator_stats_tests.rs` - 翻译器统计测试

---

## 9. 风险和注意事项

### 9.1 性能风险
- **风险**: 统计收集可能影响翻译性能
- **缓解**: 使用轻量级数据结构，采样而非记录所有数据

### 9.2 线程安全风险
- **风险**: 多线程环境下统计可能不一致
- **缓解**: 使用 `Arc<RwLock<>>` 保证线程安全

### 9.3 内存使用风险
- **风险**: 记录所有延迟可能消耗大量内存
- **缓解**: 限制记录数量，使用采样或流式计算

### 9.4 向后兼容风险
- **风险**: 新增字段可能破坏现有代码
- **缓解**: 使用 `Option` 类型，提供默认值

---

## 10. 总结

本方案详细分析了当前 reporter 模块的统计能力，识别了缺少翻译器级别 API 调用统计的问题，并提出了完整的解决方案。通过扩展数据结构、trait 和翻译器实现，用户将能够获得详细的翻译器使用统计，包括调用次数、成功率、延迟等关键指标，从而更好地优化翻译配置和监控翻译质量。

实施此方案将显著提升 reporter 模块的实用性，使其成为更强大的监控和分析工具。
