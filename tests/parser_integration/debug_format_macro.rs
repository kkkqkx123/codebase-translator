//! Debug test to check tree-sitter node text for format! macro

use tree_sitter::StreamingIterator;

#[test]
fn debug_format_macro_query_structure() {
    let content = r#"
fn main() {
    let name = "test_space";
    let message = format!("图空间 '{}' 不存在", name);
    println!("{}", message);

    let error = format!("配置文件 '{}' 未找到", "config.toml");
    eprintln!("{}", error);

    let info = format!("成功连接到数据库 '{}'", "postgres");
    log::info!("{}", info);

    // Test with multiple placeholders
    let result = format!("用户 '{}' 在 {} 时访问了资源", "admin", "2024-01-01");
    println!("{}", result);

    // Test format_args!
    let msg = format_args!("处理文件 '{}' 大小为 {} 字节", "data.txt", 1024);
    println!("{}", msg);

    // Test print! macros
    print!("正在处理: {}", name);
    println!("完成: {}", name);
    eprint!("错误: {}", name);
    eprintln!("警告: {}", name);
}
"#;

    // Parse using tree-sitter directly
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Failed to set language");

    let tree = parser.parse(&content, None).expect("Failed to parse");
    let root_node = tree.root_node();

    // Test 1: Simple macro_invocation query
    println!("=== Test 1: Simple macro_invocation query ===");
    let query_str = r#"(macro_invocation) @macro"#;
    let query = tree_sitter::Query::new(&tree_sitter_rust::LANGUAGE.into(), query_str)
        .expect("Failed to create query");
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, content.as_bytes());
    let mut count = 0;
    while matches.next().is_some() {
        count += 1;
    }
    println!("Found {} macro_invocation nodes\n", count);

    // Test 2: Current query from RustQueries::macro_strings()
    println!("=== Test 2: Current macro_strings() query ===");
    let query_str = r#"
(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (string_literal) @macro_string))

(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (raw_string_literal) @macro_string))
"#;
    let query = tree_sitter::Query::new(&tree_sitter_rust::LANGUAGE.into(), query_str)
        .expect("Failed to create query");
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, content.as_bytes());

    while let Some(m) = matches.next() {
        println!("Match:");
        for capture in m.captures {
            let node = capture.node;
            let text = node.utf8_text(content.as_bytes()).unwrap_or("ERROR");
            println!("  {}@{}: {:?}", capture.index, node.kind(), text);
        }
        println!();
    }

    // Test 3: Query to find all string_literals in token_tree
    println!("=== Test 3: Find all string_literals in token_tree ===");
    let query_str = r#"
(macro_invocation
  macro: (identifier) @macro_name
  (token_tree) @tree)
"#;
    let query = tree_sitter::Query::new(&tree_sitter_rust::LANGUAGE.into(), query_str)
        .expect("Failed to create query");
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, content.as_bytes());

    let mut i = 0;
    while let Some(m) = matches.next() {
        i += 1;
        println!("Macro {}:", i);
        for capture in m.captures {
            let node = capture.node;
            if node.kind() == "token_tree" {
                let text = node.utf8_text(content.as_bytes()).unwrap_or("ERROR");
                println!("  token_tree: {:?}", text);

                // Find all string_literal children
                let mut child_cursor = node.walk();
                for child in node.children(&mut child_cursor) {
                    let kind = child.kind();
                    if kind == "string_literal" || kind == "raw_string_literal" {
                        let child_text = child.utf8_text(content.as_bytes()).unwrap_or("ERROR");
                        println!("    Found {}@{:?}",
                            kind,
                            child_text
                        );
                    }
                }
            } else if node.kind() == "identifier" {
                let text = node.utf8_text(content.as_bytes()).unwrap_or("ERROR");
                println!("  macro name: {:?}", text);
            }
        }
        println!();
    }

    // Test 4: Alternative query using descendant
    println!("=== Test 4: Alternative query using descendant ===");
    let query_str = r#"
(macro_invocation
  macro: (identifier) @macro_name)
(macro_invocation
  (token_tree
    (string_literal) @macro_string))
"#;
    let query = tree_sitter::Query::new(&tree_sitter_rust::LANGUAGE.into(), query_str)
        .expect("Failed to create query");
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, content.as_bytes());

    while let Some(m) = matches.next() {
        println!("Match:");
        for capture in m.captures {
            let node = capture.node;
            let text = node.utf8_text(content.as_bytes()).unwrap_or("ERROR");
            println!("  {}@{}: {:?}", capture.index, node.kind(), text);
        }
        println!();
    }
}

#[test]
fn debug_simple_format_macro() {
    // Test with the exact example from the user
    let content = r#"
let message = format!("图空间 '{}' 不存在", name);
"#;

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Failed to set language");

    let tree = parser.parse(&content, None).expect("Failed to parse");
    let root_node = tree.root_node();

    println!("=== Simple format! macro analysis ===");
    println!("Content: {:?}", content);
    println!();

    // Print the full tree
    fn print_tree(node: &tree_sitter::Node, content: &str, indent: usize) {
        let indent_str = "  ".repeat(indent);
        let text = node.utf8_text(content.as_bytes()).unwrap_or("ERROR");
        let text_truncated = if text.chars().count() > 30 {
            let truncated: String = text.chars().take(27).collect();
            format!("{}...", truncated)
        } else {
            text.to_string()
        };
        println!("{}{} [{:?}]", indent_str, node.kind(), text_truncated);

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            print_tree(&child, content, indent + 1);
        }
    }

    print_tree(&root_node, &content, 0);

    println!("\n=== Query results ===");
    let query_str = r#"
(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (string_literal) @macro_string))
"#;
    let query = tree_sitter::Query::new(&tree_sitter_rust::LANGUAGE.into(), query_str)
        .expect("Failed to create query");
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, content.as_bytes());

    let mut count = 0;
    while matches.next().is_some() {
        count += 1;
    }
    println!("Found {} matches", count);

    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, content.as_bytes());

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let node = capture.node;
            let text = node.utf8_text(content.as_bytes()).unwrap_or("ERROR");
            println!("{}@{}: {:?}", capture.index, node.kind(), text);
        }
    }
}