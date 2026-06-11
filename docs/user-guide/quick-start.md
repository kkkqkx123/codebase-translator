# 快速开始

本指南将帮助你快速上手 Codebase Translate。

## 5 分钟快速入门

### 1. 安装

```bash
# 从源代码构建
cargo build --release

# 或使用 Cargo 安装（如果已发布）
cargo install codebase-translate
```

### 2. 初始化配置

```bash
# 初始化全局配置
translator init --global

# 初始化项目配置
cd your-project
translator init
```

### 3. 配置 API 密钥

编辑全局配置目录下的 `.env` 文件：

```env
# DeepLX
DEEPLX_API_URL=https://api.deeplx.org
DEEPLX_API_KEY=your-api-key-here

# LLM Providers
SILCON_API_KEY=xxx
```

### 4. 验证配置

```bash
translator validate
```

### 5. 执行翻译

```bash
# 翻译当前目录到英语
translator translate . --target-lang en
```

## 典型工作流程

### 翻译现有项目

```bash
# 1. 进入项目目录
cd /path/to/your/project

# 2. 初始化项目配置
translator init

# 3. 配置翻译设置（编辑 .translator.toml）
nano .translator.toml

# 4. 预览将要提取的内容
translator verify

# 5. 执行预演
translator translate --dry-run

# 6. 执行翻译
translator translate
```

### 翻译特定语言

```bash
# 将中文注释翻译为英语
translator translate . --source-langs zh --target-lang en

# 将多种语言翻译为英语
translator translate . --source-langs "zh,ja,ko" --target-lang en
```

### 使用特定翻译器

```bash
# 使用 DeepLX 翻译
translator translate . --provider deeplx --target-lang en

# 使用 LLM 翻译
translator translate . --provider llm --target-lang en

# 使用腾讯云翻译
translator translate . --provider tencent --target-lang en
```

### 翻译特定文件类型

```bash
# 只翻译 Rust 文件
translator translate . --include "*.rs"

# 翻译多种文件类型
translator translate . --include "*.rs,*.py,*.js"

# 排除测试文件
translator translate . --exclude "*test*.rs"
```

## 常见使用场景

### 场景 1: 新项目国际化

```bash
# 1. 初始化配置
translator init

# 2. 设置翻译为英语
# 编辑 .translator.toml，设置 target_lang = "en"

# 3. 执行翻译
translator translate

# 4. 查看翻译报告
cat .translator/report_*.txt
```

### 场景 2: 增量翻译

```bash
# 工具自动使用缓存，只翻译修改过的文件
translator translate

# 查看缓存统计
translator cache --detailed
```

### 场景 3: 批量翻译多个项目

```bash
# 遍历多个项目
for project in project1 project2 project3; do
    cd $project
    translator translate --target-lang en
    cd ..
done
```

### 场景 4: CI/CD 集成

```bash
# 在 CI 脚本中
translator translate . --target-lang en --log-level error

# 检查翻译结果
if [ $? -eq 0 ]; then
    echo "Translation successful"
else
    echo "Translation failed"
    exit 1
fi
```

## 配置建议

### 小型项目

```toml
[translate]
batch_size = 20
concurrency = 2

[cache]
enabled = true
```

### 中型项目

```toml
[translate]
batch_size = 50
concurrency = 5

[cache]
enabled = true
```

### 大型项目

```toml
[translate]
batch_size = 100
concurrency = 10

[cache]
enabled = true
format = "binary"
```

## 故障排查

### 翻译失败

```bash
# 启用详细日志
translator translate --log-level debug

# 验证配置
translator validate

# 检查 API 密钥
cat ~/.config/codebase-translate/.env
```

### 翻译质量不佳

```bash
# 尝试不同的翻译器
translator translate . --provider deeplx

# 调整过滤规则
# 编辑 .translator.toml 的 [filter] 部分

# 使用语言专用提取
# 编辑 .translator.toml，设置 extract_languages = ["ZH"]
```

### 性能问题

```bash
# 减少并发数
translator translate . --concurrency 2

# 减少批量大小
translator translate . --batch-size 20

# 查看统计信息
translator cache --detailed
```

## 下一步

- 阅读完整配置指南：[配置指南](configuration.md)
- 了解所有命令：[命令参考](cli-commands.md)
- 查看翻译器详情：[翻译器指南](../translator/)

## 示例项目

查看示例项目了解最佳实践：

```bash
# 克隆示例项目
git clone https://github.com/your-org/codebase-translate-examples.git
cd codebase-translate-examples

# 查看配置文件
cat .translator.toml

# 运行翻译
translator translate
```

## 获取帮助

```bash
# 查看帮助信息
translator --help

# 查看特定命令帮助
translator translate --help

# 查看版本信息
translator --version
```