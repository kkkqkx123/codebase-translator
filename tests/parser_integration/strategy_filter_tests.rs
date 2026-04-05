//! Strategy and Filter Integration Tests
//!
//! Tests for extraction strategies and content filtering.

use std::path::PathBuf;
use std::sync::Arc;

use codebase_translate::core::models::{File, NodeType};
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::filtering::{ContentFilter, FilterConfig};
use codebase_translate::parser::core::{ExtractionConfig, StrategyNodeType};
use codebase_translate::parser::ParserConfig;

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

mod strategy_tests {
    use super::*;

    #[test]
    fn test_extraction_config_comments_only() {
        let config = ExtractionConfig {
            comments: true,
            docstrings: false,
            error_messages: false,
            format_strings: false,
            log_messages: false,
            ..Default::default()
        };

        assert!(
            config.should_extract(StrategyNodeType::Comment),
            "Should extract comments"
        );
        assert!(
            !config.should_extract(StrategyNodeType::DocString),
            "Should not extract docstrings"
        );
    }

    #[test]
    fn test_extraction_config_docstrings_only() {
        let config = ExtractionConfig {
            comments: false,
            docstrings: true,
            error_messages: false,
            format_strings: false,
            log_messages: false,
            ..Default::default()
        };

        assert!(
            !config.should_extract(StrategyNodeType::Comment),
            "Should not extract comments"
        );
        assert!(
            config.should_extract(StrategyNodeType::DocString),
            "Should extract docstrings"
        );
    }

    #[test]
    fn test_extraction_config_error_messages() {
        let config = ExtractionConfig {
            comments: false,
            docstrings: false,
            error_messages: true,
            format_strings: false,
            log_messages: false,
            ..Default::default()
        };

        assert!(
            config.should_extract(StrategyNodeType::ErrorMessage),
            "Should extract error messages"
        );
        assert!(
            !config.should_extract(StrategyNodeType::LogMessage),
            "Should not extract log messages"
        );
    }

    #[test]
    fn test_extraction_config_all_enabled() {
        let config = ExtractionConfig {
            comments: true,
            docstrings: true,
            error_messages: true,
            format_strings: true,
            log_messages: true,
            ..Default::default()
        };

        assert!(config.should_extract(StrategyNodeType::Comment));
        assert!(config.should_extract(StrategyNodeType::DocString));
        assert!(config.should_extract(StrategyNodeType::ErrorMessage));
        assert!(config.should_extract(StrategyNodeType::FormatString));
        assert!(config.should_extract(StrategyNodeType::LogMessage));
    }

    #[test]
    fn test_extraction_config_node_type_mapping() {
        let config = ExtractionConfig::default();

        assert_eq!(
            config.get_node_type(StrategyNodeType::Comment),
            NodeType::Comment
        );
        assert_eq!(
            config.get_node_type(StrategyNodeType::DocString),
            NodeType::DocString
        );
        assert_eq!(
            config.get_node_type(StrategyNodeType::ErrorMessage),
            NodeType::ErrorMessage
        );
        assert_eq!(
            config.get_node_type(StrategyNodeType::FormatString),
            NodeType::FormatString
        );
        assert_eq!(
            config.get_node_type(StrategyNodeType::LogMessage),
            NodeType::LogMessage
        );
    }
}

mod filter_tests {
    use super::*;

    #[test]
    fn test_content_filter_default_exclusions() {
        let filter = ContentFilter::default();

        assert!(
            !filter.should_translate("TODO: fix this"),
            "Should filter TODO comments"
        );
        assert!(
            !filter.should_translate("FIXME: broken code"),
            "Should filter FIXME comments"
        );
        assert!(
            !filter.should_translate("NOTE: important"),
            "Should filter NOTE comments"
        );
        assert!(
            !filter.should_translate("XXX: hack"),
            "Should filter XXX comments"
        );
        assert!(
            !filter.should_translate("Copyright 2024"),
            "Should filter Copyright"
        );
        assert!(
            !filter.should_translate("Licensed under MIT"),
            "Should filter Licensed"
        );
    }

    #[test]
    fn test_content_filter_url_exclusion() {
        let filter = ContentFilter::default();

        assert!(
            !filter.should_translate("Visit https://example.com for more info"),
            "Should filter URLs"
        );
        assert!(
            !filter.should_translate("Check http://test.org"),
            "Should filter HTTP URLs"
        );
    }

    #[test]
    fn test_content_filter_email_exclusion() {
        let filter = ContentFilter::default();

        assert!(
            !filter.should_translate("Contact us at test@example.com"),
            "Should filter email addresses"
        );
    }

    #[test]
    fn test_content_filter_markdown_exclusion() {
        let filter = ContentFilter::default();

        assert!(
            !filter.should_translate("See [link](https://example.com)"),
            "Should filter markdown links"
        );
        assert!(
            !filter.should_translate("![image](image.png)"),
            "Should filter markdown images"
        );
    }

    #[test]
    fn test_content_filter_length_limits() {
        let config = FilterConfig {
            min_length: 10,
            max_length: 50,
            ..Default::default()
        };
        let filter = ContentFilter::new(config).expect("Failed to create filter");

        assert!(
            !filter.should_translate("Short"),
            "Should filter content below min_length"
        );
        assert!(
            filter.should_translate("This is a good length comment"),
            "Should allow content within length limits"
        );
        assert!(
            !filter.should_translate("a".repeat(100).as_str()),
            "Should filter content above max_length"
        );
    }

    #[test]
    fn test_content_filter_placeholder_detection() {
        let config = FilterConfig {
            allow_placeholders: false,
            ..Default::default()
        };
        let filter = ContentFilter::new(config).expect("Failed to create filter");

        assert!(
            !filter.should_translate("Hello %s"),
            "Should filter printf-style placeholders"
        );
        assert!(
            !filter.should_translate("Value: $1"),
            "Should filter positional placeholders"
        );
        assert!(
            !filter.should_translate("Name: ${name}"),
            "Should filter named placeholders"
        );
    }

    #[test]
    fn test_content_filter_placeholder_allowed() {
        let config = FilterConfig {
            allow_placeholders: true,
            ..Default::default()
        };
        let filter = ContentFilter::new(config).expect("Failed to create filter");

        assert!(
            filter.should_translate("Hello %s"),
            "Should allow placeholders when configured"
        );
    }

    #[test]
    fn test_content_filter_include_patterns() {
        let config = FilterConfig {
            include_patterns: vec![r"translate.*".to_string()],
            ..Default::default()
        };
        let filter = ContentFilter::new(config).expect("Failed to create filter");

        assert!(
            filter.should_translate("translate this please"),
            "Should include matching patterns"
        );
        assert!(
            !filter.should_translate("do not translate this"),
            "Should exclude non-matching patterns when include is set"
        );
    }

    #[test]
    fn test_content_filter_custom_exclude_patterns() {
        let config = FilterConfig {
            exclude_patterns: vec![r"@\w+".to_string()],
            ..Default::default()
        };
        let filter = ContentFilter::new(config).expect("Failed to create filter");

        assert!(
            !filter.should_translate("Contact @username for help"),
            "Should filter custom exclude patterns"
        );
        assert!(
            filter.should_translate("Regular comment without at-mention"),
            "Should allow non-matching content"
        );
    }

    #[test]
    fn test_content_filter_code_pattern_detection() {
        let filter = ContentFilter::default();

        assert!(
            !filter.should_translate("const x = 5;"),
            "Should filter code-like patterns"
        );
        assert!(
            !filter.should_translate("function test() {}"),
            "Should filter function definitions"
        );
        assert!(
            !filter.should_translate("if (true) { return; }"),
            "Should filter control flow statements"
        );
    }

    #[test]
    fn test_content_filter_normal_text() {
        let filter = ContentFilter::default();

        assert!(
            filter.should_translate("This is a normal comment that should be translated"),
            "Should allow normal text"
        );
        assert!(
            filter.should_translate("This function calculates the sum of two numbers"),
            "Should allow descriptive comments"
        );
    }
}

mod integration_tests {
    use super::*;

    #[test]
    fn test_strategy_and_filter_integration() {
        let strategy_config = ExtractionConfig {
            comments: true,
            docstrings: true,
            error_messages: false,
            format_strings: false,
            log_messages: false,
            custom_patterns: vec![],
        };

        let filter_config = FilterConfig {
            exclude_keywords: vec!["TODO".to_string()],
            ..Default::default()
        };
        let filter = Arc::new(ContentFilter::new(filter_config).expect("Failed to create filter"));

        let parser_config = ParserConfig::default();
        let coordinator =
            ParserCoordinator::new(parser_config, strategy_config, filter).expect("Failed to create coordinator");

        let content = r#"
/// This is a doc comment
fn main() {
    // This is a normal comment
    // TODO: fix this later
}
"#;

        let file = create_test_file(content, "test.rs");
        let units = coordinator.parse_file(&file).expect("Parsing failed");

        let texts: Vec<_> = units.iter().map(|u| u.content.as_str()).collect();
        
        assert!(texts.iter().any(|t| t.contains("doc comment")));
        assert!(texts.iter().any(|t| t.contains("normal comment")));
        assert!(!texts.iter().any(|t| t.contains("TODO")));
    }

    #[test]
    fn test_full_exclusion_pipeline() {
        let strategy_config = ExtractionConfig {
            comments: true,
            docstrings: false,
            error_messages: false,
            format_strings: false,
            log_messages: false,
            custom_patterns: vec![],
        };

        let filter_config = FilterConfig {
            exclude_keywords: vec!["NOTE".to_string(), "XXX".to_string()],
            exclude_patterns: vec![r"https?://\S+".to_string()],
            min_length: 5,
            max_length: 1000,
            ..Default::default()
        };
        let filter = Arc::new(ContentFilter::new(filter_config).expect("Failed to create filter"));

        let parser_config = ParserConfig::default();
        let coordinator =
            ParserCoordinator::new(parser_config, strategy_config, filter).expect("Failed to create coordinator");

        let content = r#"
fn main() {
    // This should be extracted
    // NOTE: this should be filtered
    // Visit https://example.com
    // Hi
}
"#;

        let file = create_test_file(content, "test.rs");
        let units = coordinator.parse_file(&file).expect("Parsing failed");

        let texts: Vec<_> = units.iter().map(|u| u.content.as_str()).collect();
        
        assert!(texts.iter().any(|t| t.contains("should be extracted")));
        assert!(!texts.iter().any(|t| t.contains("NOTE")));
        assert!(!texts.iter().any(|t| t.contains("https://")));
        assert!(!texts.iter().any(|t| t == "Hi"));
    }

    #[test]
    fn test_markdown_strategy_filtering() {
        let strategy_config = ExtractionConfig {
            comments: true,
            docstrings: false,
            error_messages: false,
            format_strings: false,
            log_messages: false,
            custom_patterns: vec![],
        };

        let filter = Arc::new(ContentFilter::default());

        let parser_config = ParserConfig::default();
        let coordinator =
            ParserCoordinator::new(parser_config, strategy_config, filter).expect("Failed to create coordinator");

        let content = r#"# Header

This is a paragraph.

// This looks like a comment

TODO: Remember to update this
"#;

        let file = create_test_file(content, "test.md");
        let units = coordinator.parse_file(&file).expect("Parsing failed");

        let texts: Vec<_> = units.iter().map(|u| u.content.as_str()).collect();
        
        assert!(!texts.iter().any(|t| t.contains("TODO")));
    }
}

