# 腾讯云翻译器

腾讯云翻译器使用腾讯云机器翻译服务，每月提供 500 万字符免费额度。

## 特性

- 每月 500 万字符免费额度
- 支持多种语言对
- 翻译质量高
- 稳定可靠
- 支持术语库和句库

## 支持的语言

### 源语言

- auto（自动检测）
- zh（中文）
- en（英语）
- ja（日语）
- ko（韩语）
- de（德语）
- fr（法语）
- es（西班牙语）
- it（意大利语）
- pt（葡萄牙语）
- ru（俄语）

### 目标语言

- zh（中文）
- en（英语）
- ja（日语）
- ko（韩语）
- de（德语）
- fr（法语）
- es（西班牙语）
- it（意大利语）
- pt（葡萄牙语）
- ru（俄语）

## 配置示例

```toml
[tencent]
secret_id = "${TENCENT_SECRET_ID}"
secret_key = "${TENCENT_SECRET_KEY}"
region = "ap-guangzhou"
project_id = 0
endpoint = "tmt.tencentcloudapi.com"
proxy_url = ""
timeout = 30
rate_limit = 5
max_retries = 3
untranslated_text = []
term_repo_id_list = []
sent_repo_id_list = []
```

## 配置参数

| 参数 | 说明 | 默认值 | 必填 |
|------|------|--------|------|
| secret_id | 腾讯云 Secret ID | - | 是 |
| secret_key | 腾讯云 Secret Key | - | 是 |
| region | 区域 | ap-guangzhou | 否 |
| project_id | 项目 ID | 0 | 否 |
| endpoint | API 端点 | tmt.tencentcloudapi.com | 否 |
| proxy_url | 代理 URL | 空 | 否 |
| timeout | 超时时间（秒） | 30 | 否 |
| rate_limit | 速率限制（请求/秒） | 5 | 否 |
| max_retries | 最大重试次数 | 3 | 否 |
| untranslated_text | 未翻译文本模式 | [] | 否 |
| term_repo_id_list | 术语库 ID 列表 | [] | 否 |
| sent_repo_id_list | 句库 ID 列表 | [] | 否 |

## 使用限制

- **最大输入字符数**: 6000 字符
- **建议分割大小**: 5000 字符
- **速率限制**: 不超过 5 请求/秒（腾讯云要求）
- **免费额度**: 500 万字符/月

## 获取密钥

1. 访问 [腾讯云控制台](https://console.cloud.tencent.com/)
2. 登录或注册账号
3. 搜索"机器翻译"
4. 开通机器翻译服务
5. 在访问管理中创建 API 密钥
6. 记录 `SecretId` 和 `SecretKey`

## 使用建议

### 1. 配置密钥

```env
# .env
TENCENT_SECRET_ID=your-secret-id
TENCENT_SECRET_KEY=your-secret-key
```

### 2. 设置合理的速率限制

```toml
[tencent]
rate_limit = 5  # 腾讯云要求不超过 5 请求/秒
```

### 3. 使用术语库（可选）

```toml
[tencent]
term_repo_id_list = ["term-repo-id-1"]
sent_repo_id_list = ["sent-repo-id-1"]
```

### 4. 配置区域

```toml
[tencent]
region = "ap-guangzhou"  # 根据实际情况选择
```

可用区域：
- ap-guangzhou（广州）
- ap-shanghai（上海）
- ap-beijing（北京）

## 注意事项

### 优点

1. **免费额度**: 每月 500 万字符免费
2. **稳定性**: 服务稳定可靠
3. **质量高**: 翻译质量高
4. **速度快**: 响应速度快
5. **功能丰富**: 支持术语库和句库

### 缺点

1. **需要认证**: 需要配置密钥
2. **限流严格**: 速率限制较低
3. **地区限制**: 可能在某些地区无法访问

### 最佳实践

1. **监控使用量**: 定期检查免费额度使用情况

2. **合理设置速率限制**: 不要超过腾讯云的限制

```toml
[tencent]
rate_limit = 4  # 留一点余量
```

3. **使用缓存**: 避免重复翻译相同内容

```toml
[cache]
enabled = true
```

4. **配置多个翻译器**: 配置其他翻译器作为备选

```toml
enabled_providers = ["tencent", "deeplx", "llm"]
```

5. **检查配额**: 定期查看 API 配额使用情况

## 故障排查

### 认证失败

**问题**: 认证失败，返回错误

**解决方案**:
1. 检查 `SecretId` 和 `SecretKey` 是否正确
2. 检查密钥是否有效
3. 检查是否有权限访问机器翻译服务
4. 查看详细日志

```bash
translator translate --log-level debug
```

### 配额用尽

**问题**: 免费配额用尽

**解决方案**:
1. 查看配额使用情况
2. 购买额外的配额
3. 切换到其他翻译器

### 速率限制

**问题**: 被速率限制

**解决方案**:
1. 降低 `rate_limit` 配置
2. 减少并发数
3. 增加 `max_retries`

```bash
translator translate . --concurrency 2
```

### 超时

**问题**: 请求超时

**解决方案**:
1. 增加 `timeout` 配置
2. 检查网络连接
3. 尝试其他区域

```toml
[tencent]
timeout = 60
```

## 性能优化

### 1. 配置合理的并发数

```toml
[translate]
concurrency = 4  # 不超过速率限制
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

### 4. 调整超时时间

```toml
[tencent]
timeout = 60
```

## 成本控制

### 1. 监控使用量

定期检查配额使用情况：

```bash
# 查看腾讯云控制台
# 或查看翻译统计
translator cache --detailed
```

### 2. 使用缓存

避免重复翻译相同内容：

```toml
[cache]
enabled = true
```

### 3. 合理设置速率限制

避免不必要的请求：

```toml
[tencent]
rate_limit = 3  # 降低速率
```

## 术语库和句库

### 创建术语库

1. 登录腾讯云控制台
2. 进入机器翻译服务
3. 创建术语库
4. 添加术语
5. 获取术语库 ID

### 使用术语库

```toml
[tencent]
term_repo_id_list = ["term-repo-id-1"]
```

### 创建句库

1. 登录腾讯云控制台
2. 进入机器翻译服务
3. 创建句库
4. 添加句子
5. 获取句库 ID

### 使用句库

```toml
[tencent]
sent_repo_id_list = ["sent-repo-id-1"]
```

## 与其他翻译器对比

| 特性 | 腾讯云 | DeepLX | LLM |
|------|--------|--------|-----|
| 费用 | 500万字符/月免费 | 免费 | 按使用量收费 |
| 质量 | 高 | 高 | 可配置 |
| 速度 | 快 | 快 | 慢 |
| 功能 | 丰富 | 基础 | 最丰富 |
| 稳定性 | 高 | 中等 | 高 |
| 认证 | 需要 | 不需要 | 需要 |

## 示例

### 基础使用

```bash
# 使用腾讯云翻译
translator translate . --provider tencent --target-lang en
```

### 配置多个区域

```toml
[tencent]
region = "ap-guangzhou"
```

### 使用术语库

```toml
[tencent]
term_repo_id_list = ["your-term-repo-id"]
```

### 自定义未翻译文本

```toml
[tencent]
untranslated_text = ["TODO", "FIXME"]
```

## 相关文档

- [配置指南](../user-guide/configuration.md)
- [DeepLX 翻译器](deeplx.md)
- [LLM 翻译器](llm.md)
- [翻译器选择指南](provider-selection.md)