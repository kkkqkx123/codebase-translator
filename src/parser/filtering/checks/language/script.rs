//! Script type definitions and utilities

/// Script type for language detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Script {
    /// Unknown script
    #[default]
    Unknown,
    /// Latin script (English, German, French, etc.)
    Latin,
    /// CJK script (Chinese, Japanese, Korean)
    Cjk,
    /// Arabic script
    Arabic,
    /// Hebrew script
    Hebrew,
    /// Greek script
    Greek,
    /// Cyrillic script (Russian, Ukrainian, etc.)
    Cyrillic,
}

impl Script {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Latin => "Latin",
            Self::Cjk => "CJK",
            Self::Arabic => "Arabic",
            Self::Hebrew => "Hebrew",
            Self::Greek => "Greek",
            Self::Cyrillic => "Cyrillic",
            Self::Unknown => "Unknown",
        }
    }

    /// Convert language code to script
    pub fn from_lang_code(lang: &str) -> Option<Self> {
        match lang.to_uppercase().as_str() {
            "ZH" | "JA" | "KO" | "HANS" | "HANT" | "ZH-CN" | "ZH-TW" | "EN-US" | "EN-GB" => {
                Some(Self::Cjk)
            }
            "AR" => Some(Self::Arabic),
            "HE" => Some(Self::Hebrew),
            "EL" => Some(Self::Greek),
            "RU" | "UK" | "BG" | "SR" | "BE" => Some(Self::Cyrillic),
            "EN" | "DE" | "FR" | "ES" | "IT" | "PT" | "NL" | "SV" | "PL" | "TR" | "CS" | "RO"
            | "HU" | "DA" | "NO" => Some(Self::Latin),
            _ => None,
        }
    }
}

impl std::fmt::Display for Script {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_display() {
        assert_eq!(Script::Latin.to_string(), "Latin");
        assert_eq!(Script::Cjk.to_string(), "CJK");
        assert_eq!(Script::Arabic.to_string(), "Arabic");
        assert_eq!(Script::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_from_lang_code() {
        assert_eq!(Script::from_lang_code("EN"), Some(Script::Latin));
        assert_eq!(Script::from_lang_code("ZH"), Some(Script::Cjk));
        assert_eq!(Script::from_lang_code("JA"), Some(Script::Cjk));
        assert_eq!(Script::from_lang_code("AR"), Some(Script::Arabic));
        assert_eq!(Script::from_lang_code("RU"), Some(Script::Cyrillic));
        assert_eq!(Script::from_lang_code("UNKNOWN"), None);
    }
}
