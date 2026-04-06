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
            line_comment_prefixes: vec!["//"],
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

    pub fn shell() -> Self {
        Self {
            line_comment_prefixes: vec!["#"],
            block_comment_delimiters: vec![("<#", "#>")],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "shell",
            extensions: vec!["sh", "bash", "zsh", "fish"],
        }
    }

    pub fn powershell() -> Self {
        Self {
            line_comment_prefixes: vec!["#"],
            block_comment_delimiters: vec![("<#", "#>")],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "powershell",
            extensions: vec!["ps1", "psm1", "psd1"],
        }
    }

    pub fn batch() -> Self {
        Self {
            line_comment_prefixes: vec!["REM", "::"],
            block_comment_delimiters: vec![],
            doc_comment_prefixes: vec![],
            string_quotes: vec![],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "batch",
            extensions: vec!["bat", "cmd"],
        }
    }

    pub fn sql() -> Self {
        Self {
            line_comment_prefixes: vec!["--"],
            block_comment_delimiters: vec![("/*", "*/")],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['\''],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "sql",
            extensions: vec!["sql", "mysql", "pgsql"],
        }
    }

    pub fn html() -> Self {
        Self {
            line_comment_prefixes: vec![],
            block_comment_delimiters: vec![("<!--", "-->")],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "html",
            extensions: vec!["html", "htm", "xml", "svg"],
        }
    }

    pub fn markdown() -> Self {
        Self {
            line_comment_prefixes: vec![],
            block_comment_delimiters: vec![("<!--", "-->")],
            doc_comment_prefixes: vec![],
            string_quotes: vec![],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "markdown",
            extensions: vec!["md", "markdown"],
        }
    }

    pub fn config() -> Self {
        Self {
            line_comment_prefixes: vec!["#", ";"],
            block_comment_delimiters: vec![],
            doc_comment_prefixes: vec![],
            string_quotes: vec!['"', '\''],
            template_quote: None,
            raw_string_prefixes: vec![],
            multiline_delimiters: vec![],
            name: "config",
            extensions: vec!["yaml", "yml", "toml", "ini", "conf", "txt"],
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
            Self::shell(),
            Self::powershell(),
            Self::batch(),
            Self::sql(),
            Self::html(),
            Self::markdown(),
            Self::config(),
        ]
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext_lower = ext.to_lowercase();
        Self::all_languages().into_iter().find(|lang| {
            lang.extensions
                .iter()
                .any(|e| e.to_lowercase() == ext_lower)
        })
    }

    pub fn supports(&self, filename: &str) -> bool {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        self.extensions
            .iter()
            .any(|e| e.to_lowercase() == ext.to_lowercase())
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
    fn test_shell_config() {
        let config = ScannerLanguageConfig::shell();
        assert_eq!(config.name, "shell");
        assert!(config.line_comment_prefixes.contains(&"#"));
        assert!(config.supports("script.sh"));
        assert!(config.supports("script.bash"));
    }

    #[test]
    fn test_sql_config() {
        let config = ScannerLanguageConfig::sql();
        assert_eq!(config.name, "sql");
        assert!(config.line_comment_prefixes.contains(&"--"));
        assert!(config.supports("query.sql"));
    }

    #[test]
    fn test_html_config() {
        let config = ScannerLanguageConfig::html();
        assert_eq!(config.name, "html");
        assert!(config.block_comment_delimiters.contains(&("<!--", "-->")));
        assert!(config.supports("page.html"));
        assert!(config.supports("config.xml"));
    }

    #[test]
    fn test_markdown_config() {
        let config = ScannerLanguageConfig::markdown();
        assert_eq!(config.name, "markdown");
        assert!(config.supports("readme.md"));
    }

    #[test]
    fn test_config_config() {
        let config = ScannerLanguageConfig::config();
        assert_eq!(config.name, "config");
        assert!(config.line_comment_prefixes.contains(&"#"));
        assert!(config.supports("config.yaml"));
        assert!(config.supports("settings.toml"));
    }

    #[test]
    fn test_from_extension() {
        assert!(ScannerLanguageConfig::from_extension("js").is_some());
        assert!(ScannerLanguageConfig::from_extension("ts").is_some());
        assert!(ScannerLanguageConfig::from_extension("py").is_some());
        assert!(ScannerLanguageConfig::from_extension("rs").is_some());
        assert!(ScannerLanguageConfig::from_extension("sh").is_some());
        assert!(ScannerLanguageConfig::from_extension("sql").is_some());
        assert!(ScannerLanguageConfig::from_extension("html").is_some());
        assert!(ScannerLanguageConfig::from_extension("md").is_some());
        assert!(ScannerLanguageConfig::from_extension("yaml").is_some());
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
        assert!(extensions.contains("sh"));
        assert!(extensions.contains("sql"));
        assert!(extensions.contains("html"));
        assert!(extensions.contains("md"));
        assert!(extensions.contains("yaml"));
    }
}
