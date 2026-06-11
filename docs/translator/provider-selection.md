# 翻译器选择指南

本指南帮助你根据项目需求选择合适的翻译器。

## 翻译器对比

| 特性 | DeepLX | LLM | 腾讯云 |
|------|--------|-----|--------|
| 费用 | 免费 | 按使用量收费 | 500万字符/月免费 |
| 翻译质量 | 高 | 可配置（低-高） | 高 |
| 响应速度 | 快 | 慢 | 快 |
| 功能 | 基础 | 丰富 | 丰富 |
| 稳定性 | 中等 | 高 | 高 |
| 认证 | 不需要 | 需要 | 需要 |
| 速率限制 | 自由 | 依赖提供商 | 5请求/秒 |
| 最大字符数 | 5000 | 依赖模型 | 6000 |
| 自定义 | 低 | 高 | 中 |
| 适用场景 | 个人项目 | 专业项目 | 商业项目 |

## 选择建议

### 场景 1: 个人项目或小团队

**推荐**: DeepLX

**理由**:
- 完全免费，无成本压力
- 翻译质量高，满足大多数需求
- 配置简单，无需 API 密钥

**注意事项**:
- 可能存在法律风险
- 依赖第三方服务，稳定性不确定

**配置示例**:
```toml
enabled_providers = ["deeplx"]

[deeplx]
api_url = "https://api.deeplx.org"
rate_limit = 10
```

### 场景 2: 中小型商业项目

**推荐**: 腾讯云

**理由**:
- 每月 500 万字符免费额度
- 稳定可靠，服务质量有保障
- 支持术语库和句库

**注意事项**:
- 需要配置 API 密钥
- 速率限制较低
- 可能在某些地区无法访问

**配置示例**:
```toml
enabled_providers = ["tencent"]

[tencent]
secret_id = "${TENCENT_SECRET_ID}"
secret_key = "${TENCENT_SECRET_KEY}"
rate_limit = 5
```

### 场景 3: 大型项目或专业需求

**推荐**: LLM（多提供商）

**理由**:
- 支持自定义提示词，可优化翻译质量
- 智能路由和负载均衡
- 健康检查和自动故障转移
- 支持多个模型和提供商

**注意事项**:
- 按使用量收费，需要控制成本
- 配置相对复杂
- 响应速度较慢

**配置示例**:
```toml
enabled_providers = ["llm"]

[llm.health_check]
enabled = true
interval = 30

[[llm.providers]]
id = "silicon"
model_list = ["model1", "model2"]
weight = 50

[[llm.providers]]
id = "zhipu"
model = "glm-4-flash"
weight = 30
```

### 场景 4: 高可靠性要求

**推荐**: 多翻译器组合

**理由**:
- 自动故障转移
- 负载均衡
- 提高可用性

**配置示例**:
```toml
enabled_providers = ["deeplx", "llm", "tencent"]

[deeplx]
rate_limit = 10

[llm.health_check]
enabled = true

[[llm.providers]]
id = "silicon"
weight = 30

[tencent]
rate_limit = 5
```

### 场景 5: 成本敏感型项目

**推荐**: 腾讯云 + DeepLX

**理由**:
- 优先使用腾讯云免费额度
- 腾讯云配额用尽后自动切换到 DeepLX

**配置示例**:
```toml
enabled_providers = ["tencent", "deeplx"]
```

## 性能考量

### 翻译速度

从快到慢：
1. DeepLX
2. 腾讯云
3. LLM

**建议**:
- 追求速度: DeepLX 或腾讯云
- 追求质量: LLM（可接受较慢速度）
- 平衡考虑: 腾讯云

### 批量处理

**大量文件翻译**:
- 使用 DeepLX 或腾讯云
- 配置合理的并发数
- 启用缓存

**配置示例**:
```toml
[translate]
batch_size = 100
concurrency = 10

[cache]
enabled = true
format = "binary"
```

## 质量考量

### 技术文档翻译

**推荐**: LLM（自定义提示词）

**理由**:
- 可以自定义提示词优化技术术语翻译
- 支持多模型选择

**配置示例**:
```toml
[[llm.providers]]
id = "tech-docs"
custom_system_prompt = "You are a technical translator specializing in software documentation. Preserve technical terms and code structure. Use consistent terminology."
```

### 通用注释翻译

**推荐**: DeepLX 或腾讯云

**理由**:
- 翻译质量高
- 速度快
- 配置简单

### 多语言混合翻译

**推荐**: LLM

**理由**:
- 更好地处理多种语言
- 可以区分不同语言的上下文

## 成本考量

### 预算有限

**推荐**: DeepLX

- 完全免费
- 无需担心费用

### 有一定预算

**推荐**: 腾讯云

- 500 万字符/月免费
- 超出后按量计费，价格合理

### 预算充足

**推荐**: LLM

- 翻译质量可配置
- 支持高级功能

## 可靠性考量

### 服务稳定性

从高到低：
1. 腾讯云
2. LLM（多提供商）
3. DeepLX

**建议**:
- 高可靠性要求: 腾讯云或 LLM
- 一般要求: DeepLX

### 故障转移

**推荐**: 配置多个翻译器

```toml
enabled_providers = ["tencent", "deeplx", "llm"]
```

工具会自动在翻译器之间进行故障转移。

## 混合使用策略

### 主备模式

```toml
enabled_providers = ["tencent", "deeplx"]
```

- 腾讯云作为主翻译器
- DeepLX 作为备选

### 负载均衡

```toml
enabled_providers = ["deeplx", "llm"]

[llm.health_check]
enabled = true

[[llm.providers]]
id = "silicon"
weight = 70

[[llm.providers]]
id = "zhipu"
weight = 30
```

- 根据权重分配负载
- 自动健康检查

### 优先级模式

根据文本长度选择翻译器：

- 短文本: DeepLX（快速）
- 长文本: LLM（质量好）

```toml
enabled_providers = ["deeplx", "llm"]
```

工具会根据文本长度自动选择。

## 配置建议

### 通用配置

```toml
# 启用多个翻译器提高可靠性
enabled_providers = ["tencent", "deeplx"]

# 配置合理的批量大小和并发数
[translate]
batch_size = 50
concurrency = 5

# 启用缓存
[cache]
enabled = true
format = "binary"
```

### 高性能配置

```toml
# 使用最快的翻译器
enabled_providers = ["deeplx"]

[deeplx]
rate_limit = 20

[translate]
batch_size = 100
concurrency = 10
```

### 高质量配置

```toml
# 使用 LLM 并优化提示词
enabled_providers = ["llm"]

[llm.health_check]
enabled = true

[[llm.providers]]
id = "high-quality"
model = "gpt-4"
temperature = 0.2
custom_system_prompt = "You are a professional translator. Maintain technical accuracy and natural flow."
```

## 测试建议

### 测试翻译质量

```bash
# 使用小规模测试
translator translate ./test-files --target-lang en

# 查看翻译结果
git diff
```

### 测试性能

```bash
# 使用 --dry-run 预览
translator translate . --dry-run

# 使用不同的并发数测试
translator translate . --concurrency 2
translator translate . --concurrency 5
translator translate . --concurrency 10
```

### 测试成本

```bash
# 启用详细日志查看 API 调用
translator translate . --log-level debug

# 查看缓存统计
translator cache --detailed
```

## 迁移建议

### 从 DeepLX 迁移到腾讯云

```toml
# 先启用腾讯云
enabled_providers = ["deeplx", "tencent"]

# 确认工作正常后，移除 DeepLX
enabled_providers = ["tencent"]
```

### 从单一翻译器迁移到多翻译器

```toml
# 逐步添加翻译器
enabled_providers = ["current", "new1"]
enabled_providers = ["current", "new1", "new2"]

# 确认工作正常后，调整权重
```

## 常见问题

### 1. 应该使用几个翻译器？

**建议**:
- 小型项目: 1-2 个
- 中型项目: 2-3 个
- 大型项目: 3 个以上

### 2. 如何配置权重？

**建议**:
- 根据翻译器的性能和成本配置
- 性能好、成本低的翻译器配置更高的权重

### 3. 如何控制成本？

**建议**:
- 优先使用免费翻译器
- 启用缓存
- 配置合理的速率限制
- 定期检查使用情况

### 4. 如何提高翻译质量？

**建议**:
- 使用 LLM 并优化提示词
- 使用腾讯云的术语库
- 配置语言专用提取

## 总结

| 场景 | 推荐翻译器 |
|------|-----------|
| 个人项目 | DeepLX |
| 小型商业项目 | 腾讯云 |
| 大型项目 | LLM |
| 高可靠性要求 | 多翻译器组合 |
| 成本敏感型项目 | 腾讯云 + DeepLX |
| 高性能要求 | DeepLX |
| 高质量要求 | LLM（自定义提示词） |

## 相关文档

- [DeepLX 翻译器](deeplx.md)
- [LLM 翻译器](llm.md)
- [腾讯云翻译器](tencent.md)
- [配置指南](../user-guide/configuration.md)