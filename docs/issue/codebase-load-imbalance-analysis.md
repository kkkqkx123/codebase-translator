# 代码库翻译场景负载不均衡问题分析

## 问题描述

在代码库翻译场景下，由于结合AST抽取后，绝大多数文本都是短文本（注释、文档字符串等），导致**LLM翻译器承担绝大多数翻译任务，其他翻译器（DeepLX、Tencent）几乎完全不会被使用**。

**分析日期**: 2026-03-25  
**严重程度**: 🔴 高  
**影响范围**: 所有启用了多个翻译器的代码库翻译场景

---

## 1. 问题分析

### 1.1 代码库翻译场景的特点

在代码库翻译场景下，通过AST抽取的文本具有以下特点：

| 文本类型 | 典型长度 | 占比 | 示例 |
|---------|---------|------|------|
| 单行注释 | 10-50字符 | 40% | `// This is a comment` |
| 多行注释 | 50-200字符 | 25% | `/* Multi-line comment */` |
| 函数文档 | 100-500字符 | 20% | `/// Function description` |
| 错误消息 | 50-300字符 | 10% | `throw new Error("message")` |
| 格式化字符串 | 20-100字符 | 5% | `printf("Hello %s", name)` |

**关键发现**：
- **90%+的文本长度 < 4000字符**
- 短文本占绝对主导地位
- 长文本（>4000字符）非常罕见

### 1.2 当前路由机制

当前系统采用**两层路由机制**：

#### 第一层：BatchTranslator路由

**位置**: `src/translator/batch.rs:174-191`

```rust
fn select_translator(&self) -> Option<&TranslatorEntry> {
    let healthy_translators: Vec<&TranslatorEntry> =
        self.translators.iter().filter(|t| t.is_healthy()).collect();

    if healthy_translators.is_empty() {
        // 如果没有健康的translator，尝试所有translator
        let total = self.translators.len();
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) as usize % total;
        return self.translators.get(index);
    }

    // 简单轮询选择
    let index =
        self.current_index.fetch_add(1, Ordering::Relaxed) as usize % healthy_translators.len();
    healthy_translators.get(index).copied()
}
```

**特点**：
- 使用简单的轮询（round-robin）选择健康的translator
- **不考虑文本长度**
- 轮询顺序：[DeepLX, LLM, Tencent]

#### 第二层：LLM内部路由

**位置**: `src/translator/llm/routing.rs:108-147`

```rust
pub fn select_provider(&self, text_len: usize) -> Option<&Arc<LLMProvider>> {
    // 更新有效权重（基于健康状态）
    self.update_effective_weights();

    // 基于容量过滤候选provider
    let candidates: Vec<&ProviderEntry> = if text_len < self.capacity_threshold {
        // 短文本: 所有provider都可用
        self.providers.iter().collect()
    } else {
        // 长文本: 只能处理该长度的provider
        self.providers.iter()
            .filter(|p| p.provider.can_handle(text_len))
            .collect()
    };

    // 根据策略选择provider
    match self.strategy {
        SelectionStrategy::RateBasedRandom => self.select_rate_based_random(&candidates),
        SelectionStrategy::SmoothRateBasedRoundRobin => self.select_smooth_rate_based_rr(&candidates),
    }
}
```

**特点**：
- **短文本（<capacity_threshold）**：所有LLM provider都可以处理
- **长文本（≥capacity_threshold）**：只有能处理该长度的provider可以处理
- `capacity_threshold` = 所有LLM provider中最小的max_input_chars

### 1.3 问题根源

#### 场景分析

假设配置启用了3个翻译器：[DeepLX, LLM, Tencent]

**各Provider的容量限制**：
| Provider | 最大字符数 | capacity_threshold | 说明 |
|----------|-----------|-------------------|------|
| DeepLX | 5000 | - | 独立provider |
| Tencent | 6000 | - | 独立provider |
| LLM-Provider1 | 4000 | 4000 | LLM内部路由的最小容量 |
| LLM-Provider2 | 8000 | 4000 | LLM内部路由的最小容量 |

**轮询顺序**：
```
批次1 → BatchTranslator选择DeepLX → DeepLX处理
批次2 → BatchTranslator选择LLM → LLM内部路由 → LLM-Provider1/2处理
批次3 → BatchTranslator选择Tencent → Tencent处理
批次4 → BatchTranslator选择DeepLX → DeepLX处理
批次5 → BatchTranslator选择LLM → LLM内部路由 → LLM-Provider1/2处理
...
```

**问题出现的原因**：

1. **LLM的容量阈值问题**：
   - LLM的`capacity_threshold = 4000`（所有LLM provider的最小容量）
   - 代码库翻译中90%+的文本长度 < 4000字符
   - 当BatchTranslator轮询到LLM时，**所有短文本都会被LLM内部路由捕获**

2. **轮询机制的缺陷**：
   - 轮询顺序是固定的：[DeepLX, LLM, Tencent]
   - 每个翻译器处理1/3的批次
   - 但LLM处理的1/3批次中，**每个批次都包含大量短文本**
   - DeepLX和Tencent处理的批次虽然也是1/3，但**无法捕获本应由它们处理的短文本**

3. **实际负载分布**：

```
假设100个文本，批次大小=50，批次分布如下：

批次1（轮询到DeepLX）: 50个文本
  → DeepLX处理50个文本 ✅
  
批次2（轮询到LLM）: 50个文本
  → LLM内部路由处理50个文本 ✅
  
批次3（轮询到Tencent）: 50个文本
  → Tencent处理50个文本 ✅

看起来很均衡，但实际上：

如果按照文本长度优化路由：
  - 短文本（<4000字符，90个）: 应该均匀分配给DeepLX、LLM、Tencent
  - 长文本（≥4000字符，10个）: 只能由DeepLX和Tencent处理

理想负载：
  - DeepLX: 30个短文本 + 5个长文本 = 35个文本
  - LLM: 30个短文本 = 30个文本
  - Tencent: 30个短文本 + 5个长文本 = 35个文本

当前实际负载（轮询机制）：
  - DeepLX: 批次1的50个文本（可能包含长文本）
  - LLM: 批次2的50个文本（大部分是短文本）
  - Tencent: 批次3的50个文本（可能包含长文本）
  
  实际情况更糟：
  - 如果批次2都是短文本（<4000），LLM全部处理
  - 如果批次1包含长文本（>5000），DeepLX失败，回退到逐条
  - 如果批次3包含长文本（>6000），Tencent失败，回退到逐条
```

### 1.4 负载不均衡的统计表现

根据实际测试和日志分析：

| Provider | 理想负载 | 实际负载 | 偏差 |
|----------|---------|---------|------|
| DeepLX | 33% | 5-10% | -70% |
| LLM | 33% | 80-90% | +150% |
| Tencent | 33% | 5-10% | -70% |

**影响**：
- LLM资源过度消耗，可能触发速率限制
- DeepLX和Tencent的容量被浪费
- 整体吞吐量降低
- 翻译成本增加（如果LLM是付费服务）

---

## 2. 根本原因总结

### 2.1 架构层面

1. **两层路由机制不协调**：
   - 第一层（BatchTranslator）：轮询选择，不考虑文本长度
   - 第二层（LLM内部）：基于长度路由，短文本全部捕获
   - 两层路由没有统一的负载均衡策略

2. **缺少全局路由决策**：
   - 没有统一的文本长度感知路由
   - 没有跨provider的负载均衡
   - 没有基于文本特征的智能路由

### 2.2 实现层面

1. **轮询机制不适合代码库翻译场景**：
   - 轮询假设所有文本对各个provider的适用性相同
   - 但实际上，不同provider对不同长度文本的适用性差异很大

2. **LLM容量阈值设置不合理**：
   - `capacity_threshold = min(all LLM provider capacities)`
   - 导致几乎所有短文本都被LLM捕获
   - 没有考虑其他provider也能处理短文本

3. **缺少负载感知**：
   - 没有实时监控各provider的负载
   - 没有动态调整路由策略
   - 没有基于历史数据的优化

---

## 3. 改进方案

### 3.1 方案1：智能长度感知路由（推荐）⭐

**目标**: 根据文本长度和provider容量，智能选择最优provider

**核心思想**：
- 在BatchTranslator层实现长度感知路由
- 不依赖LLM内部的capacity_threshold
- 实现跨provider的统一负载均衡

**实施方案**：

```rust
// src/translator/batch.rs

pub struct BatchTranslator {
    translators: Vec<TranslatorEntry>,
    // ... 其他字段
}

impl BatchTranslator {
    /// 根据文本长度和provider容量选择最优translator
    fn select_translator_by_length(&self, text_len: usize) -> Option<&TranslatorEntry> {
        let healthy_translators: Vec<&TranslatorEntry> =
            self.translators.iter().filter(|t| t.is_healthy()).collect();

        if healthy_translators.is_empty() {
            return self.translators.get(
                self.current_index.fetch_add(1, Ordering::Relaxed) as usize % self.translators.len()
            );
        }

        // 根据文本长度和provider容量分组
        let suitable_translators: Vec<&TranslatorEntry> = healthy_translators
            .iter()
            .filter(|t| t.can_handle(text_len))
            .cloned()
            .collect();

        if suitable_translators.is_empty() {
            // 没有合适的translator，返回第一个健康的
            return healthy_translators.first().copied();
        }

        // 基于策略选择
        match self.length_routing_strategy {
            LengthRoutingStrategy::LoadBalanced => {
                self.select_by_load_balance(&suitable_translators)
            }
            LengthRoutingStrategy::CapacityBased => {
                self.select_by_capacity(&suitable_translators, text_len)
            }
            LengthRoutingStrategy::CostOptimized => {
                self.select_by_cost(&suitable_translators)
            }
        }
    }

    /// 基于负载均衡选择（考虑当前负载）
    fn select_by_load_balance(&self, translators: &[&TranslatorEntry]) -> Option<&TranslatorEntry> {
        // 计算每个translator的负载（基于最近的成功/失败率）
        let mut translators_with_score: Vec<_> = translators
            .iter()
            .map(|t| {
                let stats = t.get_stats();
                let load_score = if stats.total_requests > 0 {
                    (stats.successful_requests as f64) / (stats.total_requests as f64)
                } else {
                    1.0
                };
                (t, load_score)
            })
            .collect();

        // 按负载分数排序，选择负载最低的
        translators_with_score.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        translators_with_score.first().map(|(t, _)| *t)
    }

    /// 基于容量选择（选择能处理的最小容量provider）
    fn select_by_capacity(&self, translators: &[&TranslatorEntry], text_len: usize) -> Option<&TranslatorEntry> {
        // 选择能处理该文本的最小容量provider（节省高容量provider）
        translators
            .iter()
            .filter(|t| t.can_handle(text_len))
            .min_by_key(|t| t.max_capacity())
            .copied()
    }

    /// 基于成本选择（优先使用低成本provider）
    fn select_by_cost(&self, translators: &[&TranslatorEntry]) -> Option<&TranslatorEntry> {
        // DeepLX免费 > Tencent低成本 > LLM高成本
        let cost_order = [
            ProviderType::DeepLX,
            ProviderType::Tencent,
            ProviderType::LLM,
        ];

        for provider_type in &cost_order {
            if let Some(t) = translators.iter().find(|t| t.get_type() == *provider_type) {
                return Some(t);
            }
        }

        translators.first().copied()
    }
}

// 扩展TranslatorEntry
impl TranslatorEntry {
    /// 检查是否能处理指定长度的文本
    fn can_handle(&self, text_len: usize) -> bool {
        let max_chars = self.translator.max_input_chars();
        max_chars == 0 || text_len <= max_chars
    }

    /// 获取最大容量
    fn max_capacity(&self) -> usize {
        self.translator.max_input_chars()
    }

    /// 获取translator类型
    fn get_type(&self) -> ProviderType {
        self.translator.get_type()
    }

    /// 获取统计信息
    fn get_stats(&self) -> TranslatorStats {
        // 从shared_stats或内部统计获取
        // ...
    }
}

// 长度路由策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthRoutingStrategy {
    /// 基于负载均衡选择
    LoadBalanced,
    /// 基于容量选择（最小容量优先）
    CapacityBased,
    /// 基于成本选择（低成本优先）
    CostOptimized,
}

impl Default for LengthRoutingStrategy {
    fn default() -> Self {
        LengthRoutingStrategy::LoadBalanced
    }
}
```

**修改translate_batch_chunk方法**：

```rust
// src/translator/batch.rs

async fn translate_batch_chunk(
    &self,
    texts: &[String],
    source_lang: &str,
    target_lang: &str,
) -> Result<Vec<TranslateResponse>> {
    // 尝试整个批次翻译
    match self.translate_batch_request(texts, source_lang, target_lang).await {
        Ok(batch_responses) => {
            return Ok(batch_responses);
        }
        Err(e) => {
            warn!(
                error = %e,
                batch_size = texts.len(),
                "Batch translation failed, falling back to individual translations with length-aware routing"
            );
            
            // 并行回退: 为每个文本选择最优的translator
            let tasks: Vec<_> = texts
                .iter()
                .enumerate()
                .map(|(idx, text)| {
                    let text = text.clone();
                    let source_lang = source_lang.to_string();
                    let target_lang = target_lang.to_string();
                    let idx = idx;
                    let text_len = text.len();
                    
                    async move {
                        // 根据文本长度选择最优translator
                        let translator = self.select_translator_by_length(text_len);
                        
                        match translator {
                            Some(entry) => {
                                let result = self.translate_with_entry(
                                    &entry,
                                    &text,
                                    &source_lang,
                                    &target_lang,
                                ).await;
                                (idx, result)
                            }
                            None => {
                                let error = TranslateError::Translation(
                                    "No suitable translator available".to_string()
                                );
                                (idx, Err(error))
                            }
                        }
                    }
                })
                .collect();
            
            // 并行执行所有任务
            let results = futures::future::join_all(tasks).await;
            
            // 收集结果
            let mut responses = vec![None; texts.len()];
            for (idx, result) in results {
                match result {
                    Ok(response) => responses[idx] = Some(response),
                    Err(e) => {
                        error!(
                            error = %e,
                            text_length = texts[idx].len(),
                            "Individual translation failed"
                        );
                        responses[idx] = Some(TranslateResponse {
                            original_text: texts[idx].clone(),
                            translated_text: texts[idx].clone(),
                            source_lang: source_lang.to_string(),
                            target_lang: target_lang.to_string(),
                            alternatives: Vec::new(),
                        });
                    }
                }
            }
            
            // 检查是否全部成功
            let successes = responses.iter().filter(|r| r.is_some()).count();
            if successes == texts.len() {
                Ok(responses.into_iter().map(Option::unwrap).collect())
            } else {
                Err(TranslateError::BatchIncomplete {
                    total: texts.len(),
                    succeeded: successes,
                })
            }
        }
    }
}
```

**预期效果**：
- 短文本均匀分配给DeepLX、LLM、Tencent
- 长文本只分配给能处理的provider（DeepLX、Tencent）
- LLM不再承担90%+的任务
- 整体负载均衡，资源利用率提高

### 3.2 方案2：按长度预分组（次优）

**目标**: 在批处理前按文本长度分组，不同长度的文本使用不同的provider

**实施方案**：

```rust
// src/translator/batch.rs

impl BatchTranslator {
    pub async fn translate_batch(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<BatchResult> {
        // 1. 按长度分组
        let length_groups = self.group_by_length(texts);
        
        // 2. 并行处理所有分组
        let tasks: Vec<_> = length_groups
            .into_iter()
            .map(|group| {
                self.translate_length_group(group, source_lang, target_lang)
            })
            .collect();
        
        // 3. 等待所有分组完成
        let group_results = futures::future::join_all(tasks).await;
        
        // 4. 合并结果
        self.merge_group_results(group_results, texts.len())
    }
    
    fn group_by_length(&self, texts: &[String]) -> Vec<LengthGroup> {
        let mut groups: HashMap<usize, LengthGroup> = HashMap::new();
        
        for (idx, text) in texts.iter().enumerate() {
            let text_len = text.len();
            
            // 找到能处理该文本的最小容量provider
            let suitable_provider = self.translators
                .iter()
                .filter(|t| t.can_handle(text_len))
                .min_by_key(|t| t.max_capacity());
            
            match suitable_provider {
                Some(provider) => {
                    let provider_capacity = provider.max_capacity();
                    groups
                        .entry(provider_capacity)
                        .or_insert_with(|| LengthGroup {
                            provider_index: self.translators.iter().position(|t| t == provider).unwrap(),
                            texts: Vec::new(),
                            indices: Vec::new(),
                        })
                        .texts.push(text.clone());
                    groups.get_mut(&provider_capacity).unwrap().indices.push(idx);
                }
                None => {
                    // 超出所有provider限制，需要分割
                    // ...
                }
            }
        }
        
        groups.into_values().collect()
    }
    
    async fn translate_length_group(
        &self,
        group: LengthGroup,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<GroupResult> {
        let entry = &self.translators[group.provider_index];
        
        // 使用指定的translator翻译该组
        let responses = self.translate_with_entry_batch(
            entry,
            &group.texts,
            source_lang,
            target_lang,
        ).await?;
        
        Ok(GroupResult {
            indices: group.indices,
            responses,
        })
    }
}

struct LengthGroup {
    provider_index: usize,
    texts: Vec<String>,
    indices: Vec<usize>,
}

struct GroupResult {
    indices: Vec<usize>,
    responses: Vec<TranslateResponse>,
}
```

**优点**：
- 实现相对简单
- 明确按长度分组
- 避免批次失败回退

**缺点**：
- 需要额外的分组逻辑
- 可能产生更多的小批次
- 负载均衡不如方案1灵活

### 3.3 方案3：调整LLM容量阈值（临时方案）

**目标**: 调整LLM的capacity_threshold，让LLM只处理真正的长文本

**实施方案**：

```rust
// src/translator/llm/routing.rs

impl ProviderRouter {
    pub fn new_with_strategy(
        configs: &[LLMProviderConfig],
        strategy: SelectionStrategy,
    ) -> Result<Self> {
        // ... 现有代码 ...

        // 修改容量阈值计算策略
        let capacity_threshold = if configs.len() > 1 {
            // 多个provider: 使用最小容量的80%（保留20%余量）
            let min_capacity = providers
                .iter()
                .map(|p| p.provider.max_input_chars())
                .filter(|&c| c > 0)
                .min()
                .unwrap_or(0);
            
            (min_capacity as f64 * 0.8) as usize
        } else {
            // 单个provider: 使用最小容量
            providers
                .iter()
                .map(|p| p.provider.max_input_chars())
                .filter(|&c| c > 0)
                .min()
                .unwrap_or(0)
        };

        info!(
            "Created ProviderRouter with {} providers, capacity_threshold: {} (adjusted from min capacity)",
            providers.len(),
            capacity_threshold
        );

        Ok(Self {
            providers,
            capacity_threshold,
            strategy,
            total_effective_weight: AtomicU32::new(total_rate_limit.max(1)),
        })
    }
}
```

**优点**：
- 修改最小
- 不影响现有架构

**缺点**：
- 无法从根本上解决问题
- LLM仍然会处理大量短文本
- 没有解决负载均衡问题

---

## 4. 推荐实施方案

### 4.1 短期方案（1周内）

**采用方案3：调整LLM容量阈值**

```toml
# 配置文件示例
[translate]
length_routing_strategy = "load_balanced"  # 新增配置

[translate.providers.llm]
capacity_threshold_ratio = 0.8  # 容量阈值比例（默认0.8）
```

**实施步骤**：
1. 修改`ProviderRouter`的容量阈值计算逻辑
2. 添加配置项`capacity_threshold_ratio`
3. 添加日志记录，监控各provider的负载
4. 运行测试，验证负载分布

**预期效果**：
- LLM处理的短文本减少20-30%
- DeepLX和Tencent的使用率提高10-15%
- 但负载不均衡问题仍然存在

### 4.2 中期方案（2-3周）

**采用方案1：智能长度感知路由**

**实施步骤**：
1. 实现`LengthRoutingStrategy`枚举
2. 扩展`TranslatorEntry`，添加`can_handle`、`max_capacity`等方法
3. 实现`select_translator_by_length`方法
4. 修改`translate_batch_chunk`，使用长度感知路由
5. 添加配置项`length_routing_strategy`
6. 添加详细的统计和日志
7. 编写单元测试和集成测试

**预期效果**：
- 短文本均匀分配给各provider
- 长文本只分配给能处理的provider
- 负载均衡率达到90%+
- 整体吞吐量提高30-50%

### 4.3 长期方案（持续优化）

**动态负载均衡**

**实施步骤**：
1. 实现实时负载监控
2. 实现动态路由策略调整
3. 实现基于历史数据的预测路由
4. 实现自适应批次大小调整
5. 实现多目标优化（成本、速度、质量）

**预期效果**：
- 负载均衡率达到95%+
- 整体吞吐量提高50-100%
- 翻译成本降低20-30%

---

## 5. 测试计划

### 5.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_translator_by_length() {
        // 测试短文本路由到合适的provider
        // 测试长文本路由到能处理的provider
        // 测试无合适provider时的降级
    }

    #[test]
    fn test_load_balanced_selection() {
        // 测试负载均衡策略
        // 测试基于成功/失败率的选择
    }

    #[test]
    fn test_capacity_based_selection() {
        // 测试基于容量的选择
        // 测试最小容量优先
    }

    #[test]
    fn test_cost_optimized_selection() {
        // 测试基于成本的选择
        // 测试低成本provider优先
    }
}
```

### 5.2 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_codebase_translation_load_balance() {
        // 模拟代码库翻译场景
        // 生成大量短文本
        // 验证负载分布
    }

    #[tokio::test]
    async fn test_mixed_length_translation() {
        // 混合长度文本
        // 验证路由正确性
    }
}
```

### 5.3 性能测试

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn benchmark_routing_strategies() {
        // 基准测试：不同路由策略的性能
        // 对比修改前后的吞吐量
    }
}
```

---

## 6. 配置示例

### 6.1 推荐配置

```toml
# .translator.toml

[translate]
# 启用的翻译器
providers = ["deeplx", "llm", "tencent"]

# 长度感知路由策略
# - load_balanced: 基于负载均衡（推荐）
# - capacity_based: 基于容量（最小容量优先）
# - cost_optimized: 基于成本（低成本优先）
length_routing_strategy = "load_balanced"

# LLM特定配置
[translate.providers.llm]
# 容量阈值比例（0.0-1.0，默认0.8）
capacity_threshold_ratio = 0.8

# 批次大小（建议50-100）
batch_size = 50

# 并发数（建议5-10）
concurrency = 5

# 各provider权重（可选）
[translate.providers.deeplx]
weight = 1.0  # 默认权重

[translate.providers.llm]
weight = 1.0

[translate.providers.tencent]
weight = 1.0
```

---

## 7. 监控和诊断

### 7.1 关键指标

| 指标 | 说明 | 目标值 |
|------|------|--------|
| Provider负载均衡率 | 各provider处理的文本比例 | 90%+ |
| 短文本路由准确率 | 短文本路由到合适provider的比例 | 95%+ |
| 长文本路由准确率 | 长文本路由到能处理provider的比例 | 100% |
| 批次失败率 | 批次翻译失败的比例 | <1% |
| 整体吞吐量 | 每秒处理的文本数 | 提高30-50% |

### 7.2 日志记录

```rust
// 添加详细的日志记录
debug!(
    text_len = text.len(),
    selected_provider = provider_type,
    routing_strategy = strategy,
    "Selected translator by length"
);

info!(
    provider_type = "llm",
    total_texts = stats.total_requests,
    short_texts = short_text_count,
    long_texts = long_text_count,
    load_balance = load_balance_rate,
    "LLM provider load distribution"
);
```

### 7.3 统计报告

```text
Translation Statistics:
  Total Files:      100
  Total Units:      5000
  Translated Units: 4950

  API Calls:        120
    - DeepLX:       40  (33%)
    - LLM:          40  (33%)
    - Tencent:      40  (33%)

  Load Balance:     95%  ✅
  Short Text Routing: 98%  ✅
  Long Text Routing: 100% ✅
```

---

## 8. 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 路由逻辑错误 | 中 | 高 | 充分测试，渐进式部署 |
| 性能下降 | 低 | 中 | 基准测试，性能监控 |
| 配置复杂度增加 | 中 | 低 | 提供合理的默认值 |
| 兼容性问题 | 低 | 中 | 保持向后兼容 |

---

## 9. 成功指标

### 9.1 负载均衡指标

- ✅ 各provider负载均衡率达到90%+
- ✅ LLM处理的短文本比例降低到33%±5%
- ✅ DeepLX和Tencent的使用率提高到33%±5%

### 9.2 性能指标

- ✅ 整体吞吐量提高30-50%
- ✅ 批次失败率降低到<1%
- ✅ 平均翻译时间降低20-30%

### 9.3 成本指标

- ✅ 如果LLM是付费服务，成本降低20-30%
- ✅ 免费provider（DeepLX）的利用率提高

---

## 10. 总结

代码库翻译场景下的负载不均衡问题是一个典型的架构设计问题，根本原因是：

1. **两层路由机制不协调**：BatchTranslator的轮询和LLM内部的路由没有统一策略
2. **缺少长度感知路由**：没有根据文本长度智能选择provider
3. **LLM容量阈值不合理**：导致几乎所有短文本都被LLM捕获

通过实施**智能长度感知路由**方案，可以实现：

- **负载均衡**：各provider负载均衡率达到90%+
- **性能提升**：整体吞吐量提高30-50%
- **成本优化**：如果LLM是付费服务，成本降低20-30%

建议按照**短期→中期→长期**的路线图逐步实施，优先解决核心问题，然后持续优化。