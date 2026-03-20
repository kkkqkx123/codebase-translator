# 翻译器重试机制与速率限制集成方案

## 问题背景

### 当前架构问题

**配置层级现状：**
```
GlobalConfig (全局配置)
├── deeplx
│   ├── max_retries  ← 存在，但仅用于传递到 DeepLXConfig
│   └── rate_limit   ← 存在，有验证逻辑，但未传递到任何地方
├── tencent
│   ├── max_retries  ← 存在，但仅用于传递到 TencentConfig
│   └── rate_limit   ← 存在，有验证逻辑，但未传递到任何地方
└── limits
    └── （缺少 max_retries）
```

**翻译器配置现状：**
```
DeepLXConfig / LLMConfig / TencentConfig
├── max_retries  ← 存在，但翻译器内部从未使用
└── 其他字段... ← 正常使用
```

**实际调用链路：**
```
factory::create_translator()
  → TranslationService::new()
    → TranslatorImpl (直接调用)
      → 各翻译器 (无重试、无速率限制)
```

### 核心问题

1. **配置字段存在但未使用**
   - `deeplx.rate_limit` 和 `tencent.rate_limit` 有验证逻辑但从未被使用
   - 各翻译器的 `max_retries` 字段存在但从未被使用
   - 配置验证在 `global.rs` 中，但实际调用时未传递到 `BatchTranslator`

2. **重试机制未生效**
   - `BatchTranslator` 实现了完整的重试机制（指数退避：1s × 2^attempt）
   - `MultiTranslator` 实现了跨翻译器故障转移
   - 但这些功能在实际工作流程中从未被调用

3. **速率限制未生效**
   - `BatchTranslator` 使用 `governor` 库实现了速率限制
   - 但 `TranslationService` 直接调用 `TranslatorImpl`，绕过了 `BatchTranslator`

4. **配置验证与实际使用不一致**
   - `global.rs` 中有 `rate_limit` 的验证逻辑
   - 但验证后的配置从未传递到实际使用的组件

---

## 正确的集成方案

### 设计目标

- 每个翻译器支持独立的重试次数配置
- 每个翻译器支持独立的速率限制配置
- 配置正确传递到 `BatchTranslator`
- 实现精准的速率控制

### 实施方案

#### 1. 修改工厂函数，创建 BatchOptions

**文件：** `src/factory/mod.rs`

```rust
pub fn create_translator(
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
) -> Result<TranslationService> {
    info!(
        provider = %project_config.translate.provider,
        "Creating translator instance"
    );

    // 创建翻译器配置
    let translator_config = match project_config.translate.provider {
        ProviderType::DeepLX => {
            TranslatorConfig {
                provider: ProviderType::DeepLX,
                deeplx: Some(crate::translator::common::DeepLXConfig {
                    api_url: global_config.deeplx.api_url.clone(),
                    api_key: global_config.deeplx.api_key.clone(),
                    proxy_url: global_config.deeplx.proxy_url.clone(),
                    max_retries: global_config.deeplx.max_retries as usize,
                }),
                llm: None,
                tencent: None,
            }
        }
        ProviderType::LLM => {
            // LLM 配置
            TranslatorConfig {
                provider: ProviderType::LLM,
                deeplx: None,
                llm: Some(/* LLM 配置 */),
                tencent: None,
            }
        }
        ProviderType::Tencent => {
            TranslatorConfig {
                provider: ProviderType::Tencent,
                deeplx: None,
                llm: None,
                tencent: Some(crate::translator::common::TencentConfig {
                    secret_id: global_config.tencent.secret_id.clone().unwrap_or_default(),
                    secret_key: global_config.tencent.secret_key.clone().unwrap_or_default(),
                    region: global_config.tencent.region.clone(),
                    project_id: global_config.tencent.project_id as i64,
                    proxy_url: global_config.tencent.proxy_url.clone(),
                    timeout: global_config.tencent.timeout,
                    max_retries: global_config.tencent.max_retries as usize,
                    untranslated_text: global_config.tencent.untranslated_text.clone(),
                    term_repo_id_list: global_config.tencent.term_repo_id_list.clone(),
                    sent_repo_id_list: global_config.tencent.sent_repo_id_list.clone(),
                }),
            }
        }
    };

    // 创建翻译器
    let translator_impl = crate::translator::factory::create_translator_from_config(&translator_config)?;

    // 根据不同的 provider 创建 BatchOptions
    let batch_options = crate::translator::common::BatchOptions {
        rate_limit: match project_config.translate.provider {
            ProviderType::DeepLX => global_config.deeplx.rate_limit,
            ProviderType::LLM => global_config.limits.rate_limit,
            ProviderType::Tencent => global_config.tencent.rate_limit,
        },
        workers: 5,  // 可以从配置读取
        max_retries: match project_config.translate.provider {
            ProviderType::DeepLX => global_config.deeplx.max_retries as usize,
            ProviderType::LLM => 3,  // LLM 使用默认值
            ProviderType::Tencent => global_config.tencent.max_retries as usize,
        },
        limit_policy: Some(match project_config.translate.provider {
            ProviderType::DeepLX => crate::translator::deeplx::default_limit_policy(),
            ProviderType::LLM => crate::translator::common::LimitPolicy::default(),
            ProviderType::Tencent => crate::translator::tencent::default_limit_policy(),
        }),
    };

    // 创建 BatchTranslator
    let batch_translator = crate::translator::BatchTranslator::new(
        Arc::new(translator_impl),
        batch_options,
    );

    // 创建使用 BatchTranslator 的 TranslationService
    TranslationService::with_batch_translator(Arc::new(batch_translator))
}
```

#### 2. 修改 TranslationService 支持 BatchTranslator

**文件：** `src/translator/service.rs`

```rust
pub struct TranslationService {
    runtime: tokio::runtime::Runtime,
    batch_translator: Option<Arc<BatchTranslator>>,
    translator: Option<Arc<TranslatorImpl>>,
}

impl TranslationService {
    /// 保留原有方法用于向后兼容
    pub fn new(config: TranslatorConfig) -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new()?;
        let translator = Arc::new(create_translator_from_config(&config)?);
        Ok(Self {
            runtime,
            batch_translator: None,
            translator: Some(translator),
        })
    }

    /// 新增：使用 BatchTranslator（带重试和速率限制）
    pub fn with_batch_translator(batch_translator: Arc<BatchTranslator>) -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new()?;
        Ok(Self {
            runtime,
            batch_translator: Some(batch_translator),
            translator: None,
        })
    }

    pub fn translate_batch(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>> {
        if let Some(ref batch_translator) = self.batch_translator {
            // 使用 BatchTranslator（带重试和速率限制）
            let result = self.runtime.block_on(async {
                batch_translator.translate_batch(texts, target_lang).await
            })?;
            Ok(result.results.into_iter().map(|r| r.translated_text).collect())
        } else if let Some(ref translator) = self.translator {
            // 回退到直接调用（向后兼容）
            let texts = texts.to_vec();
            let target_lang = target_lang.to_string();
            let translator = translator.clone();
            let result = self.runtime.block_on(async move {
                translator.translate(&texts, &target_lang).await
            })?;
            Ok(result)
        } else {
            Err(TranslateError::Translation("No translator configured".to_string()))
        }
    }

    // 其他方法需要适配 batch_translator...
}
```

#### 3. 配置文件保持不变

**文件：** `translator.toml`

```toml
[deeplx]
api_url = "${DEEPLX_API_URL}"
api_key = "${DEEPLX_API_KEY}"
proxy_url = "${DEEPLX_PROXY_URL}"
rate_limit = 10      # DeepLX 的速率限制（请求/秒）
max_retries = 3      # DeepLX 的重试次数

[tencent]
secret_id = "${TENCENT_SECRET_ID}"
secret_key = "${TENCENT_SECRET_KEY}"
region = "ap-guangzhou"
proxy_url = ""
timeout = 30
rate_limit = 5       # Tencent 的速率限制（腾讯云要求不超过5次/秒）
max_retries = 3      # Tencent 的重试次数
```

---

## 配置映射关系

| 配置来源 | 目标组件 | 用途 | 默认值 |
|---------|---------|------|--------|
| `deeplx.rate_limit` | `BatchOptions.rate_limit` | DeepLX 速率限制 | 10 |
| `deeplx.max_retries` | `BatchOptions.max_retries` | DeepLX 重试次数 | 3 |
| `tencent.rate_limit` | `BatchOptions.rate_limit` | Tencent 速率限制 | 5 |
| `tencent.max_retries` | `BatchOptions.max_retries` | Tencent 重试次数 | 3 |
| `limits.rate_limit` | `BatchOptions.rate_limit` | LLM 速率限制 | 10 |

---

## 重试机制说明

### BatchTranslator 的重试逻辑

**指数退避算法：**
```
重试次数    延迟时间
1          1s
2          2s
3          4s
4          8s
...
```

**实现代码：**
```rust
for attempt in 0..self.max_retries {
    match translator.translate(texts).await {
        Ok(translated) => return Ok(translated),
        Err(e) => {
            if attempt < self.max_retries - 1 {
                let delay = Duration::from_millis(1000 * 2_u64.pow(attempt as u32));
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

### MultiTranslator 的故障转移

**跨翻译器重试：**
- 主翻译器失败后，尝试其他可用的翻译器
- 受 `max_retries` 限制
- 自动标记不健康的翻译器

---

## 速率限制说明

### 使用 governor 库

**基于令牌桶算法的速率限制：**
```rust
let quota = Quota::per_second(
    NonZeroU32::new(rate_limit.max(1)).expect("max(1) is always non-zero"),
);
let rate_limiter = RateLimiter::direct(quota);
```

**并发控制：**
- 使用 `Semaphore` 控制并发 worker 数量
- 默认 5 个并发 worker

### API 要求

**腾讯云要求：**
```
单个接口每秒请求次数不高于 5 次
```

因此 `tencent.rate_limit` 默认值为 5，并在验证中强制检查：
```rust
if self.tencent.rate_limit == 0 || self.tencent.rate_limit > 5 {
    return Err("Tencent rate_limit must be between 1 and 5".to_string());
}
```

---

## 关键优势

### 1. 精准控制
- 每个翻译器有独立的速率限制和重试配置
- 可以根据不同 API 的要求调整参数

### 2. 符合 API 要求
- 腾讯云要求每秒不超过 5 次，可以精确控制
- 其他 API 有类似要求时可以灵活配置

### 3. 配置统一
- 所有配置通过 GlobalConfig 管理
- 配置验证在启动时完成，避免运行时错误

### 4. 向后兼容
- 保留原有的 `TranslationService::new()` 方法
- 如果未使用 BatchTranslator，仍可正常工作

### 5. 功能完整
- 重试机制：指数退避，避免瞬时故障
- 速率限制：防止触发 API 限制
- 并发控制：平衡性能和资源使用

---

## 实施检查清单

### 代码修改

- [ ] 修改 `src/factory/mod.rs` 中的 `create_translator` 函数
- [ ] 修改 `src/translator/service.rs` 中的 `TranslationService` 结构
- [ ] 添加 `TranslationService::with_batch_translator` 方法
- [ ] 更新 `translate_batch` 方法支持 BatchTranslator
- [ ] 更新其他 `TranslationService` 方法适配新结构

### 测试验证

- [ ] 验证 DeepLX 的重试机制生效
- [ ] 验证 Tencent 的速率限制生效
- [ ] 验证 LLM 的默认配置正常工作
- [ ] 验证配置验证逻辑正确
- [ ] 验证向后兼容性

### 配置文件

- [ ] 确认 `translator.toml` 中的配置正确
- [ ] 确认 `bin/translator.toml` 中的配置正确
- [ ] 添加配置说明注释

---

## 注意事项

### 1. 性能影响
- 速率限制可能会影响整体翻译速度
- 需要根据实际情况调整 `rate_limit` 和 `workers` 参数

### 2. 错误处理
- `BatchResult` 包含错误信息，需要适当处理
- 部分失败时，需要决定是继续还是终止

### 3. 配置验证
- 确保 `rate_limit` 在合理范围内
- 确保 `max_retries` 不超过上限（建议 10 次）

### 4. 日志记录
- 记录重试次数和延迟
- 记录速率限制的影响
- 便于性能调优和问题排查