# DeepLX 翻译器

DeepLX 是基于 DeepL 的免费翻译服务。

## 特性

- 完全免费使用
- 支持多种语言对
- 翻译质量较高
- 无需 API 密钥（本地实例）
- 响应速度快

## 支持的语言

### 源语言

- AUTO（自动检测）
- EN（英语）
- ZH（中文）
- JA（日语）
- KO（韩语）
- DE（德语）
- FR（法语）
- ES（西班牙语）
- IT（意大利语）
- PT（葡萄牙语）
- RU（俄语）

### 目标语言

- EN（英语）
- ZH（中文）
- JA（日语）
- KO（韩语）
- DE（德语）
- FR（法语）
- ES（西班牙语）
- IT（意大利语）
- PT（葡萄牙语）
- RU（俄语）

## 配置示例

```toml
[deeplx]
api_url = "${DEEPLX_API_URL}"
api_key = "${DEEPLX_API_KEY}"
proxy_url = "${DEEPLX_PROXY_URL}"
rate_limit = 10
max_retries = 3
```

## 配置参数

| 参数 | 说明 | 默认值 | 必填 |
|------|------|--------|------|
| api_url | API 地址 | https://api.deeplx.org | 否 |
| api_key | API 密钥（可选） | 空 | 否 |
| proxy_url | 代理 URL | 空 | 否 |
| rate_limit | 每秒请求数 | 10 | 否 |
| max_retries | 最大重试次数 | 3 | 否 |

## 使用限制

- **最大输入字符数**: 5000 字符
- **建议分割大小**: 4000 字符
- **推荐速率限制**: 10 请求/秒

## 使用建议

### 1. 选择合适的 API URL

默认使用 `https://api.deeplx.org`，你也可以：

- 使用本地 DeepLX 实例：`http://localhost:1188`
- 使用其他公开的 DeepLX 服务

### 2. 设置合理的速率限制

```toml
[deeplx]
rate_limit = 10  # 每秒 10 个请求
```

### 3. 使用代理（如需要）

```toml
[deeplx]
proxy_url = "http://proxy.example.com:8080"
```

## 注意事项

### 优点

1. **免费**: 完全免费使用，无费用
2. **简单**: 配置简单，无需复杂的认证
3. **质量高**: 翻译质量较好，特别是技术文档
4. **快速**: 响应速度快，适合大批量翻译

### 缺点

1. **可用性**: 依赖第三方服务，可能不稳定
2. **法律风险**: 使用免费的 DeepL API 可能存在法律风险
3. **功能有限**: 不支持术语库、自定义翻译等功能

### 最佳实践

1. **测试可用性**: 定期测试 API 是否可用

```bash
translator validate
```

2. **合理限流**: 不要设置过高的速率限制，避免被封

3. **准备备选**: 配置其他翻译器作为备选

```toml
enabled_providers = ["deeplx", "llm", "tencent"]
```

4. **监控错误**: 关注日志中的错误信息

```bash
translator translate --log-level debug
```

## 故障排查

### 连接失败

**问题**: 无法连接到 DeepLX API

**解决方案**:
1. 检查网络连接
2. 尝试其他 API URL
3. 检查代理设置
4. 查看详细日志

```bash
translator translate --log-level debug
```

### 翻译失败

**问题**: 返回错误信息

**解决方案**:
1. 检查输入文本长度（不超过 5000 字符）
2. 检查语言代码是否正确
3. 查看错误日志

### 速率限制

**问题**: 被限流

**解决方案**:
1. 降低 `rate_limit` 配置
2. 增加 `max_retries` 数量
3. 减少并发数

```bash
translator translate . --concurrency 2
```

## 性能优化

### 1. 批量翻译

```toml
[translate]
batch_size = 50
concurrency = 10
```

### 2. 使用缓存

```toml
[cache]
enabled = true
format = "binary"
```

### 3. 调整速率限制

```toml
[deeplx]
rate_limit = 15  # 根据实际情况调整
```

## 与其他翻译器对比

| 特性 | DeepLX | LLM | 腾讯云 |
|------|--------|-----|--------|
| 费用 | 免费 | 按使用量收费 | 500万字符/月免费 |
| 质量 | 高 | 可配置 | 高 |
| 速度 | 快 | 慢 | 快 |
| 功能 | 基础 | 丰富 | 丰富 |
| 稳定性 | 中等 | 高 | 高 |

## 示例

### 基础使用

```bash
# 使用 DeepLX 翻译
translator translate . --provider deeplx --target-lang en
```

### 自定义配置

```bash
# 使用本地 DeepLX 实例
DEEPLX_API_URL=http://localhost:1188 translator translate . --provider deeplx
```

### 限流配置

```toml
[deeplx]
api_url = "https://api.deeplx.org"
rate_limit = 5        # 降低速率限制
max_retries = 5       # 增加重试次数
```

## 相关文档

- [配置指南](../user-guide/configuration.md)
- [LLM 翻译器](llm.md)
- [腾讯云翻译器](tencent.md)
- [翻译器选择指南](provider-selection.md)