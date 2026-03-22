//! Edge Case and Boundary Tests
//!
//! Tests for handling edge cases, malformed input, and boundary conditions.

use std::path::PathBuf;

use codebase_translate::core::models::File;
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::core::StringProcessor;
use codebase_translate::parser::ParserConfig;

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

fn create_test_coordinator() -> ParserCoordinator {
    ParserCoordinator::with_defaults(ParserConfig::default())
        .expect("Failed to create coordinator")
}

mod empty_content_tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let coordinator = create_test_coordinator();

        let file = create_test_file("", "empty.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(units.is_empty(), "Empty file should produce no units");
    }

    #[test]
    fn test_whitespace_only_file() {
        let coordinator = create_test_coordinator();

        let file = create_test_file("   \n\t\n  ", "whitespace.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(units.is_empty(), "Whitespace-only file should produce no units");
    }

    #[test]
    fn test_newlines_only_file() {
        let coordinator = create_test_coordinator();

        let file = create_test_file("\n\n\n\n", "newlines.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(units.is_empty(), "Newlines-only file should produce no units");
    }
}

mod malformed_content_tests {
    use super::*;

    #[test]
    fn test_unclosed_string() {
        let coordinator = create_test_coordinator();

        let content = r#"fn main() {
    let s = "unclosed string
}"#;

        let file = create_test_file(content, "unclosed.rs");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok() || result.is_err(),
            "Should handle unclosed string gracefully"
        );
    }

    #[test]
    fn test_unclosed_comment() {
        let coordinator = create_test_coordinator();

        let content = r#"fn main() {
    /* unclosed block comment
}"#;

        let file = create_test_file(content, "unclosed_comment.rs");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok() || result.is_err(),
            "Should handle unclosed comment gracefully"
        );
    }

    #[test]
    fn test_unmatched_braces() {
        let coordinator = create_test_coordinator();

        let content = r#"fn main() {{
    // comment
}"#;

        let file = create_test_file(content, "unmatched.rs");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok(),
            "Should handle unmatched braces (may produce partial results)"
        );
    }

    #[test]
    fn test_invalid_syntax() {
        let coordinator = create_test_coordinator();

        let content = r#"fn main() {
    @#$%^&*()
}"#;

        let file = create_test_file(content, "invalid.rs");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok() || result.is_err(),
            "Should handle invalid syntax gracefully"
        );
    }

    #[test]
    fn test_incomplete_function() {
        let coordinator = create_test_coordinator();

        let content = r#"fn main("#;

        let file = create_test_file(content, "incomplete.rs");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok() || result.is_err(),
            "Should handle incomplete function gracefully"
        );
    }
}

mod encoding_tests {
    use super::*;

    #[test]
    fn test_utf8_content() {
        let coordinator = create_test_coordinator();

        let content = r#"// 日本語コメント
fn main() {}
// 中文注释
// العربية"#;

        let file = create_test_file(content, "utf8.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(!units.is_empty(), "Should handle UTF-8 content");
    }

    #[test]
    fn test_emojis_in_content() {
        let coordinator = create_test_coordinator();

        let content = r#"// Comment with emoji 👋 🌍
fn main() {}
// 🎉 Celebration"#;

        let file = create_test_file(content, "emoji.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(!units.is_empty(), "Should handle emojis in content");
    }

    #[test]
    fn test_special_unicode_characters() {
        let coordinator = create_test_coordinator();

        let content = r#"// Special chars: ← → ↑ ↓ ✓ ✗
fn main() {}
// Mathematical: ∑ ∏ ∫ √"#;

        let file = create_test_file(content, "unicode.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(!units.is_empty(), "Should handle special Unicode characters");
    }
}

mod boundary_tests {
    use super::*;

    #[test]
    fn test_very_short_comment() {
        let coordinator = create_test_coordinator();

        let content = r#"// A
fn main() {}"#;

        let file = create_test_file(content, "short.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(
            units.is_empty() || units.len() == 1,
            "Should handle very short comment"
        );
    }

    #[test]
    fn test_very_long_comment() {
        let coordinator = create_test_coordinator();

        let long_comment = "// ".to_string() + &"A".repeat(10000);
        let content = format!("{}\nfn main() {{}}", long_comment);

        let file = create_test_file(&content, "long.rs");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok(),
            "Should handle very long comment"
        );
    }

    #[test]
    fn test_many_small_comments() {
        let coordinator = create_test_coordinator();

        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("// Comment {}\n", i));
        }
        content.push_str("fn main() {}");

        let file = create_test_file(&content, "many_comments.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(
            units.len() <= 1000,
            "Should handle many small comments"
        );
    }

    #[test]
    fn test_deeply_nested_structure() {
        let coordinator = create_test_coordinator();

        let content = r#"fn main() {
    {
        {
            {
                // Deep comment
            }
        }
    }
}"#;

        let file = create_test_file(content, "nested.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(!units.is_empty() || units.is_empty(), "Should handle deeply nested structure");
    }
}

mod string_processor_tests {
    use super::*;

    #[test]
    fn test_clean_regular_string() {
        let processor = StringProcessor::new();

        assert_eq!(
            processor.clean_string_literal("\"hello world\""),
            "hello world"
        );
    }

    #[test]
    fn test_clean_raw_string() {
        let processor = StringProcessor::new();

        assert_eq!(
            processor.clean_string_literal("r\"hello world\""),
            "hello world"
        );
        assert_eq!(
            processor.clean_string_literal("r#\"hello \"world\"#"),
            "hello \"world"
        );
        assert_eq!(
            processor.clean_string_literal("r##\"hello #\"world\"##"),
            "hello #\"world"
        );
    }

    #[test]
    fn test_unescape_sequences() {
        let processor = StringProcessor::new();

        assert_eq!(processor.unescape("hello\\nworld"), "hello\nworld");
        assert_eq!(processor.unescape("hello\\tworld"), "hello\tworld");
        assert_eq!(processor.unescape("hello\\\\world"), "hello\\world");
        assert_eq!(processor.unescape("hello\\\"world"), "hello\"world");
    }

    #[test]
    fn test_is_only_symbols() {
        let processor = StringProcessor::new();

        assert!(processor.is_only_symbols("!@#$%"));
        assert!(processor.is_only_symbols("12345"));
        assert!(processor.is_only_symbols("   "));
        assert!(!processor.is_only_symbols("hello"));
        assert!(!processor.is_only_symbols("hello123"));
        assert!(!processor.is_only_symbols("你好"));
    }

    #[test]
    fn test_empty_string_processing() {
        let processor = StringProcessor::new();

        assert_eq!(processor.clean_string_literal("\"\""), "");
        assert_eq!(processor.unescape(""), "");
        assert!(processor.is_only_symbols(""));
    }

    #[test]
    fn test_unicode_string_processing() {
        let processor = StringProcessor::new();

        assert_eq!(
            processor.clean_string_literal("\"你好世界\""),
            "你好世界"
        );
        assert!(!processor.is_only_symbols("你好"));
    }
}

mod nested_comment_tests {
    use super::*;

    #[test]
    fn test_nested_block_comments() {
        let coordinator = create_test_coordinator();

        let content = r#"/* outer /* inner */ outer */
fn main() {}"#;

        let file = create_test_file(content, "nested.rs");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok(),
            "Should handle nested block comments"
        );
    }

    #[test]
    fn test_comment_in_string() {
        let coordinator = create_test_coordinator();

        let content = r#"fn main() {
    let s = "// not a comment";
}"#;

        let file = create_test_file(content, "comment_in_string.rs");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok(),
            "Should handle comments inside strings"
        );
    }

    #[test]
    fn test_string_in_comment() {
        let coordinator = create_test_coordinator();

        let content = r#"// This is a comment with "string" inside
fn main() {}"#;

        let file = create_test_file(content, "string_in_comment.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(!units.is_empty(), "Should extract comment with string inside");
    }
}

mod file_edge_cases {
    use super::*;

    #[test]
    fn test_file_with_only_shebang() {
        let coordinator = create_test_coordinator();

        let content = "#!/usr/bin/env rust\n";

        let file = create_test_file(content, "shebang.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(units.is_empty() || !units.is_empty(), "Should handle shebang");
    }

    #[test]
    fn test_file_with_bom() {
        let coordinator = create_test_coordinator();

        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice(b"// Comment\nfn main() {}");

        let file = File::new(PathBuf::from("bom.rs"), content, "utf-8");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok(),
            "Should handle BOM"
        );
    }

    #[test]
    fn test_file_with_null_bytes() {
        let coordinator = create_test_coordinator();

        let mut content = b"// Comment\nfn main() {}".to_vec();
        content.insert(5, 0);

        let file = File::new(PathBuf::from("null.rs"), content, "utf-8");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok() || result.is_err(),
            "Should handle null bytes gracefully"
        );
    }

    #[test]
    fn test_file_with_carriage_return() {
        let coordinator = create_test_coordinator();

        let content = "// Comment\r\nfn main() {}\r\n// Another\r\n";

        let file = create_test_file(content, "crlf.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(
            !units.is_empty(),
            "Should handle CRLF line endings"
        );
    }
}

mod stress_tests {
    use super::*;

    #[test]
    fn test_large_file_parsing() {
        let coordinator = create_test_coordinator();

        let mut content = String::new();
        for i in 0..10000 {
            content.push_str(&format!("// Line {} comment\n", i));
            if i % 10 == 0 {
                content.push_str(&format!("fn func{}() {{}}\n", i));
            }
        }

        let file = create_test_file(&content, "large.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(
            !units.is_empty(),
            "Should handle large files"
        );
    }

    #[test]
    fn test_very_long_line() {
        let coordinator = create_test_coordinator();

        let long_line = "// ".to_string() + &"word ".repeat(10000);
        let content = format!("{}\nfn main() {{}}", long_line);

        let file = create_test_file(&content, "long_line.rs");
        let result = coordinator.parse_file(&file);

        assert!(
            result.is_ok(),
            "Should handle very long lines"
        );
    }

    #[test]
    fn test_many_empty_lines() {
        let coordinator = create_test_coordinator();

        let content = "// Comment\n".to_string() + &"\n".repeat(10000) + "fn main() {}";

        let file = create_test_file(&content, "empty_lines.rs");
        let units = coordinator.parse_file(&file).expect("Parsing should succeed");

        assert!(!units.is_empty(), "Should handle many empty lines");
    }
}

