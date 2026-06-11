# LLM 翻译器

LLM 翻译器支持多种大语言模型提供商，提供智能路由和负载均衡功能。

## 特性

- 支持多个 LLM 提供商
- 智能路由和负载均衡
- 健康检查和自动故障转移
- 多模型轮询
- 自定义提示词
- 权重配置

## 支持的提供商

官方支持的提供商：

- SiliconFlow
- Zhipu（智谱）
- OpenAI
- Anthropic
- 其他兼容 OpenAI API 的提供商

## 配置示例

```toml
# 启用 LLM 翻译器
enabled_providers = ["llm"]

[llm.health_check]
enabled = true
interval = 30
timeout = 5
failure_threshold = 3
recovery_interval = 60

# SiliconFlow 提供商
[[llm.providers]]
id = "silicon"
name = "Siliconflow"
model_list = [
    "tencent/Hunyuan-MT-7B",
    "THUDM/GLM-4-9B-0414",
    "Qwen/Qwen2.5-7B-Instruct"
]
max_tokens = 8192
temperature = 0.3
base_url = "https://api.siliconflow.cn/v1"
api_keys = ["${SILCON_API_KEY}"]
rate_limit = 40
weight = 50

# Zhipu 提供商
[[llm.providers]]
id = "zhipu"
name = "Zhipu"
model = "glm-4-flash"
max_tokens = 4096
temperature = 0.3
base_url = "https://open.bigmodel.cn/api/paas/v4"
api_keys = ["${ZHIPU_API_KEY}"]
rate_limit = 5
weight = 20
```

## 配置参数

### 健康检查配置

| 参数 | 说明 | 默认值 |
|------|------|--------|
| enabled | 是否启用健康检查 | true |
| interval | 健康检查间隔（秒） | 30 |
| timeout | 健康检查超时（秒） | 5 |
| failure_threshold | 连续失败次数标记为不可用 | 3 |
| recovery_interval | 恢复检查间隔（秒） | 60 |

### 提供商配置

| 参数 | 说明 | 默认值 | 必填 |
|------|------|--------|------|
| id | 提供商唯一标识 | - | 是 |
| name | 提供商名称 | - | 是 |
| model_list | 模型列表（多模型轮询） | [] | 否 |
| model | 单一模型名称 | - | 否* |
| max_tokens | 最大 token 数 | 4096 | 是 |
| temperature | 温度参数 (0.0 - 2.0) | 0.3 | 否 |
| base_url | API 基础 URL | - | 是 |
| api_keys | API 密钥列表 | [] | 是 |
| proxy_url | 代理 URL | 空 | 否 |
| timeout | 超时时间（秒） | 30 | 否 |
| rate_limit | 每秒请求数限制 | 10 | 否 |
| weight | 权重（用于负载均衡） | 10 | 否 |
| extra_headers | 额外请求头 | {} | 否 |
| extra_params | 额外请求参数 | {} | 否 |
| custom_system_prompt | 自定义系统提示词 | - | 否 |
| custom_user_prompt | 自定义用户提示词 | - | 否 |

**注**: `model_list` 和 `model` 至少需要配置一个。如果 `model_list` 不为空，会为每个模型创建独立的提供商实例进行轮询。

## 路由策略

### 短文本路由（< 容量阈值）

- 使用权重轮询策略
- 在所有健康的提供商之间分配负载
- 根据权重比例分配请求

### 长文本路由（>= 容量阈值）

- 只在能处理长文本的提供商之间分配
- 使用权重轮询策略
- 确保文本不会超出提供商的容量限制

## 健康检查

### 检查机制

- 定期发送健康检查请求
- 连续失败 N 次后标记为不健康
- 不健康的提供商不会接收新请求
- 定期尝试恢复不健康的提供商

### 配置建议

```toml
[llm.health_check]
enabled = true
interval = 30           # 每 30 秒检查一次
timeout = 5             # 检查超时 5 秒
failure_threshold = 3   # 连续失败 3 次标记为不可用
recovery_interval = 60  # 每 60 秒尝试恢复一次
```

## 自定义提示词

### 系统提示词

```toml
[[llm.providers]]
id = "custom"
custom_system_prompt = "You are a professional translator specializing in technical documentation. Preserve technical terms and code structure."
```

### 用户提示词模板

```toml
[[llm.providers]]
id = "custom"
custom_user_prompt = "Translate from {source_lang} to {target_lang}:\n\n{text}"
```

支持的占位符：
- `{source_lang}`: 源语言
- `{target_lang}`: 目标语言
- `{text}`: 待翻译文本

## 使用建议

### 1. 配置多个提供商

```toml
[[llm.providers]]
id = "silicon"
weight = 50

[[llm.providers]]
id = "zhipu"
weight = 30

[[llm.providers]]
id = "openai"
weight = 20
```

### 2. 使用多模型轮询

```toml
[[llm.providers]]
id = "multi_model"
model_list = [
    "model1",
    "model2",
    "model3"
]
```

### 3. 根据文本长度配置容量

```toml
[[llm.providers]]
id = "small"
max_tokens = 4096
weight = 30

[[llm.providers]]
id = "large"
max_tokens = 16384
weight = 70
```

### 4. 启用健康检查

```toml
[llm.health_check]
enabled = true
```

## 注意事项

### 优点

1. **灵活**: 支持多种提供商和模型
2. **可靠**: 健康检查和自动故障转移
3. **智能**: 智能路由和负载均衡
4. **可定制**: 自定义提示词和参数

### 缺点

1. **成本**: 按使用量收费
2. **速度**: 响应速度相对较慢
3. **复杂性**: 配置相对复杂

### 最佳实践

1. **合理配置权重**: 根据提供商的性能和成本配置权重

2. **启用健康检查**: 确保自动故障转移正常工作

3. **多提供商冗余**: 配置多个提供商避免单点故障

4. **监控使用情况**: 定期检查 API 使用量和成本

5. **优化提示词**: 根据项目特点优化提示词

## 故障排查

### 提供商不可用

**问题**: 提供商被标记为不可用

**解决方案**:
1. 检查 API 密钥是否正确
2. 检查网络连接
3. 检查 API 配额是否用尽
4. 查看健康检查日志

```bash
translator translate --log-level debug
```

### 翻译质量不佳

**问题**: 翻译质量不符合预期

**解决方案**:
1. 调整温度参数（0.0 - 2.0）
2. 优化提示词
3. 尝试不同的模型
4. 检查源文本格式

### 速率限制

**问题**: 被 API 限流

**解决方案**:
1. 降低 `rate_limit` 配置
2. 配置多个提供商
3. 增加 API 密钥数量

```toml
[[llm.providers]]
api_keys = [
    "${API_KEY_1}",
    "${API_KEY_2}",
    "${API_KEY_3}"
]
```

## 性能优化

### 1. 配置合理的并发数

```toml
[translate]
concurrency = 5  # 根据提供商数量调整
```

### 2. 使用批量翻译

```toml
[translate]
batch_size = 50
```

### 3. 启用缓存

```toml
[cache]
enabled = true
format = "binary"
```

### 4. 优化健康检查频率

```toml
[llm.health_check]
interval = 60  # 减少检查频率
```

## 成本控制

### 1. 使用更便宜的模型

```toml
[[llm.providers]]
model = "cheaper-model"  # 选择成本更低的模型
```

### 2. 配置合理的速率限制

```toml
[[llm.providers]]
rate_limit = 10  # 控制请求速率
```

### 3. 使用缓存避免重复翻译

```toml
[cache]
enabled = true
```

## 与其他翻译器对比

| 特性 | LLM | DeepLX | 腾讯云 |
|------|-----|--------|--------|
| 费用 | 按使用量收费 | 免费 | 500万字符/月免费 |
| 质量 | 可配置 | 高 | 高 |
| 速度 | 慢 | 快 | 快 |
| 功能 | 最丰富 | 基础 | 丰富 |
| 稳定性 | 高 | 中等 | 高 |

## 示例

### 基础使用

```bash
# 使用 LLM 翻译
translator translate . --provider llm --target-lang en
```

### 多提供商配置

```bash
# 配置多个提供商后，工具会自动进行负载均衡
translator translate . --provider llm
```

### 自定义提示词

```toml
[[llm.providers]]
id = "custom"
custom_system_prompt = "You are a technical translator."
custom_user_prompt = "Translate: {text}"
```

## 相关文档

- [配置指南](../user-guide/configuration.md)
- [DeepLX 翻译器](deeplx.md)
- [腾讯云翻译器](tencent.md)
- [翻译器选择指南](provider-selection.md)