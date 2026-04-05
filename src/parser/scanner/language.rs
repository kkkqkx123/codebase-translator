//! Language-specific configuration for text scanning
//!
//! Defines comment delimiters, string quotes, and other language-specific
//! syntax elements needed for text region extraction.

use std::collections::HashSet;

/// Language-specific configuration
#[derive(Debug, Clone)]
pub struct ScannerLanguageConfig {
    /// Line comment prefixes (longest first for matching)
    pub line_comment_prefixes: Vec<&'static str>,
    /// Block comment delimiters (start, end)
    pub block_comment_delimiters: Vec<(&'static str, &'static str)>,
    /// Doc comment prefixes (longest first for matching)
    pub doc_comment_prefixes: Vec<&'static str>,
    /// String quotes
    pub string_quotes: Vec<char>,
    /// Template string quote
    pub template_quote: Option<char>,
    /// Raw string prefixes
    pub raw_string_prefixes: Vec<&'static str>,
    /// Multi-line string delimiters
    pub multiline_delimiters: Vec<&'static str>,
    /// Language name
    pub name: &'static str,
    /// Supported file extensions
    pub extensions: Vec<&'static str>,
}

impl ScannerLanguageConfig {
    pub fn javascript() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec!["/**", "///"],
            string_quotes: vec!['"', '\''],
            template_quote: Some('`'),
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "javascript",
            extensions: vec!["js", "mjs", "cjs"],
        }
    }

    pub fn typescript() -> Self {
        Self {
            extensions: vec!["ts", "tsx", "mts", "cts"],
            ..Self::javascript()
        }
    }

    pub fn jsx() -> Self {
        Self {
            extensions: vec!["jsx"],
            ..Self::javascript()
        }
    }

    pub fn python() -> Self {
        Self {
            line_comment_prefixes: vec!["#"],
            block_comment_delimiters: vec![],
            doc_comment_prefixes: vec!["\"\"\"", "'''"],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec!["r\"", "r'", "r\"\"\"", "r'''"],
            multiline_delimiters: vec!["\"\"\"", "'''"],
            name: "python",
            extensions: vec!["py", "pyw", "pyi"],
        }
    }

    pub fn rust() -> Self {
        Self {
            line_comment_prefixes: vec!["///", "//"],
            block_comment_delimiters: vec![("/**", "*/"), ("/*", "*/")],
            doc_comment_prefixes: vec!["///", "/**"],
            string_quotes: vec!['"'],
            template_quote: None,
            raw_string_prefixes: vec!["r#", "r\""],
            multiline_delimiters: vec![],
            name: "rust",
            extensions: vec!["rs"],
        }
    }

    pub fn go() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['"', '\''],
            template_quote: Some('`'),
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "go",
            extensions: vec!["go"],
        }
    }

    pub fn java() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec!["/**"],
            string_quotes: vec!['"'],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "java",
            extensions: vec!["java"],
        }
    }

    pub fn c() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "c",
            extensions: vec!["c", "h"],
        }
    }

    pub fn cpp() -> Self {
        Self {
            doc_comment_prefixes: vec!["///", "/**"],
            extensions: vec!["cpp", "cc", "cxx", "hpp", "hh", "hxx"],
            ..Self::c()
        }
    }

    pub fn csharp() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec!["///", "/**"],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec!["@\"", "@'"],
            multiline_delimiters: vec![],
            name: "csharp",
            extensions: vec!["cs"],
        }
    }

    pub fn kotlin() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec!["/**"],
            string_quotes: vec!['"'],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec!["\"\"\""],
            name: "kotlin",
            extensions: vec!["kt", "kts"],
        }
    }

    pub fn swift() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec!["///", "/**"],
            string_quotes: vec!['"'],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec!["\"\"\""],
            name: "swift",
            extensions: vec!["swift"],
        }
    }

    pub fn ruby() -> Self {
        Self {
            line_comment_prefixes: vec!["#"],
            block_comment_delimiters: vec![("=begin", "=end")],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "ruby",
            extensions: vec!["rb", "rake"],
        }
    }

    pub fn php() -> Self {
        Self {
            line_comment_prefixes: vec!["//", "#"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec!["/**"],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "php",
            extensions: vec!["php"],
        }
    }

    pub fn lua() -> Self {
        Self {
            line_comment_prefixes: vec!["--"],
            block_comment_delimiters: vec![("--[[", "]]")],
            doc_comment_prefixes: vec!["---"],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec!["[[", "]]"],
            name: "lua",
            extensions: vec!["lua"],
        }
    }

    pub fn scala() -> Self {
        Self {
            line_comment_prefixes: vec!["//"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['"'],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec!["\"\"\""],
            name: "scala",
            extensions: vec!["scala", "sc"],
        }
    }

    pub fn all_languages() -> Vec<Self> {
        vec![
            Self::javascript(),
            Self::typescript(),
            Self::jsx(),
            Self::python(),
            Self::rust(),
            Self::go(),
            Self::java(),
            Self::c(),
            Self::cpp(),
            Self::csharp(),
            Self::kotlin(),
            Self::swift(),
            Self::ruby(),
            Self::php(),
            Self::lua(),
            Self::scala(),
        ]
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext_lower = ext.to_lowercase();
        Self::all_languages().into_iter().find(|lang| {
            lang.extensions.iter().any(|e| e.to_lowercase() == ext_lower)
        })
    }

    pub fn supports(&self, filename: &str) -> bool {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        self.extensions.iter().any(|e| e.to_lowercase() == ext.to_lowercase())
    }

    pub fn all_extensions() -> HashSet<&'static str> {
        Self::all_languages()
            .iter()
            .flat_map(|lang| lang.extensions.iter().copied())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_javascript_config() {
        let config = ScannerLanguageConfig::javascript();
        assert_eq!(config.name, "javascript");
        assert!(config.line_comment_prefixes.contains(&"//"));
        assert!(config.template_quote.is_some());
        assert!(config.supports("test.js"));
    }

    #[test]
    fn test_python_config() {
        let config = ScannerLanguageConfig::python();
        assert_eq!(config.name, "python");
        assert!(config.line_comment_prefixes.contains(&"#"));
        assert!(config.multiline_delimiters.contains(&"\"\"\""));
        assert!(config.supports("test.py"));
    }

    #[test]
    fn test_rust_config() {
        let config = ScannerLanguageConfig::rust();
        assert_eq!(config.name, "rust");
        assert!(config.doc_comment_prefixes.contains(&"///"));
        assert!(config.raw_string_prefixes.contains(&"r#"));
        assert!(config.supports("main.rs"));
    }

    #[test]
    fn test_from_extension() {
        assert!(ScannerLanguageConfig::from_extension("js").is_some());
        assert!(ScannerLanguageConfig::from_extension("ts").is_some());
        assert!(ScannerLanguageConfig::from_extension("py").is_some());
        assert!(ScannerLanguageConfig::from_extension("rs").is_some());
        assert!(ScannerLanguageConfig::from_extension("unknown").is_none());
    }

    #[test]
    fn test_all_extensions() {
        let extensions = ScannerLanguageConfig::all_extensions();
        assert!(extensions.contains("js"));
        assert!(extensions.contains("ts"));
        assert!(extensions.contains("py"));
        assert!(extensions.contains("rs"));
        assert!(extensions.contains("go"));
        assert!(extensions.contains("java"));
    }
}
