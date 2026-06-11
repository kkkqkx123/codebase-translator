# Scanner Module Design

## 概述

Scanner 模块提供文件系统扫描功能，根据 include/exclude 模式筛选文件，支持 .gitignore 规则，为翻译系统提供文件发现和过滤能力。

## 设计目的

1. **文件发现**：递归扫描目录树，发现所有候选文件
2. **模式过滤**：使用通配符模式过滤文件
3. **Gitignore 集成**：尊重 .gitignore 规则，忽略不应翻译的文件
4. **高效扫描**：优化的文件遍历，避免不必要的系统调用

## 核心组件

### 1. Scanner Trait

**位置**：`src/scanner/trait.rs`

**职责**：
- 定义扫描器接口
- 提供统一的扫描方法

**关键类型**：
```rust
pub trait Scanner: Send + Sync {
    fn scan(&self, opts: ScanOptions) -> Result<Vec<FileEntry>>;
}

pub struct ScanOptions {
    pub root_path: String,              // 根路径
    pub include_patterns: Vec<String>,  // 包含模式（通配符）
    pub exclude_patterns: Vec<String>,  // 排除模式（通配符）
    pub follow_symlinks: bool,          // 是否跟随符号链接
    pub respect_gitignore: bool,        // 是否尊重 .gitignore
    pub gitignore_patterns: Vec<String>, // 额外的 gitignore 模式
    pub gitignore_path: Option<PathBuf>, // .gitignore 文件路径
}
```

### 2. FSScanner

**位置**：`src/scanner/walker.rs`

**职责**：
- 文件系统扫描器实现
- 递归遍历目录
- 应用过滤规则

**关键功能**：
```rust
pub struct FSScanner;

impl Scanner for FSScanner {
    fn scan(&self, opts: ScanOptions) -> Result<Vec<FileEntry>> {
        // 1. 加载 .gitignore（如果启用）
        let gitignore = self.load_gitignore(&opts)?;

        // 2. 收集额外的 gitignore 模式
        let additional_patterns: Vec<_> = opts.gitignore_patterns
            .iter()
            .map(|p| ignore::gitignore::Glob::new(p))
            .collect::<Result<_, _>>()?;

        // 3. 递归扫描目录
        let entries = self.walk_directory(
            Path::new(&opts.root_path),
            &opts,
            &gitignore,
            &additional_patterns,
        )?;

        // 4. 应用 include/exclude 模式
        let filtered = self.apply_patterns(entries, &opts)?;

        Ok(filtered)
    }
}
```

**扫描流程**：
1. 验证根路径存在
2. 加载 .gitignore 规则
3. 递归遍历目录
4. 跳过隐藏目录（如 .git, .translate）
5. 应用 gitignore 规则
6. 应用 include/exclude 模式
7. 返回匹配的文件列表

### 3. GitignoreMatcher

**位置**：`src/scanner/gitignore.rs`

**职责**：
- 解析和匹配 .gitignore 规则
- 支持标准 .gitignore 语法

**关键功能**：
```rust
pub struct GitignoreMatcher {
    patterns: Vec<ignore::gitignore::Glob>,
}

impl GitignoreMatcher {
    pub fn from_path(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self> {
        let patterns: Result<Vec<_>, _> = content
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            .map(|line| ignore::gitignore::Glob::new(line))
            .collect();

        Ok(Self {
            patterns: patterns?,
        })
    }

    pub fn matches(&self, path: &Path, is_dir: bool) -> bool {
        for pattern in &self.patterns {
            if pattern.matches_path_with(path, is_dir) {
                return true;
            }
        }
        false
    }
}
```

**支持的模式**：
- 通配符：`*`, `**`, `?`
- 否定：`!pattern`
- 目录匹配：`/pattern/`
- 扩展名：`*.rs`

### 4. FileEntry

**位置**：`src/core/models.rs`

**职责**：
- 表示文件条目
- 包含文件元数据

**关键字段**：
```rust
pub struct FileEntry {
    pub path: PathBuf,      // 文件路径（相对或绝对）
    pub size: u64,          // 文件大小
    pub is_file: bool,      // 是否为文件
    pub is_symlink: bool,   // 是否为符号链接
}
```

## 技术选型

### Gitignore 解析
- **ignore**：.gitignore 解析库
  - 标准 .gitignore 语法
  - 高性能匹配
  - 广泛使用

### 模式匹配
- **glob**：通配符模式匹配
  - 标准通配符语法
  - Unicode 支持
  - 路径归一化

### 文件系统操作
- **std::fs**：标准文件系统操作
  - 跨平台
  - 异常安全
  - 高效

## 关键设计要点

### 1. 目录遍历

```rust
fn walk_directory(
    &self,
    dir: &Path,
    opts: &ScanOptions,
    gitignore: &Option<GitignoreMatcher>,
    additional_patterns: &[ignore::gitignore::Glob],
) -> Result<Vec<FileEntry>> {
    let mut entries = Vec::new();

    // 读取目录条目
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        // 跳过隐藏目录
        if path.is_dir() && is_hidden_directory(&path) {
            continue;
        }

        // 检查 gitignore
        if let Some(gitignore) = gitignore {
            let relative = path.strip_prefix(&opts.root_path)?;
            if gitignore.matches(relative, path.is_dir()) {
                continue;
            }
        }

        // 检查额外模式
        if !additional_patterns.is_empty() {
            let relative = path.strip_prefix(&opts.root_path)?;
            for pattern in additional_patterns {
                if pattern.matches_path_with(relative, path.is_dir()) {
                    continue; // 跳过匹配的路径
                }
            }
        }

        // 递归遍历目录
        if path.is_dir() {
            if metadata.is_symlink() && !opts.follow_symlinks {
                continue;
            }
            let mut sub_entries = self.walk_directory(
                &path,
                opts,
                gitignore,
                additional_patterns,
            )?;
            entries.append(&mut sub_entries);
        } else if path.is_file() {
            entries.push(FileEntry {
                path,
                size: metadata.len(),
                is_file: true,
                is_symlink: metadata.is_symlink(),
            });
        }
    }

    Ok(entries)
}
```

### 2. 模式过滤

```rust
fn apply_patterns(
    &self,
    entries: Vec<FileEntry>,
    opts: &ScanOptions,
) -> Result<Vec<FileEntry>> {
    // 编译 include 模式
    let include_patterns: Vec<_> = opts.include_patterns
        .iter()
        .map(|p| glob::Pattern::new(p))
        .collect::<Result<_, _>>()?;

    // 编译 exclude 模式
    let exclude_patterns: Vec<_> = opts.exclude_patterns
        .iter()
        .map(|p| glob::Pattern::new(p))
        .collect::<Result<_, _>>()?;

    entries.into_iter()
        .filter(|entry| {
            // 检查 include 模式
            let matches_include = include_patterns.is_empty()
                || include_patterns.iter().any(|p| {
                    p.matches_path(&entry.path)
                });

            // 检查 exclude 模式
            let matches_exclude = exclude_patterns.iter().any(|p| {
                p.matches_path(&entry.path)
            });

            matches_include && !matches_exclude
        })
        .collect()
}
```

### 3. 隐藏目录检测

```rust
fn is_hidden_directory(path: &Path) -> bool {
    let filename = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // 跳过常见隐藏目录
    matches!(filename,
        ".git" | ".svn" | ".hg" | "node_modules" | "target" |
        ".translate" | ".idea" | ".vscode" | "build" | "dist"
    )
}
```

### 4. 路径归一化

```rust
fn normalize_path(path: &Path) -> PathBuf {
    // 转换为绝对路径
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap()
            .join(path)
    };

    // 规范化路径
    absolute.canonicalize()
        .unwrap_or(absolute)
}
```

### 5. 符号链接处理

```rust
if metadata.is_symlink() {
    if opts.follow_symlinks {
        // 解析符号链接
        let target = std::fs::read_link(&path)?;
        // 继续处理目标
    } else {
        // 跳过符号链接
        continue;
    }
}
```

## 使用示例

### 基本扫描

```rust
use codebase_translate::scanner::{FSScanner, ScanOptions};

let scanner = FSScanner::new();

let opts = ScanOptions {
    root_path: "/workspace".to_string(),
    include_patterns: vec!["**/*.rs".to_string(), "**/*.py".to_string()],
    exclude_patterns: vec!["**/target/**".to_string()],
    follow_symlinks: false,
    respect_gitignore: true,
    gitignore_patterns: vec![],
    gitignore_path: None,
};

let entries = scanner.scan(opts)?;
for entry in entries {
    println!("{:?}", entry.path);
}
```

### 自定义 Gitignore

```rust
let opts = ScanOptions {
    root_path: "/workspace".to_string(),
    include_patterns: vec!["**/*.rs".to_string()],
    exclude_patterns: vec![],
    follow_symlinks: false,
    respect_gitignore: false,  // 不使用默认 .gitignore
    gitignore_patterns: vec![
        "**/tests/**".to_string(),
        "**/examples/**".to_string(),
    ],
    gitignore_path: Some(Path::new("/custom/.gitignore")),
};
```

### 仅包含特定目录

```rust
let opts = ScanOptions {
    root_path: "/workspace/src".to_string(),
    include_patterns: vec!["**".to_string()],  // 包含所有
    exclude_patterns: vec![],
    follow_symlinks: false,
    respect_gitignore: true,
    gitignore_patterns: vec![],
    gitignore_path: None,
};
```

## 性能考量

1. **并行扫描**（待实现）：
   - 多线程扫描不同子目录
   - 工作窃取调度
   - 减少总扫描时间

2. **缓存优化**：
   - Gitignore 规则缓存
   - 模式编译缓存
   - 避免重复计算

3. **系统调用优化**：
   - 批量读取目录
   - 减少元数据查询
   - 延迟加载

4. **内存效率**：
   - 流式返回结果
   - 按需分配
   - 避免克隆路径

## 扩展性

1. **新的扫描器**：
   - Git 仓库扫描器（使用 git ls-files）
   - S3 扫描器
   - FTP 扫描器

2. **高级过滤**：
   - 文件内容过滤
   - 文件大小过滤
   - 修改时间过滤

3. **事件驱动**：
   - 文件发现事件
   - 进度回调
   - 取消支持

4. **缓存层**：
   - 文件列表缓存
   - 增量扫描
   - 文件监控