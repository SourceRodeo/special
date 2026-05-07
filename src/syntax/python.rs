/**
@module SPECIAL.SYNTAX.PYTHON
Builds shared item and call graphs for Python source files from tree-sitter syntax trees for parser-backed Python traceability.
*/
// @fileimplements SPECIAL.SYNTAX.PYTHON
use std::path::{Component, Path};

use tree_sitter::{Node, Parser};

use super::{
    CallSyntaxKind, ParsedSourceGraph, SourceItem, SourceItemKind, SourceLanguage, SourceSpan,
    SyntaxProvider, build_qualified_name, collect_calls_with, structural_shape,
};

pub(crate) struct PythonSyntaxProvider;

impl SyntaxProvider for PythonSyntaxProvider {
    fn parse(&self, path: &Path, text: &str) -> Option<ParsedSourceGraph> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .ok()?;
        let tree = parser.parse(text, None)?;
        if tree.root_node().has_error() {
            return None;
        }
        let mut items = Vec::new();
        collect_items(path, tree.root_node(), text.as_bytes(), &mut items);
        Some(ParsedSourceGraph {
            language: SourceLanguage::new("python"),
            items,
        })
    }
}

pub(crate) fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    PythonSyntaxProvider.parse(path, text)
}

fn collect_items(path: &Path, node: Node<'_>, source: &[u8], items: &mut Vec<SourceItem>) {
    if node.kind() == "function_definition"
        && let Some(item) = parse_function_definition(path, node, source)
    {
        items.push(item);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_items(path, child, source, items);
    }
}

fn parse_function_definition(path: &Path, node: Node<'_>, source: &[u8]) -> Option<SourceItem> {
    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .to_string();
    let body = node.child_by_field_name("body")?;
    let container_path = class_container_segments(node, source);
    let kind = if container_path.is_empty() {
        SourceItemKind::Function
    } else {
        SourceItemKind::Method
    };
    Some(build_item(
        path,
        node,
        source,
        name,
        body,
        container_path,
        kind,
    ))
}

fn build_item(
    path: &Path,
    node: Node<'_>,
    source: &[u8],
    name: String,
    body: Node<'_>,
    container_path: Vec<String>,
    kind: SourceItemKind,
) -> SourceItem {
    let module_path = file_module_segments(path);
    let qualified_name = build_qualified_name(&module_path, &container_path, &name);
    let span = SourceSpan::from_tree_sitter(node);
    let (shape_fingerprint, shape_node_count) = structural_shape(node);
    let test_file = is_python_test_file(path);
    SourceItem {
        source_path: path.display().to_string(),
        stable_id: format!("{}:{}:{}", path.display(), qualified_name, span.start_line),
        public: is_public_name(&name),
        root_visible: is_public_name(&name),
        is_test: test_file && name.starts_with("test_"),
        name,
        qualified_name,
        module_path,
        container_path,
        shape_fingerprint,
        shape_node_count,
        kind,
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
        "attribute" => {
            let attribute = node
                .child_by_field_name("attribute")
                .or_else(|| last_named_child(node))?;
            let object = node
                .child_by_field_name("object")
                .or_else(|| first_named_child(node))?;
            Some((
                attribute.utf8_text(source).ok()?.to_string(),
                Some(object.utf8_text(source).ok()?.to_string()),
                CallSyntaxKind::Field,
            ))
        }
        _ => None,
    }
}

fn class_container_segments(node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "class_definition"
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

fn file_module_segments(path: &Path) -> Vec<String> {
    let mut normalized = path.components().collect::<Vec<_>>();
    if let Some(index) = normalized
        .iter()
        .position(|component| component.as_os_str() == "src")
    {
        normalized.drain(..=index);
    }

    let mut segments = normalized
        .iter()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return segments;
    }

    let file_name = segments.pop().unwrap_or_default();
    let stem = file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or(file_name);
    if stem != "__init__" {
        segments.push(stem);
    }
    segments
}

fn is_public_name(name: &str) -> bool {
    !name.starts_with('_')
}

pub(crate) fn is_python_test_file(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "tests")
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("test_") || name.ends_with("_test.py"))
}

fn first_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).next()
}

fn last_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).last()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::parse_source_graph;
    use crate::syntax::{CallSyntaxKind, SourceItemKind};

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.PYTHON_ITEMS_AND_CALLS
    fn provider_facade_collects_python_items_calls_methods_and_pytest_roots() {
        let graph = parse_source_graph(
            Path::new("src/test_app.py"),
            r#"
from app import live_impl

def test_live_impl():
    return live_impl()

def helper():
    return Service().run()

class Service:
    def run(self):
        return helper()
"#,
        )
        .expect("python graph should parse");
        let items = graph
            .items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();
        assert!(items.contains(&"test_live_impl"));
        assert!(items.contains(&"helper"));
        assert!(items.contains(&"run"));

        let test_item = graph
            .items
            .iter()
            .find(|item| item.name == "test_live_impl")
            .expect("pytest root should be present");
        assert!(test_item.is_test);
        assert!(test_item.calls.iter().any(|call| {
            call.name == "live_impl"
                && call.qualifier.is_none()
                && call.syntax == CallSyntaxKind::Identifier
        }));

        let method = graph
            .items
            .iter()
            .find(|item| item.name == "run")
            .expect("method should be present");
        assert_eq!(method.kind, SourceItemKind::Method);
        assert_eq!(method.qualified_name, "test_app::Service::run");
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.PYTHON_ITEMS_AND_CALLS
    fn provider_facade_rejects_python_syntax_error_trees() {
        let graph = parse_source_graph(Path::new("src/app.py"), "def broken(:\n    pass\n");

        assert!(
            graph.is_none(),
            "syntax-error Python trees should not produce partial source graphs"
        );
    }
}
