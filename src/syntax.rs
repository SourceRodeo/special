/**
@module SPECIAL.SYNTAX
Normalizes parser-specific syntax trees into a shared per-file item and call graph that language packs can populate without leaking raw parser nodes into higher layers. This layer is the parser baseline for traceability, not a claim that syntax trees alone recover full semantic reachability, and it should discover compile-time syntax providers through the shared `SPECIAL.LANGUAGE_PACKS` registry instead of hardcoding one parse branch per language in the syntax core.

@group SPECIAL.SYNTAX
shared syntax contracts.

@group SPECIAL.SYNTAX.PROVIDERS
shared syntax provider behavior.

@spec SPECIAL.SYNTAX.PROVIDERS.GO_ITEMS_AND_CALLS
the Go syntax provider records top-level functions, visibility, qualified names, and identifier or selector call edges.

@spec SPECIAL.SYNTAX.PROVIDERS.PYTHON_ITEMS_AND_CALLS
the Python syntax provider records functions, methods, pytest test roots, module/container-qualified names, and identifier or attribute call edges for parser-backed Python traceability.

@spec SPECIAL.SYNTAX.PROVIDERS.RUST_ITEMS_AND_CALLS
the Rust syntax provider records functions, methods, module/container-qualified names, test roots, local binary invocations, and identifier/scoped/field call edges.

@spec SPECIAL.SYNTAX.PROVIDERS.RUST_TEST_DETECTION
the Rust syntax provider treats real tests and functions inside `#[cfg(test)]` modules as test roots without mistaking non-test cfg attributes or stringified binary names for tests or invocations.

@spec SPECIAL.SYNTAX.NORMALIZED_FINGERPRINTS
the shared syntax layer records normalized source-shape fingerprints for repeated literal-to-access mappings without replacing concrete structural fingerprints.

@spec SPECIAL.SYNTAX.PROVIDERS.TYPESCRIPT_ITEMS_AND_CALLS
the TypeScript syntax provider records exported/internal functions, exported arrow functions, and identifier or field call edges.

@spec SPECIAL.SYNTAX.PROVIDERS.TYPESCRIPT_TEST_CALLBACKS
the TypeScript syntax provider records supported test callback forms as test roots only in test files.
*/
// @fileimplements SPECIAL.SYNTAX
use std::path::Path;

use tree_sitter::Node;

pub(crate) mod go;
pub(crate) mod python;
mod registry;
pub(crate) mod rust;
pub(crate) mod typescript;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SourceLanguage(&'static str);

impl SourceLanguage {
    pub(crate) const fn new(id: &'static str) -> Self {
        Self(id)
    }

    pub(crate) const fn id(self) -> &'static str {
        self.0
    }

    pub(crate) fn from_path(path: &Path) -> Option<Self> {
        registry::language_for_path(path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SourceItemKind {
    Function,
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceSpan {
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
    pub(crate) start_column: usize,
    pub(crate) end_column: usize,
    pub(crate) start_byte: usize,
    pub(crate) end_byte: usize,
}

impl SourceSpan {
    fn from_tree_sitter(node: tree_sitter::Node<'_>) -> Self {
        Self {
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            start_column: node.start_position().column,
            end_column: node.end_position().column,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CallSyntaxKind {
    Identifier,
    ScopedIdentifier,
    Field,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceCall {
    pub(crate) name: String,
    pub(crate) qualifier: Option<String>,
    pub(crate) syntax: CallSyntaxKind,
    pub(crate) span: SourceSpan,
}

type ResolvedCallName = (String, Option<String>, CallSyntaxKind);
type CallResolver = for<'tree> fn(Node<'tree>, &[u8]) -> Option<ResolvedCallName>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SourceInvocationKind {
    LocalCargoBinary { binary_name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceInvocation {
    pub(crate) kind: SourceInvocationKind,
    pub(crate) span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceItem {
    pub(crate) source_path: String,
    pub(crate) stable_id: String,
    pub(crate) name: String,
    pub(crate) qualified_name: String,
    pub(crate) module_path: Vec<String>,
    pub(crate) container_path: Vec<String>,
    pub(crate) shape_fingerprint: String,
    pub(crate) normalized_fingerprints: Vec<String>,
    pub(crate) shape_node_count: usize,
    pub(crate) kind: SourceItemKind,
    pub(crate) span: SourceSpan,
    pub(crate) public: bool,
    pub(crate) root_visible: bool,
    pub(crate) is_test: bool,
    pub(crate) calls: Vec<SourceCall>,
    pub(crate) invocations: Vec<SourceInvocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedSourceGraph {
    pub(crate) language: SourceLanguage,
    pub(crate) items: Vec<SourceItem>,
}

trait SyntaxProvider {
    fn parse(&self, path: &Path, text: &str) -> Option<ParsedSourceGraph>;
}

pub(crate) fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    registry::parse_source_graph_at_path(path, text)
}

#[cfg(test)]
pub(crate) fn parse_source_graph_for_language(
    language: SourceLanguage,
    text: &str,
) -> Option<ParsedSourceGraph> {
    parse_source_graph_for_language_at_path(language, Path::new("lib.rs"), text)
}

#[cfg(test)]
fn parse_source_graph_for_language_at_path(
    language: SourceLanguage,
    path: &Path,
    text: &str,
) -> Option<ParsedSourceGraph> {
    registry::parse_source_graph_for_language(language, path, text)
}

pub(crate) fn structural_shape(node: Node<'_>) -> (String, usize) {
    let mut kinds = Vec::new();
    collect_structural_shape(node, &mut kinds);
    let node_count = kinds.len();
    (kinds.join(">"), node_count)
}

pub(crate) fn normalized_shape_fingerprints(node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut events = Vec::new();
    collect_normalized_shape_events(node, source, &mut events);

    let mut rows = literal_access_rows(&events);
    rows.sort();
    rows.dedup();

    let mut fingerprints = Vec::new();
    if rows.len() >= 3 {
        fingerprints.push(format!("literal-access-map:{}", rows.join("|")));
    }

    fingerprints
}

#[derive(Debug, Clone)]
enum NormalizedShapeEvent {
    Literal(String),
    Access(String),
}

fn collect_normalized_shape_events(
    node: Node<'_>,
    source: &[u8],
    events: &mut Vec<NormalizedShapeEvent>,
) {
    if node.kind() == "token_tree" {
        if let Ok(text) = node.utf8_text(source) {
            collect_token_tree_shape_events(text, events);
        }
        return;
    }
    if is_access_node(node)
        && !node.parent().is_some_and(is_access_node)
        && let Some(access) = normalized_access_text(node, source)
    {
        events.push(NormalizedShapeEvent::Access(access));
        return;
    }

    if is_string_literal_node(node)
        && let Some(label) = normalized_literal_text(node, source)
    {
        events.push(NormalizedShapeEvent::Literal(label));
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_normalized_shape_events(child, source, events);
    }
}

fn is_access_node(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "field_expression" | "member_expression" | "selector_expression" | "attribute"
    )
}

fn is_string_literal_node(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "string_literal"
            | "raw_string_literal"
            | "string"
            | "interpreted_string_literal"
            | "template_string"
    )
}

fn normalized_literal_text(node: Node<'_>, source: &[u8]) -> Option<String> {
    let raw = node.utf8_text(source).ok()?.trim();
    normalized_literal_value(raw)
}

fn normalized_literal_value(raw: &str) -> Option<String> {
    let text = strip_literal_quotes(raw);
    let text = text
        .split('{')
        .next()
        .unwrap_or(text)
        .replace("\\n", " ")
        .replace("\\t", " ");
    let words = text
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .take(6)
        .map(|part| part.to_ascii_lowercase())
        .collect::<Vec<_>>();
    if words.is_empty() {
        None
    } else {
        Some(words.join("-"))
    }
}

fn collect_token_tree_shape_events(text: &str, events: &mut Vec<NormalizedShapeEvent>) {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let ch = bytes[index] as char;
        if matches!(ch, '"' | '\'' | '`') {
            let start = index;
            index += 1;
            while index < bytes.len() {
                let current = bytes[index] as char;
                if current == '\\' {
                    index = (index + 2).min(bytes.len());
                    continue;
                }
                index += 1;
                if current == ch {
                    break;
                }
            }
            if let Some(label) = normalized_literal_value(&text[start..index.min(bytes.len())]) {
                events.push(NormalizedShapeEvent::Literal(label));
            }
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = index;
            index += 1;
            while index < bytes.len() {
                let current = bytes[index] as char;
                if current.is_ascii_alphanumeric() || matches!(current, '_' | '.') {
                    index += 1;
                } else {
                    break;
                }
            }
            let candidate = &text[start..index];
            if candidate.contains('.')
                && let Some(access) = normalized_access_value(candidate)
            {
                events.push(NormalizedShapeEvent::Access(access));
            }
            continue;
        }

        index += 1;
    }
}

fn strip_literal_quotes(raw: &str) -> &str {
    let mut text = raw.trim();
    loop {
        let Some(first) = text.chars().next() else {
            return text;
        };
        if !matches!(first, 'r' | 'b' | 'f') {
            break;
        }
        text = text[first.len_utf8()..].trim_start_matches('#');
    }
    text.trim_matches(['"', '\'', '`', '#'])
}

fn normalized_access_text(node: Node<'_>, source: &[u8]) -> Option<String> {
    normalized_access_value(node.utf8_text(source).ok()?)
}

fn normalized_access_value(raw: &str) -> Option<String> {
    let mut text = raw.to_string();
    text.retain(|ch| !ch.is_whitespace());
    for suffix in [
        ".to_string",
        ".to_owned",
        ".as_str",
        ".clone",
        ".display",
        ".String",
    ] {
        if let Some(stripped) = text.strip_suffix(suffix) {
            text = stripped.to_string();
        }
    }
    let separator_count = text.matches('.').count();
    if separator_count == 0 || text.contains('"') || text.contains('\'') {
        return None;
    }
    let segments = text.split('.').collect::<Vec<_>>();
    let significant = if segments.len() > 2 {
        segments[1..].join(".")
    } else {
        segments.join(".")
    };
    if significant.len() < 3 || significant.len() > 120 {
        None
    } else {
        Some(significant)
    }
}

fn literal_access_rows(events: &[NormalizedShapeEvent]) -> Vec<String> {
    let mut rows = Vec::new();
    for (index, event) in events.iter().enumerate() {
        let NormalizedShapeEvent::Literal(label) = event else {
            continue;
        };
        let Some(access) = events[index + 1..]
            .iter()
            .take_while(|event| !matches!(event, NormalizedShapeEvent::Literal(_)))
            .find_map(|event| match event {
                NormalizedShapeEvent::Access(access) => Some(access),
                NormalizedShapeEvent::Literal(_) => None,
            })
        else {
            continue;
        };
        rows.push(format!("{label}->{access}"));
    }
    rows
}

fn collect_structural_shape(node: Node<'_>, kinds: &mut Vec<String>) {
    kinds.push(node.kind().to_string());
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_structural_shape(child, kinds);
    }
}

pub(super) fn build_qualified_name(
    module_path: &[String],
    container_path: &[String],
    name: &str,
) -> String {
    let mut segments = module_path.to_vec();
    segments.extend(container_path.iter().cloned());
    segments.push(name.to_string());
    segments.join("::")
}

pub(super) fn first_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).next()
}

pub(super) fn last_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).last()
}

pub(super) fn ancestor_name_segments(
    node: Node<'_>,
    source: &[u8],
    ancestor_kind: &str,
    name_field: &str,
) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == ancestor_kind
            && let Some(name) = parent
                .child_by_field_name(name_field)
                .and_then(|name| name.utf8_text(source).ok())
        {
            segments.push(name.to_string());
        }
        current = parent.parent();
    }
    segments.reverse();
    segments
}

pub(super) fn collect_calls_with(
    node: Node<'_>,
    source: &[u8],
    function_name: CallResolver,
) -> Vec<SourceCall> {
    let mut calls = Vec::new();
    collect_calls_with_inner(node, source, &mut calls, function_name);
    calls
}

fn collect_calls_with_inner(
    node: Node<'_>,
    source: &[u8],
    calls: &mut Vec<SourceCall>,
    function_name: CallResolver,
) {
    if matches!(node.kind(), "call_expression" | "call")
        && let Some(function) = node.child_by_field_name("function")
        && let Some((name, qualifier, syntax)) = function_name(function, source)
    {
        calls.push(SourceCall {
            name,
            qualifier,
            syntax,
            span: SourceSpan::from_tree_sitter(function),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_calls_with_inner(child, source, calls, function_name);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        CallSyntaxKind, SourceInvocationKind, SourceItemKind, SourceLanguage,
        parse_source_graph_for_language, parse_source_graph_for_language_at_path,
    };

    fn item_named<'a>(graph: &'a super::ParsedSourceGraph, name: &str) -> &'a super::SourceItem {
        graph
            .items
            .iter()
            .find(|item| item.name == name)
            .unwrap_or_else(|| panic!("item {name} should be present"))
    }

    fn item_qualified<'a>(
        graph: &'a super::ParsedSourceGraph,
        qualified_name: &str,
    ) -> &'a super::SourceItem {
        graph
            .items
            .iter()
            .find(|item| item.qualified_name == qualified_name)
            .unwrap_or_else(|| panic!("item {qualified_name} should be present"))
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.NORMALIZED_FINGERPRINTS
    fn rust_provider_collects_normalized_literal_access_maps() {
        let graph = parse_source_graph_for_language(
            SourceLanguage::new("rust"),
            r#"
pub struct RepoMetrics {
    pub alpha: Count,
    pub beta: Count,
    pub gamma: Count,
}

pub struct Count {
    pub total: usize,
}

pub struct CountRow {
    pub label: String,
    pub value: String,
}

pub fn render_rows(metrics: &RepoMetrics) -> Vec<CountRow> {
    vec![
        CountRow { label: "alpha".to_string(), value: metrics.alpha.total.to_string() },
        CountRow { label: "beta".to_string(), value: metrics.beta.total.to_string() },
        CountRow { label: "gamma".to_string(), value: metrics.gamma.total.to_string() },
    ]
}
"#,
        )
        .expect("rust graph should parse");

        let item = item_named(&graph, "render_rows");
        assert!(
            item.normalized_fingerprints.iter().any(|fingerprint| {
                fingerprint.contains("literal-access-map:")
                    && fingerprint.contains("alpha->alpha.total")
                    && fingerprint.contains("gamma->gamma.total")
            }),
            "{:?}",
            item.normalized_fingerprints
        );
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.GO_ITEMS_AND_CALLS
    fn go_provider_collects_items_and_calls() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::new("go"),
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
        let entry = item_named(&graph, "Entry");
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
        let helper = item_named(&graph, "helper");
        assert!(!helper.public);
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.RUST_ITEMS_AND_CALLS
    fn rust_provider_collects_items_and_calls() {
        let graph = parse_source_graph_for_language(
            SourceLanguage::new("rust"),
            r#"
use std::process::Command;

fn helper() {}

#[test]
fn verifies_demo() {
    helper();
    crate::shared::work();
    subject.run();
    Command::new(env!("CARGO_BIN_EXE_special")).output();
}
"#,
        )
        .expect("rust graph should parse");

        assert_eq!(graph.items.len(), 2);
        let helper = item_named(&graph, "helper");
        assert_eq!(helper.qualified_name, "helper");
        assert!(helper.module_path.is_empty());
        assert!(helper.container_path.is_empty());
        assert_eq!(helper.kind, SourceItemKind::Function);
        assert!(!helper.public);
        assert!(!helper.is_test);

        let test_item = item_named(&graph, "verifies_demo");
        assert_eq!(test_item.name, "verifies_demo");
        assert_eq!(test_item.qualified_name, "verifies_demo");
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
        assert_eq!(test_item.invocations.len(), 1);
        assert_eq!(
            test_item.invocations[0].kind,
            SourceInvocationKind::LocalCargoBinary {
                binary_name: "special".to_string()
            }
        );
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.RUST_TEST_DETECTION
    fn rust_provider_avoids_false_positive_test_and_invocation_detection() {
        let graph = parse_source_graph_for_language(
            SourceLanguage::new("rust"),
            r#"
#[cfg(not(test))]
fn helper() {
    format!("{}", env!("CARGO_BIN_EXE_special"));
}
"#,
        )
        .expect("rust graph should parse");

        assert_eq!(graph.items.len(), 1);
        let helper = item_named(&graph, "helper");
        assert!(!helper.is_test);
        assert!(helper.invocations.is_empty());
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.RUST_ITEMS_AND_CALLS
    fn rust_provider_records_stable_and_qualified_item_names() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::new("rust"),
            Path::new("src/lib.rs"),
            r#"
mod nested {
    struct Worker;

    impl Worker {
        fn run() {}
    }
}
"#,
        )
        .expect("rust graph should parse");

        assert_eq!(graph.items.len(), 1);
        let method = item_qualified(&graph, "nested::Worker::run");
        assert_eq!(method.name, "run");
        assert_eq!(method.qualified_name, "nested::Worker::run");
        assert_eq!(method.container_path, vec!["Worker".to_string()]);
        assert!(method.stable_id.contains("nested::Worker::run"));
        assert!(!method.public);
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.RUST_ITEMS_AND_CALLS
    fn rust_provider_includes_file_module_path_in_qualified_names() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::new("rust"),
            Path::new("src/render/html.rs"),
            "pub fn render_spec_html() {}\n",
        )
        .expect("rust graph should parse");

        assert_eq!(graph.items.len(), 1);
        let function = item_qualified(&graph, "render::html::render_spec_html");
        assert_eq!(
            function.module_path,
            vec!["render".to_string(), "html".to_string()]
        );
        assert!(function.container_path.is_empty());
        assert_eq!(function.qualified_name, "render::html::render_spec_html");
        assert!(function.public);
        assert!(
            function
                .stable_id
                .contains("render::html::render_spec_html")
        );
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.RUST_ITEMS_AND_CALLS
    fn rust_provider_collects_trait_impl_methods() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::new("rust"),
            Path::new("src/lib.rs"),
            r#"
use std::fmt;

pub fn render_summary() -> String {
    format!("{}", Report)
}

struct Report;

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("report")
    }
}

pub fn orphan_impl() {}
"#,
        )
        .expect("rust graph should parse");

        assert_eq!(graph.items.len(), 3);
        let method = item_qualified(&graph, "Report::fmt");
        assert_eq!(method.name, "fmt");
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.TYPESCRIPT_ITEMS_AND_CALLS
    fn typescript_provider_collects_items_and_calls() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::new("typescript"),
            Path::new("src/example.ts"),
            r#"
export function entry() {
    helper();
    api.run();
}

function helper() {}

export const render = () => {
    helper();
};
"#,
        )
        .expect("typescript graph should parse");

        assert_eq!(graph.items.len(), 3);
        let entry = item_named(&graph, "entry");
        assert!(entry.public);
        assert!(
            entry
                .calls
                .iter()
                .any(|call| call.name == "helper" && call.qualifier.is_none())
        );
        assert!(entry.calls.iter().any(|call| {
            call.name == "run"
                && call.qualifier.as_deref() == Some("api")
                && call.syntax == CallSyntaxKind::Field
        }));

        let helper = item_named(&graph, "helper");
        assert!(!helper.public);

        let render = item_named(&graph, "render");
        assert!(render.public);
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.TYPESCRIPT_TEST_CALLBACKS
    fn typescript_provider_collects_inline_test_callbacks() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::new("typescript"),
            Path::new("src/example.test.ts"),
            r#"
import { liveImpl } from "./app";

it("covers live behavior", async () => {
    await liveImpl();
});

test.only("covers alternate behavior", () => {
    liveImpl();
});

test.each([["first"]])("covers %s behavior", () => {
    liveImpl();
});

it.concurrent.each([["second"]])("covers %s behavior", async () => {
    await liveImpl();
});
"#,
        )
        .expect("typescript graph should parse");

        let test_roots = graph
            .items
            .iter()
            .filter(|item| item.is_test)
            .collect::<Vec<_>>();
        assert_eq!(test_roots.len(), 4);
        assert!(test_roots.iter().any(|item| {
            item.name == "it"
                && item.span.start_line == 4
                && item.calls.iter().any(|call| call.name == "liveImpl")
        }));
        assert!(test_roots.iter().any(|item| {
            item.name == "test.only"
                && item.span.start_line == 8
                && item.calls.iter().any(|call| call.name == "liveImpl")
        }));
        assert!(test_roots.iter().any(|item| {
            item.name == "test.each"
                && item.span.start_line == 12
                && item.calls.iter().any(|call| call.name == "liveImpl")
        }));
        assert!(test_roots.iter().any(|item| {
            item.name == "it.concurrent.each"
                && item.span.start_line == 16
                && item.calls.iter().any(|call| call.name == "liveImpl")
        }));
    }

    #[test]
    // @verifies SPECIAL.SYNTAX.PROVIDERS.TYPESCRIPT_TEST_CALLBACKS
    fn typescript_provider_does_not_collect_test_callbacks_outside_test_files() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::new("typescript"),
            Path::new("src/example.ts"),
            r#"
it("ordinary source should not become a test root", () => {
    liveImpl();
});
"#,
        )
        .expect("typescript graph should parse");

        assert!(graph.items.is_empty());
    }
}
