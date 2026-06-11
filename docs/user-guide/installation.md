# 安装指南

## 系统要求

- Rust 1.80 或更高版本
- 支持的操作系统：Linux、macOS、Windows
- 互联网连接（用于调用翻译 API）

## 安装步骤

### 从源代码构建

1. 克隆仓库：

```bash
git clone https://github.com/your-org/codebase-translate.git
cd codebase-translate
```

2. 构建发布版本：

```bash
cargo build --release
```

3. 编译后的二进制文件位于 `target/release/translator`

4. 将二进制文件添加到 PATH：

```bash
# Linux/Mac
sudo cp target/release/translator /usr/local/bin/

# Windows
# 将 target/release/translator.exe 复制到系统 PATH 中的目录
```

### 使用 Cargo 安装

如果发布了到 crates.io，可以直接使用 cargo 安装：

```bash
cargo install codebase-translate
```

## 初始化配置

### 初始化全局配置

```bash
translator init --global
```

这将在用户配置目录创建 `translator.toml` 文件。

### 初始化项目配置

```bash
cd /path/to/your/project
translator init
```

这将在项目根目录创建 `.translator.toml` 文件。

## 配置 API 密钥

1. 复制示例环境变量文件：

```bash
cp .env.example .env
```

2. 编辑 `.env` 文件，填入你的 API 密钥：

```env
# DeepLX
DEEPLX_API_URL=https://api.deeplx.org
DEEPLX_API_KEY=your-api-key-here

# LLM Providers
SILCON_API_KEY=xxx
ZHIPU_API_KEY=xxx

# 腾讯云
TENCENT_SECRET_ID=xxx
TENCENT_SECRET_KEY=xxx
```

## 验证安装

运行以下命令验证安装是否成功：

```bash
translator --version
translator validate
```

## 卸载

### 从源代码构建的版本

```bash
# Linux/Mac
sudo rm /usr/local/bin/translator

# Windows
# 删除 translator.exe
```

### 使用 Cargo 安装的版本

```bash
cargo uninstall codebase-translate
```

### 清理配置文件

```bash
# 全局配置
rm -rf ~/.config/codebase-translate/

# 项目配置
rm -rf /path/to/project/.translator.toml
rm -rf /path/to/project/.translator/
```

## 常见问题

### 编译失败

确保安装了 Rust 1.80 或更高版本：

```bash
rustc --version
```

如果版本过低，更新 Rust：

```bash
rustup update
```

### 找不到命令

确保 `translator` 已添加到 PATH：

```bash
# 检查 PATH
echo $PATH

# 手动添加到 PATH（临时）
export PATH="$PATH:/path/to/translator"

# 永久添加到 PATH，编辑 ~/.bashrc 或 ~/.zshrc
export PATH="$PATH:/usr/local/bin"
```

### 权限错误

在 Linux/Mac 上可能需要 sudo：

```bash
sudo cp target/release/translator /usr/local/bin/
```

## 依赖项

主要依赖项：

- `tokio`: 异步运行时
- `reqwest`: HTTP 客户端
- `serde`: 序列化/反序列化
- `tree-sitter`: 代码解析
- `whatlang`: 语言检测
- `tracing`: 日志框架
- `anyhow`: 错误处理
- `thiserror`: 错误派生

完整依赖列表请查看 `Cargo.toml` 文件。

## 开发环境

如果要参与开发，需要：

1. 克隆仓库
2. 安装开发依赖：

```bash
cargo build
cargo test
```

3. 运行代码质量检查：

```bash
cargo clippy --all-targets --all-features
cargo fmt --check
```

## 发布版本

发布版本使用优化的编译选项，性能更高。开发版本可以使用：

```bash
cargo build
```

这会在 `target/debug/` 目录生成二进制文件，但性能较低。