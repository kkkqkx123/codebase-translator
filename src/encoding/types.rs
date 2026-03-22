use serde::{Deserialize, Serialize};

use crate::encoding::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingResult {
    pub encoding: String,
    pub confidence: f64,
}

impl EncodingResult {
    pub fn new(encoding: impl Into<String>, confidence: f64) -> Self {
        Self {
            encoding: encoding.into(),
            confidence,
        }
    }

    pub fn is_confident(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorConfig {
    pub detect_encodings: Vec<String>,
    pub min_confidence: f64,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            detect_encodings: vec![
                "UTF-8".to_string(),
                "GBK".to_string(),
                "Big5".to_string(),
                "Shift_JIS".to_string(),
            ],
            min_confidence: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderConfig {
    pub remove_bom: bool,
    pub strict_mode: bool,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            remove_bom: true,
            strict_mode: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncodingType {
    UTF8,
    UTF16LE,
    UTF16BE,
    GBK,
    Big5,
    ShiftJIS,
    GB18030,
}

impl EncodingType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "UTF-8" | "UTF8" => Some(Self::UTF8),
            "UTF-16LE" | "UTF16LE" => Some(Self::UTF16LE),
            "UTF-16BE" | "UTF16BE" => Some(Self::UTF16BE),
            "GBK" => Some(Self::GBK),
            "BIG5" => Some(Self::Big5),
            "SHIFT_JIS" | "SHIFTJIS" => Some(Self::ShiftJIS),
            "GB18030" => Some(Self::GB18030),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UTF8 => "UTF-8",
            Self::UTF16LE => "UTF-16LE",
            Self::UTF16BE => "UTF-16BE",
            Self::GBK => "GBK",
            Self::Big5 => "Big5",
            Self::ShiftJIS => "Shift_JIS",
            Self::GB18030 => "GB18030",
        }
    }

    pub fn to_encoding_rs(&self) -> &'static encoding_rs::Encoding {
        match self {
            Self::UTF8 => encoding_rs::UTF_8,
            Self::UTF16LE => encoding_rs::UTF_16LE,
            Self::UTF16BE => encoding_rs::UTF_16BE,
            Self::GBK | Self::GB18030 => encoding_rs::GBK,
            Self::Big5 => encoding_rs::BIG5,
            Self::ShiftJIS => encoding_rs::SHIFT_JIS,
        }
    }
}

impl std::fmt::Display for EncodingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_result() {
        let result = EncodingResult::new("UTF-8", 0.95);
        assert_eq!(result.encoding, "UTF-8");
        assert_eq!(result.confidence, 0.95);
        assert!(result.is_confident(0.7));
        assert!(!result.is_confident(0.99));
    }

    #[test]
    fn test_detector_config_default() {
        let config = DetectorConfig::default();
        assert_eq!(config.min_confidence, 0.7);
        assert_eq!(config.detect_encodings.len(), 4);
        assert!(config.detect_encodings.contains(&"UTF-8".to_string()));
    }

    #[test]
    fn test_encoder_config_default() {
        let config = EncoderConfig::default();
        assert!(config.remove_bom);
        assert!(!config.strict_mode);
    }

    #[test]
    fn test_encoding_type_from_str() {
        assert_eq!(EncodingType::from_str("UTF-8"), Some(EncodingType::UTF8));
        assert_eq!(EncodingType::from_str("utf8"), Some(EncodingType::UTF8));
        assert_eq!(EncodingType::from_str("GBK"), Some(EncodingType::GBK));
        assert_eq!(EncodingType::from_str("Big5"), Some(EncodingType::Big5));
        assert_eq!(
            EncodingType::from_str("Shift_JIS"),
            Some(EncodingType::ShiftJIS)
        );
        assert_eq!(
            EncodingType::from_str("ShiftJIS"),
            Some(EncodingType::ShiftJIS)
        );
        assert_eq!(
            EncodingType::from_str("GB18030"),
            Some(EncodingType::GB18030)
        );
        assert_eq!(EncodingType::from_str("UNKNOWN"), None);
    }

    #[test]
    fn test_encoding_type_as_str() {
        assert_eq!(EncodingType::UTF8.as_str(), "UTF-8");
        assert_eq!(EncodingType::UTF16LE.as_str(), "UTF-16LE");
        assert_eq!(EncodingType::GBK.as_str(), "GBK");
        assert_eq!(EncodingType::Big5.as_str(), "Big5");
        assert_eq!(EncodingType::ShiftJIS.as_str(), "Shift_JIS");
    }

    #[test]
    fn test_encoding_type_display() {
        assert_eq!(format!("{}", EncodingType::UTF8), "UTF-8");
        assert_eq!(format!("{}", EncodingType::GBK), "GBK");
    }
}
