//! Placeholder Boundary Preservation Tests
//!
//! These tests verify that the scanner correctly extracts complete placeholders
//! (including ${ and } boundaries) and that the protection mechanism preserves
//! them during translation.

use codebase_translate::parser::scanner::{
    PlaceholderProtector, ScannerConfig, ScannerLanguageConfig, TextRegionType, TextScanner,
};

/// Test that the scanner extracts complete placeholders including boundaries
#[test]
fn test_scanner_extracts_complete_placeholder() {
    let content = "`Hello ${name}!`";
    let config = ScannerConfig::new(vec![]).with_strings(true);
    let language = ScannerLanguageConfig::from_extension("js").unwrap();
    let scanner = TextScanner::new(language, config);

    let regions = scanner.scan(content);

    assert_eq!(regions.len(), 1, "Should find one template string");

    let region = &regions[0];
    assert_eq!(region.region_type, TextRegionType::TemplateString);
    assert_eq!(region.placeholders.len(), 1, "Should find one placeholder");

    let placeholder = &region.placeholders[0];
    assert_eq!(
        placeholder.original, "${name}",
        "Placeholder should include ${{ and }}"
    );
    assert!(
        placeholder.original.starts_with("${"),
        "Placeholder should start with ${{"
    );
    assert!(
        placeholder.original.ends_with('}'),
        "Placeholder should end with }}"
    );
}

#[test]
fn test_scanner_extracts_multiple_placeholders() {
    let content = "`${first} and ${second}`";
    let config = ScannerConfig::new(vec![]).with_strings(true);
    let language = ScannerLanguageConfig::from_extension("js").unwrap();
    let scanner = TextScanner::new(language, config);

    let regions = scanner.scan(content);

    assert_eq!(regions.len(), 1, "Should find one template string");

    let region = &regions[0];
    assert_eq!(region.placeholders.len(), 2, "Should find two placeholders");

    let placeholder1 = &region.placeholders[0];
    assert_eq!(
        placeholder1.original, "${first}",
        "First placeholder should include boundaries"
    );

    let placeholder2 = &region.placeholders[1];
    assert_eq!(
        placeholder2.original, "${second}",
        "Second placeholder should include boundaries"
    );
}

#[test]
fn test_scanner_placeholder_positions() {
    let content = "`Hello ${name}!`";
    let config = ScannerConfig::new(vec![]).with_strings(true);
    let language = ScannerLanguageConfig::from_extension("js").unwrap();
    let scanner = TextScanner::new(language, config);

    let regions = scanner.scan(content);
    let region = &regions[0];
    let placeholder = &region.placeholders[0];

    // Content: "Hello ${name}!"
    //          012345678901234
    // ${name} = $(6){(7)n(8)a(9)m(10)e(11)}(12)
    // starts at 6, ends at 13 (exclusive)
    assert_eq!(
        placeholder.start, 6,
        "Placeholder start should be relative to content start"
    );
    assert_eq!(
        placeholder.end, 13,
        "Placeholder end should be relative to content start"
    );
    assert_eq!(placeholder.len(), 7, "Placeholder length should be correct");
}

#[test]
fn test_protection_replaces_complete_placeholder() {
    let content = "`String ${value} here`";
    let config = ScannerConfig::new(vec![]).with_strings(true);
    let language = ScannerLanguageConfig::from_extension("js").unwrap();
    let scanner = TextScanner::new(language, config);

    let regions = scanner.scan(content);
    let region = &regions[0];

    let protector = PlaceholderProtector::new();
    let prepared = protector.prepare_for_translation(region, content);

    assert_eq!(
        prepared, "String __PH_0__ here",
        "Complete placeholder should be replaced"
    );
    assert!(
        !prepared.contains("${"),
        "Placeholder start boundary should be removed"
    );
    assert!(
        !prepared.contains('}'),
        "Placeholder end boundary should be removed"
    );
}

#[test]
fn test_restoration_preserves_boundaries() {
    let content = "`String ${value} here`";
    let config = ScannerConfig::new(vec![]).with_strings(true);
    let language = ScannerLanguageConfig::from_extension("js").unwrap();
    let scanner = TextScanner::new(language, config);

    let regions = scanner.scan(content);
    let region = &regions[0];

    let protector = PlaceholderProtector::new();
    let _prepared = protector.prepare_for_translation(region, content);

    // Simulate translation
    let translated = "字符串 __PH_0__ 这里";

    // Restore placeholders
    let restored = protector.restore_placeholders(translated, region);

    assert_eq!(restored, "字符串 ${value} 这里");
    assert!(
        restored.contains("${value}"),
        "Complete placeholder should be restored"
    );
    assert!(
        restored.contains("${"),
        "Placeholder start boundary should be preserved"
    );
    assert!(
        restored.contains('}'),
        "Placeholder end boundary should be preserved"
    );
}

#[test]
fn test_end_to_end_placeholder_protection() {
    let content = "`Error: ${error}, code: ${code}`";
    let config = ScannerConfig::new(vec![]).with_strings(true);
    let language = ScannerLanguageConfig::from_extension("js").unwrap();
    let scanner = TextScanner::new(language, config);

    let regions = scanner.scan(content);
    let region = &regions[0];

    // Verify scanner extracted placeholders correctly
    assert_eq!(region.placeholders.len(), 2);
    assert_eq!(region.placeholders[0].original, "${error}");
    assert_eq!(region.placeholders[1].original, "${code}");

    // Protect for translation
    let protector = PlaceholderProtector::new();
    let prepared = protector.prepare_for_translation(region, content);

    assert_eq!(prepared, "Error: __PH_0__, code: __PH_1__");
    assert!(!prepared.contains("${error}"));
    assert!(!prepared.contains("${code}"));

    // Simulate LLM translation (might break boundaries)
    let translated = "错误：__PH_0__，代码：__PH_1__";

    // Restore placeholders
    let restored = protector.restore_placeholders(translated, region);

    assert_eq!(restored, "错误：${error}，代码：${code}");
    assert!(restored.contains("${error}"));
    assert!(restored.contains("${code}"));
}

#[test]
fn test_nested_braces_in_placeholder() {
    let content = "`Value: ${obj.property}`";
    let config = ScannerConfig::new(vec![]).with_strings(true);
    let language = ScannerLanguageConfig::from_extension("js").unwrap();
    let scanner = TextScanner::new(language, config);

    let regions = scanner.scan(content);
    let region = &regions[0];

    assert_eq!(region.placeholders.len(), 1);
    let placeholder = &region.placeholders[0];
    assert_eq!(
        placeholder.original, "${obj.property}",
        "Should extract complete placeholder with nested property"
    );
}

#[test]
fn test_placeholder_with_expression() {
    let content = "`Sum: ${a + b}`";
    let config = ScannerConfig::new(vec![]).with_strings(true);
    let language = ScannerLanguageConfig::from_extension("js").unwrap();
    let scanner = TextScanner::new(language, config);

    let regions = scanner.scan(content);
    let region = &regions[0];

    assert_eq!(region.placeholders.len(), 1);
    let placeholder = &region.placeholders[0];
    assert_eq!(
        placeholder.original, "${a + b}",
        "Should extract placeholder with expression"
    );
}

#[test]
fn test_protection_with_complex_expression() {
    let content = "`Result: ${items.filter(x => x > 0).length}`";
    let config = ScannerConfig::new(vec![]).with_strings(true);
    let language = ScannerLanguageConfig::from_extension("js").unwrap();
    let scanner = TextScanner::new(language, config);

    let regions = scanner.scan(content);
    let region = &regions[0];

    assert_eq!(region.placeholders.len(), 1);
    let placeholder = &region.placeholders[0];
    assert_eq!(
        placeholder.original, "${items.filter(x => x > 0).length}",
        "Should extract complex placeholder expression"
    );

    let protector = PlaceholderProtector::new();
    let prepared = protector.prepare_for_translation(region, content);

    assert!(prepared.contains("__PH_0__"));
    assert!(!prepared.contains("${"));

    let translated = "结果：__PH_0__";
    let restored = protector.restore_placeholders(translated, region);

    assert_eq!(restored, "结果：${items.filter(x => x > 0).length}");
}

#[test]
fn test_multiple_placeholders_same_line() {
    let content = "`${x} + ${y} = ${result}`";
    let config = ScannerConfig::new(vec![]).with_strings(true);
    let language = ScannerLanguageConfig::from_extension("js").unwrap();
    let scanner = TextScanner::new(language, config);

    let regions = scanner.scan(content);
    let region = &regions[0];

    assert_eq!(region.placeholders.len(), 3);
    assert_eq!(region.placeholders[0].original, "${x}");
    assert_eq!(region.placeholders[1].original, "${y}");
    assert_eq!(region.placeholders[2].original, "${result}");

    let protector = PlaceholderProtector::new();
    let prepared = protector.prepare_for_translation(region, content);

    assert_eq!(prepared, "__PH_0__ + __PH_1__ = __PH_2__");

    let translated = "__PH_0__ 加 __PH_1__ 等于 __PH_2__";
    let restored = protector.restore_placeholders(translated, region);

    assert_eq!(restored, "${x} 加 ${y} 等于 ${result}");
}
