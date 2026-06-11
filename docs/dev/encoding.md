# Encoding Module Design

## 概述

Encoding 模块提供文件编码检测和转换功能，支持多种字符编码的自动检测和 UTF-8 统一转换，确保整个翻译系统在统一的 UTF-8 环境下运行。

## 设计目的

1. **统一编码**：将所有文件转换为 UTF-8 编码，避免编码问题
2. **自动检测**：智能检测文件编码，减少用户配置
3. **BOM 处理**：自动检测和移除字节顺序标记
4. **高精度**：提供置信度评分，确保检测准确性

## 核心组件

### 1. Detector

**位置**：`src/encoding/detector.rs`

**职责**：
- 检测文件的字符编码
- 提供置信度评分
- 支持 BOM 检测

**关键功能**：
```rust
pub struct Detector {
    config: DetectorConfig,
}

impl Detector {
    pub fn detect_file(&self, path: &Path) -> Result<EncodingResult> {
        // 返回编码类型和置信度
    }

    pub fn detect_bytes(&self, data: &[u8]) -> Result<EncodingResult> {
        // 检测字节序列的编码
    }
}
```

**检测流程**：
1. **BOM 检测**：优先检查字节顺序标记
2. **启发式检测**：使用 chardetng 库进行编码检测
3. **置信度评估**：返回检测置信度（0.0-1.0）
4. **候选编码排序**：按置信度排序返回多个候选

**支持编码**：
- UTF-8 (with/without BOM)
- UTF-16LE / UTF-16BE
- GBK / GB18030
- Big5
- Shift_JIS
- EUC-JP
- ISO-8859-1 / Latin-1

**关键设计要点**：
- 多候选编码支持
- 置信度阈值过滤
- 并行检测提高速度
- 源信息追踪用于错误报告

### 2. Encoder

**位置**：`src/encoding/encoder.rs`

**职责**：
- 编码转换
- BOM 移除
- 严格模式验证

**关键功能**：
```rust
pub struct Encoder {
    config: EncoderConfig,
}

impl Encoder {
    pub fn to_utf8(&self, data: &[u8], from_encoding: &str) -> Result<String> {
        // 转换为 UTF-8
    }

    pub fn convert_file_to_utf8(&self, path: &Path, from_encoding: &str) -> Result<()> {
        // 转换文件为 UTF-8
    }
}
```

**转换流程**：
1. **编码解析**：解析源编码名称
2. **BOM 移除**：移除字节顺序标记
3. **编码转换**：使用 encoding_rs 库进行转换
4. **严格验证**：严格模式下验证无效字符
5. **规范化**：统一换行符为 LF

**关键设计要点**：
- 使用 encoding_rs 库（Firefox 的编码实现）
- 支持严格模式（检测无效字符时失败）
- 自动 BOM 处理
- 换行符规范化

### 3. 配置

**DetectorConfig**：
```rust
pub struct DetectorConfig {
    pub detect_encodings: Vec<String>,  // 要检测的编码列表
    pub min_confidence: f64,            // 最小置信度阈值
    pub enable_bom_detection: bool,     // 启用 BOM 检测
    pub parallel_detection: bool,       // 并行检测
}
```

**EncoderConfig**：
```rust
pub struct EncoderConfig {
    pub normalize_line_endings: bool,   // 规范化换行符
    pub strict_mode: bool,              // 严格模式
    pub preserve_bom: bool,             // 保留 BOM
}
```

### 4. 类型定义

**位置**：`src/encoding/types.rs`

**关键类型**：
```rust
pub enum EncodingType {
    UTF8,
    UTF16LE,
    UTF16BE,
    GBK,
    Big5,
    ShiftJIS,
    EUCJP,
    Latin1,
}

pub struct EncodingResult {
    pub encoding: EncodingType,
    pub confidence: f64,           // 置信度 0.0-1.0
    pub bom_detected: bool,        // 是否检测到 BOM
    pub candidates: Vec<EncodingType>,  // 候选编码列表
}
```

## 技术选型

### 编码检测库
- **chardetng**：高性能编码检测
  - Rust 原生实现
  - 基于 Mozilla 的通用编码检测器
  - 支持多种编码
  - 提供置信度评分

### 编码转换库
- **encoding_rs**：高效的编码转换
  - Firefox 使用的编码实现
  - 零分配转换
  - 支持 FFI 兼容性
  - 广泛的编码支持

### 并行处理
- **Rayon**：数据并行库
  - 用于并行编码检测
  - 简单的 API
  - 自动工作窃取

## 关键设计要点

### 1. 检测策略

**优先级顺序**：
1. **BOM 检测**（最高优先级）
   - UTF-8 BOM: `EF BB BF`
   - UTF-16LE BOM: `FF FE`
   - UTF-16BE BOM: `FE FF`

2. **启发式检测**
   - 分析字节频率
   - 检查编码特征模式
   - 评估置信度

3. **多候选返回**
   - 返回多个候选编码
   - 按置信度排序
   - 允许用户选择

**示例**：
```rust
let result = detector.detect_file("file.txt")?;
println!("编码: {:?}, 置信度: {:.2}", result.encoding, result.confidence);
```

### 2. 转换保证

**UTF-8 转换**：
```rust
// GBK -> UTF-8
encoder.to_utf8(&gbk_bytes, "GBK")?;

// 自动 BOM 移除
encoder.to_utf8(&utf8_with_bom, "UTF-8")?;

// 严格模式验证
let strict_encoder = Encoder::new(EncoderConfig {
    strict_mode: true,
    ..
});
```

**错误处理**：
```rust
pub enum Error {
    UnsupportedEncoding(String),
    InvalidData(String),
    FileNotFound(String),
    ConversionFailed(String),
}
```

### 3. 性能优化

**并行检测**：
```rust
impl Detector {
    pub fn detect_bytes(&self, data: &[u8]) -> Result<EncodingResult> {
        // 并行检测多个编码
        let results: Vec<_> = self.config.detect_encodings
            .par_iter()
            .map(|enc| self.try_encoding(data, enc))
            .collect();

        // 选择置信度最高的
        self.select_best_result(results)
    }
}
```

**缓存优化**：
- 检测结果缓存
- 编码器实例复用

### 4. BOM 处理

**检测 BOM**：
```rust
fn detect_bom(data: &[u8]) -> Option<EncodingType> {
    match &data[0..3] {
        [0xEF, 0xBB, 0xBF] => Some(EncodingType::UTF8),
        [0xFF, 0xFE, ..] => Some(EncodingType::UTF16LE),
        [0xFE, 0xFF, ..] => Some(EncodingType::UTF16BE),
        _ => None,
    }
}
```

**移除 BOM**：
```rust
fn remove_bom(data: &[u8]) -> &[u8] {
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &data[3..]
    } else if data.starts_with(&[0xFF, 0xFE]) || data.starts_with(&[0xFE, 0xFF]) {
        &data[2..]
    } else {
        data
    }
}
```

### 5. 换行符规范化

```rust
fn normalize_line_endings(content: &str) -> String {
    content.replace("\r\n", "\n").replace('\r', "\n")
}
```

**规范化策略**：
- CRLF (\r\n) -> LF (\n)
- CR (\r) -> LF (\n)
- LF (\n) 保持不变

## 使用示例

### 基本使用

```rust
use codebase_translate::encoding::{Detector, Encoder};

// 检测文件编码
let detector = Detector::default();
let result = detector.detect_file(Path::new("test.txt"))?;

println!("编码: {:?}", result.encoding);
println!("置信度: {:.2}", result.confidence);

// 转换为 UTF-8
let encoder = Encoder::default();
encoder.convert_file_to_utf8(Path::new("test.txt"), &result.encoding.to_string())?;
```

### 高级使用

```rust
use codebase_translate::encoding::{Detector, Encoder, DetectorConfig, EncoderConfig};

// 自定义检测配置
let detector_config = DetectorConfig {
    detect_encodings: vec!["UTF-8".to_string(), "GBK".to_string()],
    min_confidence: 0.7,
    enable_bom_detection: true,
    parallel_detection: true,
};

let detector = Detector::new(detector_config);

// 严格模式转换
let encoder_config = EncoderConfig {
    normalize_line_endings: true,
    strict_mode: true,
    preserve_bom: false,
};

let encoder = Encoder::new(encoder_config);
```

### 批量处理

```rust
let detector = Detector::default();
let encoder = Encoder::default();

for file_path in file_paths {
    let result = detector.detect_file(&file_path)?;
    encoder.convert_file_to_utf8(&file_path, &result.encoding.to_string())?;
}
```

## 性能考量

1. **检测速度**：
   - BOM 检测：O(1)
   - 启发式检测：O(n)
   - 并行检测：O(n/p) 其中 p 为并行度

2. **转换速度**：
   - encoding_rs：零分配，极快
   - 内存映射大文件：避免完全加载

3. **内存使用**：
   - 小文件：完全加载到内存
   - 大文件：流式处理（待实现）

## 扩展性

1. **新的编码支持**：
   - 添加新的编码类型到 EncodingType
   - 实现对应的检测逻辑
   - 注册到编码器

2. **高级检测**：
   - 机器学习检测
   - 基于文件扩展名的启发式
   - 用户自定义检测规则

3. **增强验证**：
   - 语法验证（如 XML 声明）
   - 内容验证（如中文字符验证）
   - 混合编码检测

4. **缓存机制**：
   - 检测结果缓存
   - 编码信息持久化
   - 增量检测