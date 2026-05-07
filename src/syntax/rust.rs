/**
@module SPECIAL.SYNTAX.RUST
Builds shared item and call graphs for Rust source files from tree-sitter syntax trees so higher-level analysis can consume normalized structure instead of raw parser nodes.
*/
// @fileimplements SPECIAL.SYNTAX.RUST
use std::path::Path;

use tree_sitter::{Node, Parser};

use super::{
    CallSyntaxKind, ParsedSourceGraph, SourceInvocation, SourceInvocationKind, SourceItem,
    SourceItemKind, SourceLanguage, SourceSpan, SyntaxProvider, build_qualified_name,
    collect_calls_with, normalized_shape_fingerprints, structural_shape,
};

pub(crate) struct RustSyntaxProvider;

impl SyntaxProvider for RustSyntaxProvider {
    fn parse(&self, path: &Path, text: &str) -> Option<ParsedSourceGraph> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .ok()?;
        let tree = parser.parse(text, None)?;
        if tree.root_node().has_error() {
            return None;
        }
        let mut items = Vec::new();
        collect_items(path, tree.root_node(), text.as_bytes(), &mut items);
        Some(ParsedSourceGraph {
            language: SourceLanguage::new("rust"),
            items,
        })
    }
}

pub(crate) fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    RustSyntaxProvider.parse(path, text)
}

fn collect_items(path: &Path, node: Node<'_>, source: &[u8], items: &mut Vec<SourceItem>) {
    if node.kind() == "function_item"
        && let Some(item) = parse_function_item(path, node, source)
    {
        items.push(item);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_items(path, child, source, items);
    }
}

fn parse_function_item(path: &Path, node: Node<'_>, source: &[u8]) -> Option<SourceItem> {
    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .to_string();
    let body = node.child_by_field_name("body")?;
    let module_path = item_module_path(path, node, source);
    let container_path = impl_container_segments(node, source);
    let qualified_name = build_qualified_name(&module_path, &container_path, &name);
    let span = SourceSpan::from_tree_sitter(node);
    let (shape_fingerprint, shape_node_count) = structural_shape(node);
    Some(SourceItem {
        source_path: path.display().to_string(),
        stable_id: format!("{}:{}:{}", path.display(), qualified_name, span.start_line),
        name,
        qualified_name,
        module_path,
        container_path,
        shape_fingerprint,
        normalized_fingerprints: normalized_shape_fingerprints(node, source),
        shape_node_count,
        kind: item_kind(node),
        span,
        public: has_public_visibility(node, source),
        root_visible: has_root_visibility(node, source),
        is_test: has_test_attribute(node, source),
        calls: collect_calls_with(body, source, function_name),
        invocations: collect_invocations(body, source),
    })
}

fn item_kind(node: Node<'_>) -> SourceItemKind {
    let Some(parent) = node.parent() else {
        return SourceItemKind::Function;
    };
    let Some(grandparent) = parent.parent() else {
        return SourceItemKind::Function;
    };

    if parent.kind() == "declaration_list" && grandparent.kind() == "impl_item" {
        SourceItemKind::Method
    } else {
        SourceItemKind::Function
    }
}

fn has_test_attribute(node: Node<'_>, source: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(source) else {
        return false;
    };
    let lines = text.lines().collect::<Vec<_>>();
    let mut line_index = node.start_position().row;
    while line_index > 0 {
        line_index -= 1;
        let trimmed = lines
            .get(line_index)
            .map(|line| line.trim())
            .unwrap_or_default();
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("*/")
        {
            continue;
        }
        if trimmed.starts_with("#[") || trimmed.ends_with(']') {
            let (attribute_start, attribute) = collect_attribute_text(&lines, line_index);
            if attribute_marks_test(&attribute) {
                return true;
            }
            line_index = attribute_start;
            continue;
        }
        break;
    }
    false
}

fn collect_attribute_text(lines: &[&str], end_index: usize) -> (usize, String) {
    let mut start_index = end_index;
    while start_index > 0 {
        let trimmed = lines[start_index].trim();
        if trimmed.starts_with("#[") {
            break;
        }
        start_index -= 1;
    }
    (
        start_index,
        lines[start_index..=end_index].join("\n").trim().to_string(),
    )
}

fn attribute_marks_test(attribute_text: &str) -> bool {
    let Some(attribute) = attribute_text
        .strip_prefix("#[")
        .and_then(|text| text.strip_suffix(']'))
    else {
        return false;
    };
    let path = attribute
        .trim_start()
        .split(|ch: char| ch == '(' || ch.is_whitespace())
        .next()
        .unwrap_or_default()
        .trim();
    path == "test" || path.ends_with("::test")
}

fn has_public_visibility(node: Node<'_>, source: &[u8]) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| {
        child.kind() == "visibility_modifier"
            && child
                .utf8_text(source)
                .ok()
                .is_some_and(|text| text.trim() == "pub")
    })
}

fn has_root_visibility(node: Node<'_>, source: &[u8]) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| {
        child.kind() == "visibility_modifier"
            && child
                .utf8_text(source)
                .ok()
                .is_some_and(|text| text.trim().starts_with("pub"))
    })
}

fn collect_invocations(node: Node<'_>, source: &[u8]) -> Vec<SourceInvocation> {
    let mut invocations = Vec::new();
    collect_invocations_inner(node, source, &mut invocations);
    invocations
}

fn collect_invocations_inner(
    node: Node<'_>,
    source: &[u8],
    invocations: &mut Vec<SourceInvocation>,
) {
    if node.kind() == "call_expression"
        && let Some(binary_name) = local_cargo_binary_invocation(node, source)
    {
        invocations.push(SourceInvocation {
            kind: SourceInvocationKind::LocalCargoBinary { binary_name },
            span: SourceSpan::from_tree_sitter(node),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_invocations_inner(child, source, invocations);
    }
}

fn local_cargo_binary_invocation(node: Node<'_>, source: &[u8]) -> Option<String> {
    let function = node.child_by_field_name("function")?;
    let (name, qualifier, syntax) = function_name(function, source)?;
    let is_command_new = name == "new"
        && matches!(
            syntax,
            CallSyntaxKind::ScopedIdentifier | CallSyntaxKind::Field
        )
        && qualifier
            .as_deref()
            .is_some_and(|path| path == "Command" || path.ends_with("::Command"));
    if !is_command_new {
        return None;
    }

    let arguments = node.child_by_field_name("arguments")?;
    let mut cursor = arguments.walk();
    for child in arguments.named_children(&mut cursor) {
        if child.kind() != "macro_invocation" {
            continue;
        }
        let text = child.utf8_text(source).ok()?;
        if let Some(binary_name) = text
            .strip_prefix("env!(\"CARGO_BIN_EXE_")
            .and_then(|rest| rest.strip_suffix("\")"))
        {
            return Some(binary_name.to_string());
        }
    }
    None
}

fn function_name(
    node: Node<'_>,
    source: &[u8],
) -> Option<(String, Option<String>, CallSyntaxKind)> {
    match node.kind() {
        "identifier" => Some((
            node.utf8_text(source).ok()?.to_string(),
            None,
            CallSyntaxKind::Identifier,
        )),
        "scoped_identifier" => Some((
            node.child_by_field_name("name")?
                .utf8_text(source)
                .ok()?
                .to_string(),
            Some(
                node.child_by_field_name("path")?
                    .utf8_text(source)
                    .ok()?
                    .to_string(),
            ),
            CallSyntaxKind::ScopedIdentifier,
        )),
        "field_expression" => Some((
            node.child_by_field_name("field")?
                .utf8_text(source)
                .ok()?
                .to_string(),
            Some(
                node.child_by_field_name("value")?
                    .utf8_text(source)
                    .ok()?
                    .to_string(),
            ),
            CallSyntaxKind::Field,
        )),
        "generic_function" => function_name(node.child_by_field_name("function")?, source),
        "parenthesized_expression" => {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find_map(|child| function_name(child, source))
        }
        _ => None,
    }
}

fn item_module_path(path: &Path, node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut segments = file_module_segments(path);
    segments.extend(nested_mod_segments(node, source));
    segments
}

pub(crate) fn file_module_segments(path: &Path) -> Vec<String> {
    let mut normalized = path.components().collect::<Vec<_>>();
    if let Some(index) = normalized
        .iter()
        .position(|component| component.as_os_str() == "src")
    {
        normalized.drain(..=index);
    }

    let mut segments = normalized
        .iter()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return segments;
    }

    let file_name = segments.pop().unwrap_or_default();
    let stem = file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or(file_name);
    if stem != "lib" && stem != "main" && stem != "mod" {
        segments.push(stem);
    }
    segments
}

fn nested_mod_segments(node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "mod_item"
            && let Some(name) = parent
                .child_by_field_name("name")
                .and_then(|name| name.utf8_text(source).ok())
        {
            segments.push(name.to_string());
        }
        current = parent.parent();
    }
    segments.reverse();
    segments
}

fn impl_container_segments(node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "impl_item"
            && let Some(type_name) = parent
                .child_by_field_name("type")
                .and_then(|node| node.utf8_text(source).ok())
        {
            segments.push(type_name.to_string());
        }
        current = parent.parent();
    }
    segments.reverse();
    segments
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::parse_source_graph;
    use crate::syntax::{CallSyntaxKind, SourceInvocationKind, SourceItemKind};

    fn item_named<'a>(
        graph: &'a crate::syntax::ParsedSourceGraph,
        name: &str,
    ) -> &'a crate::syntax::SourceItem {
        graph
            .items
            .iter()
            .find(|item| item.name == name)
            .unwrap_or_else(|| panic!("item {name} should be present"))
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.RUST_ITEMS_AND_CALLS
    fn provider_facade_collects_rust_items_calls_tests_and_invocations() {
        let graph = parse_source_graph(
            Path::new("src/render/html.rs"),
            r#"
use std::process::Command;

mod nested {
    struct Worker;

    impl Worker {
        fn run() {}
    }
}

fn helper() {}

#[test]
fn verifies_demo() {
    helper();
    crate::shared::work();
    subject.run();
    Command::new(env!("CARGO_BIN_EXE_special")).output();
}

#[cfg(not(test))]
fn not_a_test() {
    format!("{}", env!("CARGO_BIN_EXE_special"));
}
"#,
        )
        .expect("rust graph should parse");

        let helper = item_named(&graph, "helper");
        assert_eq!(helper.qualified_name, "render::html::helper");
        assert!(!helper.public);
        assert!(!helper.is_test);

        let method = graph
            .items
            .iter()
            .find(|item| item.qualified_name == "render::html::nested::Worker::run")
            .expect("nested method should be present");
        assert_eq!(method.kind, SourceItemKind::Method);

        let test_item = item_named(&graph, "verifies_demo");
        assert!(test_item.is_test);
        assert!(test_item.calls.iter().any(|call| {
            call.name == "helper"
                && call.qualifier.is_none()
                && call.syntax == CallSyntaxKind::Identifier
        }));
        assert!(test_item.calls.iter().any(|call| {
            call.name == "work"
                && call.qualifier.as_deref() == Some("crate::shared")
                && call.syntax == CallSyntaxKind::ScopedIdentifier
        }));
        assert!(test_item.calls.iter().any(|call| {
            call.name == "run"
                && call.qualifier.as_deref() == Some("subject")
                && call.syntax == CallSyntaxKind::Field
        }));
        assert_eq!(
            test_item.invocations[0].kind,
            SourceInvocationKind::LocalCargoBinary {
                binary_name: "special".to_string()
            }
        );

        let not_a_test = item_named(&graph, "not_a_test");
        assert!(!not_a_test.is_test);
        assert!(not_a_test.invocations.is_empty());
    }

    #[test]
    fn provider_facade_collects_parenthesized_descriptor_field_calls() {
        let graph = parse_source_graph(
            Path::new("src/registry.rs"),
            r#"
fn parse_source_graph_at_path(descriptor: Descriptor, path: &Path, text: &str) {
    (descriptor.parse_source_graph)(path, text);
}
"#,
        )
        .expect("source graph should parse");
        let item = graph
            .items
            .iter()
            .find(|item| item.name == "parse_source_graph_at_path")
            .expect("registry function should be present");

        assert!(item.calls.iter().any(|call| {
            call.name == "parse_source_graph"
                && call.qualifier.as_deref() == Some("descriptor")
                && call.syntax == CallSyntaxKind::Field
        }));
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.RUST_ITEMS_AND_CALLS
    fn provider_facade_rejects_rust_syntax_error_trees() {
        let graph = parse_source_graph(Path::new("src/lib.rs"), "fn broken( {\n");

        assert!(
            graph.is_none(),
            "syntax-error Rust trees should not produce partial source graphs"
        );
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.RUST_TEST_DETECTION
    fn provider_facade_rejects_non_test_cfg_and_stringified_binary_names() {
        let graph = parse_source_graph(
            Path::new("src/lib.rs"),
            r#"
#[cfg(not(test))]
fn helper() {
    format!("{}", env!("CARGO_BIN_EXE_special"));
}
"#,
        )
        .expect("rust graph should parse");

        let helper = item_named(&graph, "helper");
        assert!(!helper.is_test);
        assert!(helper.invocations.is_empty());
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.RUST_TEST_DETECTION
    fn provider_facade_detects_multiline_test_attributes() {
        let graph = parse_source_graph(
            Path::new("src/lib.rs"),
            r#"
#[tokio::test(
    flavor = "multi_thread"
)]
async fn verifies_async_path() {
    helper().await;
}

async fn helper() {}
"#,
        )
        .expect("rust graph should parse");

        let test_item = item_named(&graph, "verifies_async_path");
        assert!(test_item.is_test);
        let helper = item_named(&graph, "helper");
        assert!(!helper.is_test);
    }
}
