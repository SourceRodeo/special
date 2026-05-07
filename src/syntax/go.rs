/**
@module SPECIAL.SYNTAX.GO
Builds shared item and call graphs for Go source files from tree-sitter syntax trees so higher-level analysis can consume normalized structure instead of raw parser nodes.
*/
// @fileimplements SPECIAL.SYNTAX.GO
use std::path::Path;

use tree_sitter::{Node, Parser};

use super::{
    CallSyntaxKind, ParsedSourceGraph, SourceItem, SourceItemKind, SourceLanguage, SourceSpan,
    SyntaxProvider, build_qualified_name, collect_calls_with, first_named_child, last_named_child,
    normalized_shape_fingerprints, structural_shape,
};

pub(crate) struct GoSyntaxProvider;

impl SyntaxProvider for GoSyntaxProvider {
    fn parse(&self, path: &Path, text: &str) -> Option<ParsedSourceGraph> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_go::LANGUAGE.into()).ok()?;
        let tree = parser.parse(text, None)?;
        if tree.root_node().has_error() {
            return None;
        }
        let mut items = Vec::new();
        collect_items(path, tree.root_node(), text.as_bytes(), &mut items);
        Some(ParsedSourceGraph {
            language: SourceLanguage::new("go"),
            items,
        })
    }
}

pub(crate) fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    GoSyntaxProvider.parse(path, text)
}

fn collect_items(path: &Path, node: Node<'_>, source: &[u8], items: &mut Vec<SourceItem>) {
    match node.kind() {
        "function_declaration" => {
            if let Some(item) = parse_function_declaration(path, node, source) {
                items.push(item);
            }
        }
        "method_declaration" => {
            if let Some(item) = parse_method_declaration(path, node, source) {
                items.push(item);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_items(path, child, source, items);
    }
}

fn parse_function_declaration(path: &Path, node: Node<'_>, source: &[u8]) -> Option<SourceItem> {
    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .to_string();
    let body = node.child_by_field_name("body")?;
    Some(build_item(
        path,
        node,
        source,
        name,
        body,
        GoItemMeta {
            container_path: Vec::new(),
            kind: SourceItemKind::Function,
        },
    ))
}

fn parse_method_declaration(path: &Path, node: Node<'_>, source: &[u8]) -> Option<SourceItem> {
    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .to_string();
    let body = node.child_by_field_name("body")?;
    let receiver = node.child_by_field_name("receiver")?;
    let receiver_type = receiver_type_name(receiver, source)?;
    Some(build_item(
        path,
        node,
        source,
        name,
        body,
        GoItemMeta {
            container_path: vec![receiver_type],
            kind: SourceItemKind::Method,
        },
    ))
}

struct GoItemMeta {
    container_path: Vec<String>,
    kind: SourceItemKind,
}

fn build_item(
    path: &Path,
    node: Node<'_>,
    source: &[u8],
    name: String,
    body: Node<'_>,
    meta: GoItemMeta,
) -> SourceItem {
    let module_path = file_module_segments(path);
    let qualified_name = build_qualified_name(&module_path, &meta.container_path, &name);
    let span = SourceSpan::from_tree_sitter(node);
    let (shape_fingerprint, shape_node_count) = structural_shape(node);
    SourceItem {
        source_path: path.display().to_string(),
        stable_id: format!("{}:{}:{}", path.display(), qualified_name, span.start_line),
        public: is_exported_name(&name),
        root_visible: is_exported_name(&name),
        is_test: is_go_test_item(path, &name),
        name,
        qualified_name,
        module_path,
        container_path: meta.container_path,
        shape_fingerprint,
        normalized_fingerprints: normalized_shape_fingerprints(node, source),
        shape_node_count,
        kind: meta.kind,
        span,
        calls: collect_calls_with(body, source, function_name),
        invocations: Vec::new(),
    }
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
        "selector_expression" => {
            let field = node
                .child_by_field_name("field")
                .or_else(|| last_named_child(node))?;
            let operand = node
                .child_by_field_name("operand")
                .or_else(|| first_named_child(node))?;
            Some((
                field.utf8_text(source).ok()?.to_string(),
                Some(operand.utf8_text(source).ok()?.to_string()),
                CallSyntaxKind::Field,
            ))
        }
        "parenthesized_expression" => {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .next()
                .and_then(|child| function_name(child, source))
        }
        _ => None,
    }
}

fn receiver_type_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    match node.kind() {
        "type_identifier" => Some(node.utf8_text(source).ok()?.to_string()),
        "generic_type" => node
            .child_by_field_name("type")
            .and_then(|child| receiver_type_name(child, source))
            .or_else(|| {
                let mut cursor = node.walk();
                node.named_children(&mut cursor)
                    .find_map(|child| receiver_type_name(child, source))
            }),
        "qualified_type" => node
            .child_by_field_name("name")
            .or_else(|| last_named_child(node))
            .and_then(|child| child.utf8_text(source).ok().map(ToString::to_string)),
        "pointer_type" | "parameter_list" | "parameter_declaration" => {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find_map(|child| receiver_type_name(child, source))
        }
        _ => None,
    }
}

fn file_module_segments(path: &Path) -> Vec<String> {
    let mut segments = path
        .iter()
        .map(|part| part.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return segments;
    }

    let file_name = segments.pop().unwrap_or_default();
    let stem = file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or(file_name);
    if stem != "main" || segments.is_empty() {
        segments.push(stem);
    }
    segments
}

fn is_exported_name(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}

fn is_go_test_item(path: &Path, name: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|file| file.ends_with("_test.go"))
        && name.starts_with("Test")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::parse_source_graph;
    use crate::syntax::CallSyntaxKind;

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.GO_ITEMS_AND_CALLS
    fn provider_facade_collects_go_items_and_calls() {
        let graph = parse_source_graph(
            Path::new("app/main.go"),
            r#"
package app

import "fmt"

func Entry() {
    helper()
    fmt.Println("hi")
}

func helper() {}
"#,
        )
        .expect("go graph should parse");

        assert_eq!(graph.items.len(), 2);
        let entry = graph
            .items
            .iter()
            .find(|item| item.name == "Entry")
            .expect("Entry should be present");
        assert!(entry.public);
        assert_eq!(entry.qualified_name, "app::Entry");
        assert!(entry.calls.iter().any(|call| {
            call.name == "helper"
                && call.qualifier.is_none()
                && call.syntax == CallSyntaxKind::Identifier
        }));
        assert!(entry.calls.iter().any(|call| {
            call.name == "Println"
                && call.qualifier.as_deref() == Some("fmt")
                && call.syntax == CallSyntaxKind::Field
        }));
    }
}
