# 文档索引

欢迎使用 Codebase Translate 文档中心。

## 快速导航

- [README](../README.md) - 项目概述和快速开始
- [安装指南](user-guide/installation.md) - 如何安装和配置
- [快速开始](user-guide/quick-start.md) - 5 分钟快速入门
- [配置指南](user-guide/configuration.md) - 详细的配置选项
- [工作流指南](user-guide/workflow.md) - 常见工作流程
- [命令参考](user-guide/cli-commands.md) - 所有命令的详细说明
- [翻译器选择指南](translator/provider-selection.md) - 如何选择合适的翻译器

## 用户指南

### 入门

1. [安装指南](user-guide/installation.md) - 系统要求、安装步骤、常见问题
2. [快速开始](user-guide/quick-start.md) - 5 分钟快速入门指南

### 配置

3. [配置指南](user-guide/configuration.md) - 全局配置、项目配置、环境变量

### 使用

4. [工作流指南](user-guide/workflow.md) - 基本工作流、CI/CD 集成、最佳实践
5. [命令参考](user-guide/cli-commands.md) - 所有命令和选项的详细说明

## 翻译器文档

### 翻译器选择

- [翻译器选择指南](translator/provider-selection.md) - 如何选择合适的翻译器，对比各种场景

### 翻译器详情

- [DeepLX 翻译器](translator/deeplx.md) - 免费翻译服务的使用说明和注意事项
- [LLM 翻译器](translator/llm.md) - 大语言模型翻译器的配置和使用
- [腾讯云翻译器](translator/tencent.md) - 腾讯云机器翻译服务的使用说明

## 开发文档

开发文档位于 [dev/](dev/) 目录，包含：

- [工作流](dev/workflow.md) - 翻译工作流程
- [扫描器](dev/scanner.md) - 文件扫描机制
- [解析器](dev/parser.md) - 代码解析
- [翻译器](dev/translator.md) - 翻译服务实现
- [缓存](dev/cache.md) - 缓存机制
- [写入器](dev/writer.md) - 文件写入
- [日志](dev/logger.md) - 日志系统
- [配置](dev/config.md) - 配置加载
- [报告](dev/reporter.md) - 统计报告

## 存档文档

存档文档位于 [archive/](archive/) 目录，包含项目历史记录和特殊主题：

- [不安全代码](archive/unsafe.md) - unsafe 代码的使用记录
- [动态分发](archive/dynamic.md) - 动态分发的使用记录

## 常见使用场景

### 场景 1: 个人项目

1. 阅读 [快速开始](user-guide/quick-start.md)
2. 选择 [DeepLX 翻译器](translator/deeplx.md)
3. 按照 [工作流指南](user-guide/workflow.md) 执行翻译

### 场景 2: 商业项目

1. 阅读 [安装指南](user-guide/installation.md)
2. 选择 [腾讯云翻译器](translator/tencent.md) 或 [LLM 翻译器](translator/llm.md)
3. 配置 [CI/CD 集成](user-guide/workflow.md#cicd-工作流)

### 场景 3: 大型项目

1. 阅读 [配置指南](user-guide/configuration.md)
2. 选择 [多翻译器组合](translator/provider-selection.md#场景-4-高可靠性要求)
3. 参考 [大型项目工作流](user-guide/workflow.md#大型项目工作流)

## 故障排查

### 配置问题

- 查看配置验证: `translator validate`
- 检查环境变量配置
- 参考 [配置指南](user-guide/configuration.md)

### 翻译问题

- 查看详细日志: `translator translate --log-level debug`
- 检查翻译器配置
- 参考各翻译器的 [故障排查](translator/) 部分

### 性能问题

- 调整并发数: `--concurrency 2`
- 减少批量大小: `--batch-size 20`
- 启用缓存: [缓存配置](user-guide/configuration.md#缓存配置)

## 贡献

如果您想贡献文档，请：

1. 保持文档清晰和准确
2. 提供示例和最佳实践
3. 及时更新过时的内容
4. 遵循现有的文档结构和风格

## 更新日志

文档更新记录：

- 2025-06-11: 创建完整的用户指南和翻译器文档
- 2025-06-10: 创建命令参考文档

## 反馈

如果您有任何问题或建议，请：

- 提交 [GitHub Issue](https://github.com/your-org/codebase-translate/issues)
- 参与 [GitHub Discussions](https://github.com/your-org/codebase-translate/discussions)
- 参考现有文档解决问题

## 术语表

- **全局配置**: 适用于所有项目的配置，存储在 `translator.toml`
- **项目配置**: 特定于某个项目的配置，存储在 `.translator.toml`
- **翻译提供商**: 提供翻译服务的第三方平台（DeepLX、LLM、腾讯云）
- **缓存**: 基于文件哈希的缓存机制，避免重复翻译
- **提取规则**: 从代码中提取需要翻译的文本的规则
- **过滤规则**: 过滤不需要翻译的文本的规则

## 相关资源

- [项目主页](https://github.com/your-org/codebase-translate)
- [示例项目](https://github.com/your-org/codebase-translate-examples)
- [API 文档](https://docs.rs/codebase-translate)
- [更新日志](CHANGELOG.md)

---

需要帮助？从 [快速开始](user-guide/quick-start.md) 开始吧！