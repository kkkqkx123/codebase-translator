# Writer Module Design

## 概述

Writer 模块负责将翻译结果写回文件，支持原子写入、备份创建、预览模式和并发写入，确保文件写入的安全性和可靠性。

## 设计目的

1. **原子写入**：确保文件写入的原子性，避免写入过程中的文件损坏
2. **备份机制**：自动创建备份，防止翻译错误导致的数据丢失
3. **预览模式**：在不修改文件的情况下预览翻译结果
4. **并发写入**：支持并发写入多个文件，提高性能

## 核心组件

### 1. TranslationApplier

**位置**：`src/writer/core.rs`

**职责**：
- 应用翻译到文件内容
- 处理单行和多行翻译
- 保护格式和占位符

**关键功能**：
```rust
pub struct TranslationApplier;

impl TranslationApplier {
    pub fn apply_translations(content: &str, units: &[TranslationUnit]) -> Result<String> {
        if units.is_empty() {
            return Ok(content.to_string());
        }

        let line_ending = detect_line_ending(content);
        let normalized_content = content.replace("\r\n", "\n");

        // 步骤 1: 处理多行合并单元（逆序以避免偏移问题）
        let multiline_units: Vec<&TranslationUnit> = units
            .iter()
            .filter(|u| {
                u.start_pos.line != u.end_pos.line || u.content.contains('\n')
            })
            .collect();

        let result = if multiline_units.is_empty() {
            // 只有单行单元
            Self::apply_single_line_units(&normalized_content, units)?
        } else {
            // 先处理多行单元，再处理单行单元
            let temp = Self::apply_multiline_units(&normalized_content, &multiline_units)?;
            Self::apply_single_line_units(&temp, units)?
        };

        // 恢复原始换行符
        if line_ending == "\r\n" {
            Ok(result.replace('\n', "\r\n"))
        } else {
            Ok(result)
        }
    }

    fn apply_multiline_units(
        content: &str,
        units: &[&TranslationUnit],
    ) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut lines_vec = lines.to_vec();

        // 逆序处理以避免行号偏移
        for unit in units.iter().rev() {
            let start_line = unit.start_pos.line - 1;  // 转换为 0-based
            let end_line = unit.end_pos.line - 1;

            if end_line >= lines_vec.len() {
                continue;
            }

            let translation = unit.translation.as_ref()
                .ok_or("Translation not available")?;

            // 替换多行内容
            lines_vec[start_line..=end_line].fill(translation);
        }

        Ok(lines_vec.join("\n"))
    }

    fn apply_single_line_units(
        content: &str,
        units: &[TranslationUnit],
    ) -> Result<String> {
        let mut result = String::new();
        let mut line_offset = 0;

        for (line_num, line) in content.lines().enumerate() {
            let current_line = line_num + 1;  // 转换为 1-based

            // 查找该行的翻译单元
            let translations: Vec<_> = units
                .iter()
                .filter(|u| u.start_pos.line == current_line)
                .collect();

            if translations.is_empty() {
                result.push_str(line);
            } else {
                let mut translated_line = line.to_string();

                // 逆序处理以避免偏移问题
                for unit in translations.iter().rev() {
                    if let Some(translation) = &unit.translation {
                        let start = unit.start_pos.column - 1;  // 转换为 0-based
                        let end = unit.end_pos.column;

                        if start <= end && end <= translated_line.len() {
                            translated_line.replace_range(start..end, translation);
                        }
                    }
                }

                result.push_str(&translated_line);
            }

            if line_num < content.lines().count() - 1 {
                result.push('\n');
            }
        }

        Ok(result)
    }
}
```

**关键设计要点**：
- 多行单元优先处理（逆序）
- 单行单元逐行处理
- 字节偏移精确替换
- 换行符规范化处理

### 2. FileWriter

**位置**：`src/writer/file.rs`

**职责**：
- 异步文件写入
- 原子写入保证
- 备份创建
- 预览模式

**关键功能**：
```rust
pub struct FileWriter {
    config: WriterConfig,
    project_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct WriterConfig {
    pub preview_only: bool,
    pub backup: bool,
    pub backup_dir: Option<PathBuf>,
    pub strict_encoding: bool,
}

impl FileWriter {
    pub async fn write(&self, path: &Path, units: &[TranslationUnit]) -> Result<()> {
        // 1. 读取文件内容
        let content = tokio::fs::read_to_string(path).await?;

        // 2. 应用翻译
        let translated_content = TranslationApplier::apply_translations(&content, units)?;

        // 3. 预览模式
        if self.config.preview_only {
            println!("=== {} ===", path.display());
            println!("{}", translated_content);
            return Ok(());
        }

        // 4. 创建备份
        if self.config.backup {
            self.create_backup(path).await?;
        }

        // 5. 原子写入
        self.atomic_write(path, &translated_content).await?;

        Ok(())
    }

    async fn create_backup(&self, path: &Path) -> Result<()> {
        let backup_path = if let Some(ref backup_dir) = self.config.backup_dir {
            backup_dir.join(path.file_name().unwrap())
        } else {
            let mut backup_path = path.to_path_buf();
            backup_path.set_extension(format!("{}.bk", path.extension().unwrap().to_str().unwrap()));
            backup_path
        };

        tokio::fs::copy(path, &backup_path).await?;
        Ok(())
    }

    async fn atomic_write(&self, path: &Path, content: &str) -> Result<()> {
        // 写入临时文件
        let temp_path = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
        tokio::fs::write(&temp_path, content).await?;

        // 原子重命名
        tokio::fs::rename(&temp_path, path).await?;

        Ok(())
    }
}
```

**原子写入保证**：
1. 写入临时文件
2. 确保写入成功
3. 原子重命名
4. 失败时删除临时文件

### 3. ConcurrentWriter

**位置**：`src/writer/concurrent.rs`

**职责**：
- 并发文件写入
- 速率限制
- 错误处理和重试
- 统计收集

**关键功能**：
```rust
pub struct ConcurrentWriter {
    config: WriterConfig,
    max_concurrent: usize,
    project_path: Option<PathBuf>,
}

pub struct WriteResult {
    pub path: PathBuf,
    pub success: bool,
    pub error: Option<String>,
    pub units_written: usize,
}

impl ConcurrentWriter {
    pub async fn write_batch(
        &self,
        writes: Vec<(PathBuf, Vec<TranslationUnit>)>,
    ) -> Result<Vec<WriteResult>> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut tasks = Vec::new();

        for (path, units) in writes {
            let semaphore = Arc::clone(&semaphore);
            let config = self.config.clone();
            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await?;
                let file_writer = FileWriter::new(config);
                file_writer.write(&path, &units).await?;
                Ok::<_, TranslateError>(WriteResult {
                    path: path.clone(),
                    success: true,
                    error: None,
                    units_written: units.len(),
                })
            });
            tasks.push(task);
        }

        let results: Result<Vec<_>, _> = try_join_all(tasks).await?;
        Ok(results.into_iter().collect::<Result<Vec<_>, _>>()?)
    }
}
```

**并发控制**：
- 信号量限制并发数
- 异步任务池
- 错误隔离

### 4. FormatProtector

**位置**：`src/writer/format/`

**职责**：
- 保护格式化字符串
- 识别和保护占位符
- 确保翻译后格式正确

**关键功能**：
```rust
pub struct FormatProtector {
    patterns: Vec<Regex>,
}

impl FormatProtector {
    pub fn protect(&self, content: &str) -> Result<String> {
        let mut protected = content.to_string();
        let mut placeholder_map = HashMap::new();

        for (idx, pattern) in self.patterns.iter().enumerate() {
            let matches: Vec<_> = pattern.find_iter(&protected).collect();
            for m in matches {
                let placeholder = format!("__PLACEHOLDER_{}__", idx);
                placeholder_map.insert(placeholder.clone(), m.as_str().to_string());
                protected = protected.replace(m.as_str(), &placeholder);
            }
        }

        Ok(protected)
    }

    pub fn restore(&self, protected: &str) -> Result<String> {
        let mut restored = protected.to_string();

        // 简单的占位符替换
        // 实际实现需要更复杂的逻辑
        Ok(restored)
    }
}
```

### 5. LineApplier

**位置**：`src/writer/applier/line.rs`

**职责**：
- 单行翻译应用
- 精确的字节偏移替换
- 行级处理

**关键功能**：
```rust
pub struct LineApplier;

impl LineApplier {
    pub fn apply_to_line(
        line: &str,
        units: &[TranslationUnit],
        line_num: usize,
    ) -> Result<String> {
        let mut result = line.to_string();

        // 获取当前行的所有翻译单元
        let line_units: Vec<_> = units
            .iter()
            .filter(|u| u.start_pos.line == line_num)
            .collect();

        // 逆序处理以避免偏移问题
        for unit in line_units.iter().rev() {
            if let Some(translation) = &unit.translation {
                let start = unit.start_pos.column - 1;
                let end = unit.end_pos.column;

                if start <= end && end <= result.len() {
                    result.replace_range(start..end, translation);
                }
            }
        }

        Ok(result)
    }
}
```

### 6. MultilineApplier

**位置**：`src/writer/applier/multiline.rs`

**职责**：
- 多行翻译应用
- 块注释处理
- 跨行内容替换

**关键功能**：
```rust
pub struct MultilineApplier;

impl MultilineApplier {
    pub fn apply_to_content(
        content: &str,
        units: &[TranslationUnit],
    ) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut lines_vec = lines.to_vec();

        // 逆序处理多行单元
        for unit in units.iter().rev() {
            if unit.start_pos.line == unit.end_pos.line {
                continue;  // 跳过单行单元
            }

            let start_line = unit.start_pos.line - 1;
            let end_line = unit.end_pos.line - 1;

            if end_line >= lines_vec.len() {
                continue;
            }

            if let Some(translation) = &unit.translation {
                // 替换多行内容
                lines_vec[start_line..=end_line].fill(translation);
            }
        }

        Ok(lines_vec.join("\n"))
    }
}
```

## 技术选型

### 异步 I/O
- **Tokio fs**：异步文件系统操作
  - 高性能异步 I/O
  - 非阻塞操作
  - 零成本抽象

### 原子操作
- **临时文件 + 重命名**：跨平台原子写入
  - 写入临时文件
  - 确保数据完整性
  - 原子重命名

### 并发控制
- **Tokio Semaphore**：并发限制
- **JoinSet**：任务管理
- **Arc<T>**：共享状态

### 序列化
- **UUID**：临时文件命名
  - 唯一性保证
  - 避免冲突

## 关键设计要点

### 1. 原子写入

```rust
async fn atomic_write(&self, path: &Path, content: &str) -> Result<()> {
    // 1. 创建临时文件
    let temp_path = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));

    // 2. 写入内容
    tokio::fs::write(&temp_path, content).await
        .map_err(|e| {
            // 清理临时文件
            let _ = std::fs::remove_file(&temp_path);
            TranslateError::Io(format!("Failed to write temp file: {}", e))
        })?;

    // 3. 原子重命名
    tokio::fs::rename(&temp_path, path).await
        .map_err(|e| {
            // 清理临时文件
            let _ = std::fs::remove_file(&temp_path);
            TranslateError::Io(format!("Failed to rename temp file: {}", e))
        })?;

    Ok(())
}
```

**优势**：
- 写入失败不影响原文件
- 原子操作保证一致性
- 跨平台兼容

### 2. 备份策略

```rust
async fn create_backup(&self, path: &Path) -> Result<()> {
    let backup_path = if let Some(ref backup_dir) = self.config.backup_dir {
        // 使用指定备份目录
        backup_dir.join(path.file_name().unwrap())
    } else {
        // 在原文件同目录创建备份
        let mut backup_path = path.to_path_buf();
        backup_path.set_extension(format!(
            "{}.bk",
            path.extension().unwrap().to_str().unwrap()
        ));
        backup_path
    };

    // 创建备份目录
    if let Some(parent) = backup_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // 复制文件
    tokio::fs::copy(path, &backup_path).await?;

    Ok(())
}
```

**备份命名**：
- 默认：`file.ext.bk`
- 自定义目录：`backup_dir/file.ext.bk`

### 3. 换行符处理

```rust
pub fn detect_line_ending(content: &str) -> &'static str {
    if content.contains("\r\n") {
        "\r\n"
    } else if content.contains('\r') {
        "\r"
    } else {
        "\n"
    }
}

pub fn normalize_line_ending(content: &str) -> String {
    content.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn restore_line_ending(content: &str, original: &str) -> String {
    let line_ending = detect_line_ending(original);
    if line_ending == "\r\n" {
        content.replace('\n', "\r\n")
    } else if line_ending == "\r" {
        content.replace('\n', "\r")
    } else {
        content.to_string()
    }
}
```

**处理流程**：
1. 检测原始换行符
2. 规范化为 LF
3. 处理内容
4. 恢复原始换行符

### 4. 字节偏移处理

```rust
pub fn replace_by_offset(
    content: &str,
    offset: usize,
    length: usize,
    replacement: &str,
) -> Result<String> {
    let bytes = content.as_bytes();

    if offset + length > bytes.len() {
        return Err("Offset out of bounds".into());
    }

    let mut result = String::with_capacity(content.len() - length + replacement.len());

    // 前置内容
    result.push_str(&content[..offset]);

    // 替换内容
    result.push_str(replacement);

    // 后置内容
    result.push_str(&content[offset + length..]);

    Ok(result)
}
```

**关键要点**：
- 使用字节偏移而非字符偏移
- 处理 UTF-8 编码
- 边界检查

### 5. 并发写入

```rust
pub async fn write_concurrent(
    &self,
    writes: Vec<(PathBuf, Vec<TranslationUnit>)>,
) -> Result<Vec<WriteResult>> {
    let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
    let mut results = Vec::new();

    let tasks: Vec<_> = writes.into_iter().map(|(path, units)| {
        let semaphore = Arc::clone(&semaphore);
        let writer = self.clone();

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await?;
            match writer.write(&path, &units).await {
                Ok(()) => Ok(WriteResult {
                    path: path.clone(),
                    success: true,
                    error: None,
                    units_written: units.len(),
                }),
                Err(e) => Ok(WriteResult {
                    path: path.clone(),
                    success: false,
                    error: Some(e.to_string()),
                    units_written: 0,
                }),
            }
        })
    }).collect();

    // 等待所有任务完成
    let task_results: Result<Vec<_>, _> = try_join_all(tasks).await?;

    // 收集结果
    for task_result in task_results {
        results.push(task_result??);
    }

    Ok(results)
}
```

**并发控制**：
- 信号量限制并发数
- 独立的错误处理
- 结果聚合

## 使用示例

### 基本写入

```rust
use codebase_translate::writer::{FileWriter, WriterConfig};

let config = WriterConfig {
    preview_only: false,
    backup: true,
    backup_dir: Some(Path::new(".translate/backups")),
    strict_encoding: false,
};

let writer = FileWriter::new(config);
writer.write(&path, &units).await?;
```

### 预览模式

```rust
let config = WriterConfig {
    preview_only: true,
    backup: false,
    backup_dir: None,
    strict_encoding: false,
};

let writer = FileWriter::new(config);
writer.write(&path, &units).await?;  // 只打印，不写入
```

### 并发写入

```rust
use codebase_translate::writer::{ConcurrentWriter, WriterConfig};

let config = WriterConfig::default();
let writer = ConcurrentWriter::new(config, 4);  // 最大并发 4

let writes = vec![
    (path1, units1),
    (path2, units2),
    (path3, units3),
];

let results = writer.write_batch(writes).await?;
for result in results {
    println!("{:?}: {}", result.path, result.success);
}
```

### 应用翻译

```rust
use codebase_translate::writer::TranslationApplier;

let content = "fn main() {
    // 这是注释
    println!(\"Hello\");
}";

let translated = TranslationApplier::apply_translations(content, &units)?;
```

## 性能考量

1. **原子写入**：
   - 临时文件写入
   - 避免文件锁
   - 快速重命名

2. **并发优化**：
   - 信号量控制
   - 独立任务
   - 错误隔离

3. **内存效率**：
   - 流式处理
   - 及时释放
   - 避免克隆

4. **I/O 优化**：
   - 批量写入
   - 缓冲区优化
   - 异步 I/O

## 扩展性

1. **新的写入策略**：
   - 增量写入
   - 差异写入
   - 压缩写入

2. **高级备份**：
   - 版本控制
   - 增量备份
   - 远程备份

3. **格式保护**：
   - 语法高亮
   - 代码格式化
   - Lint 检查

4. **集成第三方**：
   - Git 集成
   - IDE 集成
   - CI/CD 集成