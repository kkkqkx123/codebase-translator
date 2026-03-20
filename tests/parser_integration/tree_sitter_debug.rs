//! Debug test to check tree-sitter node text

use std::fs;
use std::path::PathBuf;
use tree_sitter::StreamingIterator;

const FIXTURES_DIR: &str = "tests/main_integration/fixtures";

#[test]
fn debug_tree_sitter_rust_multiply() {
    let fixture_path = PathBuf::from(FIXTURES_DIR).join("simple_rust.rs");
    let content = fs::read_to_string(&fixture_path).expect("Failed to read fixture");

    // Parse using tree-sitter directly
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Failed to set language");

    let tree = parser.parse(&content, None).expect("Failed to parse");
    let root_node = tree.root_node();

    // Query for line comments starting with ///
    let query_str = r#"((line_comment) @docstring
  (#match? @docstring "^///"))"#;

    let query = tree_sitter::Query::new(&tree_sitter_rust::LANGUAGE.into(), query_str)
        .expect("Failed to create query");

    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, content.as_bytes());

    println!("=== Tree-sitter Query Results ===");
    let mut i = 0;
    while let Some(m) = matches.next() {
        i += 1;
        for capture in m.captures {
            let node = capture.node;
            let text = node.utf8_text(content.as_bytes()).unwrap_or("ERROR");

            println!("\nMatch {}:", i);
            println!("  text: {:?}", text);
            println!("  text.len(): {}", text.len());
            println!("  text.bytes: {:?}", text.as_bytes());
            println!(
                "  start_position: row={}, column={}",
                node.start_position().row,
                node.start_position().column
            );
            println!(
                "  end_position: row={}, column={}",
                node.end_position().row,
                node.end_position().column
            );
            println!("  start_byte: {}", node.start_byte());
            println!("  end_byte: {}", node.end_byte());

            // Check if text contains newline
            if text.contains('\n') {
                println!("  *** WARNING: text contains newline! ***");
            }
        }
    }
}
