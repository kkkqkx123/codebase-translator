# 工作流指南

本指南介绍使用 Codebase Translate 的常见工作流程和最佳实践。

## 基本工作流程

### 1. 项目初始化

```bash
# 进入项目目录
cd /path/to/your/project

# 初始化项目配置
translator init

# 编辑配置文件
nano .translator.toml
```

### 2. 配置验证

```bash
# 验证配置是否正确
translator validate

# 查看将要提取的内容
translator verify
```

### 3. 预演翻译

```bash
# 预演模式，不实际修改文件
translator translate --dry-run

# 查看翻译报告
cat .translator/report_*.txt
```

### 4. 执行翻译

```bash
# 执行翻译
translator translate

# 查看翻译结果
git diff
```

### 5. 验证结果

```bash
# 检查翻译质量
git diff --stat

# 运行测试确保代码没有破坏
cargo test
npm test
```

### 6. 提交更改

```bash
# 添加翻译后的文件
git add .

# 提交更改
git commit -m "chore: translate codebase to English"
```

## 增量翻译工作流

### 日常开发

```bash
# 修改代码后，只翻译新增或修改的内容
translator translate

# 工具自动使用缓存，跳过未修改的文件
```

### 定期维护

```bash
# 检查缓存统计
translator cache --detailed

# 清理旧备份
translator clean --backup --older-than 30

# 清理旧缓存
translator clean --cache --older-than 7
```

## 多语言项目工作流

### 阶段 1: 初始翻译

```bash
# 翻译所有中文注释为英语
translator translate . --source-langs zh --target-lang en
```

### 阶段 2: 持续翻译

```bash
# 确保配置文件设置了 extract_languages
# .translator.toml
[filter]
extract_languages = ["ZH", "JA", "KO"]

# 翻译新增的外语注释
translator translate
```

### 阶段 3: 质量检查

```bash
# 验证翻译质量
translator verify --search "特定关键词"

# 查看翻译统计
translator cache
```

## CI/CD 工作流

### GitHub Actions

```yaml
name: Translate Codebase

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  translate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Codebase Translate
        run: |
          cargo install codebase-translate

      - name: Setup API Keys
        env:
          DEEPLX_API_KEY: ${{ secrets.DEEPLX_API_KEY }}
        run: |
          echo "DEEPLX_API_KEY=$DEEPLX_API_KEY" > .env

      - name: Translate
        run: |
          translator translate . --target-lang en --log-level error

      - name: Check for changes
        run: |
          if git diff --quiet; then
            echo "No translation changes"
          else
            echo "Translation changes detected"
            git diff --stat
          fi
```

### GitLab CI

```yaml
translate:
  image: rust:latest
  script:
    - cargo install codebase-translate
    - echo "DEEPLX_API_KEY=$DEEPLX_API_KEY" > .env
    - translator translate . --target-lang en --log-level error
    - git diff --stat
  variables:
    DEEPLX_API_KEY: $DEEPLX_API_KEY
```

## 大型项目工作流

### 分批翻译

```bash
# 按模块分批翻译
translator translate ./module1 --target-lang en
translator translate ./module2 --target-lang en
translator translate ./module3 --target-lang en
```

### 并行翻译

```bash
# 使用多个终端并行翻译不同目录
# 终端 1
translator translate ./src --target-lang en &

# 终端 2
translator translate ./tests --target-lang en &

# 等待所有翻译完成
wait
```

### 增量策略

```bash
# 只翻译核心模块
translator translate ./src/core --target-lang en

# 验证结果
git diff src/core

# 再翻译其他模块
translator translate ./src/utils --target-lang en
```

## 回滚和恢复

### 查看备份

```bash
# 备份文件位于 .translator/backup/
ls -la .translator/backup/

# 比较备份和当前文件
diff .translator/backup/src/main.rs src/main.rs
```

### 恢复单个文件

```bash
# 从备份恢复
cp .translator/backup/src/main.rs src/main.rs
```

### 恢复整个项目

```bash
# 使用 Git 恢复
git restore .

# 或手动恢复所有备份文件
find .translator/backup -type f -exec sh -c 'cp "$1" "${1#.translator/backup/}"' _ {} \;
```

## 最佳实践

### 1. 版本控制

- 始终在翻译前提交当前更改
- 使用 Git 追踪翻译后的更改
- 为翻译创建单独的分支

```bash
# 创建翻译分支
git checkout -b translate-to-en

# 执行翻译
translator translate . --target-lang en

# 提交更改
git add .
git commit -m "chore: translate codebase to English"

# 合并到主分支
git checkout main
git merge translate-to-en
```

### 2. 测试验证

翻译后始终运行测试确保代码没有破坏：

```bash
# Rust 项目
cargo test

# Python 项目
pytest

# Node.js 项目
npm test
```

### 3. 质量检查

定期检查翻译质量：

```bash
# 查看翻译统计
translator cache --detailed

# 搜索特定模式
translator verify --search "TODO"

# 导出详细报告
translator verify --format json --output report.json
```

### 4. 性能优化

根据项目大小调整配置：

```toml
# 小型项目
[translate]
batch_size = 20
concurrency = 2

# 大型项目
[translate]
batch_size = 100
concurrency = 10
```

### 5. 备份策略

始终启用备份功能：

```toml
[writer]
backup = true
backup_dir = ".translator/backup"
```

定期清理旧备份：

```bash
translator clean --backup --older-than 30
```

## 故障排查

### 翻译中断

如果翻译过程中断：

```bash
# 重新运行翻译，工具会自动跳过已翻译的文件
translator translate
```

### API 限流

遇到 API 限流：

```bash
# 降低并发数
translator translate . --concurrency 2

# 增加重试次数
# 编辑 translator.toml，设置 max_retries = 5
```

### 内存不足

大项目遇到内存问题：

```bash
# 减少批量大小
translator translate . --batch-size 20

# 分批翻译不同目录
translator translate ./src/module1
translator translate ./src/module2
```

## 高级技巧

### 自定义提取规则

添加自定义提取规则以处理特殊情况：

```toml
[[extraction.custom_patterns]]
name = "custom_error"
file_extensions = ["js", "ts"]
category = "error_handling"
regex = 'throw new Error\("([^"]+)"\)'
group = 1
```

### 多翻译器负载均衡

配置多个翻译器实现负载均衡：

```toml
# translator.toml
enabled_providers = ["deeplx", "llm", "tencent"]
```

工具会自动在多个翻译器之间分配负载。

### 语言专用提取

精确提取特定语言的文本：

```toml
[filter]
extract_languages = ["ZH"]
```

这样只会提取包含中文字符的文本，提高准确性。

## 团队协作

### 共享配置

将 `.translator.toml` 提交到版本控制：

```bash
git add .translator.toml
git commit -m "chore: add translator configuration"
```

### 环境变量

不要提交 `.env` 文件，使用团队共享的环境变量：

```bash
# .env.example（提交到版本控制）
DEEPLX_API_URL=https://api.deeplx.org
DEEPLX_API_KEY=your-api-key-here

# .env（不提交，团队成员本地配置）
```

### 文档记录

在项目中记录翻译配置和使用方法：

```markdown
# Translation

本项目使用 Codebase Translate 进行代码注释翻译。

## 配置

配置文件位于项目根目录的 `.translator.toml`。

## 使用方法

```bash
translator translate . --target-lang en
```

## 注意事项

- 翻译前提交所有更改
- 翻译后运行测试验证
- 使用 `--dry-run` 预览翻译结果
```