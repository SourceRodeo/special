/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.QUALITY
Extracts Rust quality, complexity, and per-item craftsmanship evidence from the same parser-backed source graph used for Rust item and call discovery.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.QUALITY
use std::collections::BTreeMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::model::{ModuleComplexitySummary, ModuleQualitySummary};
use crate::modules::analyze::source_item_signals::SourceItemMetrics;
use crate::syntax::{ParsedSourceGraph, SourceItem};
use crate::tree_sitter_utils::{first_named_child_with_kind, is_boolean_binary_expression};

type ItemMetricKey = (usize, usize, String);

#[derive(Debug, Default)]
pub(super) struct RustQualitySummary {
    public_function_count: usize,
    parameter_count: usize,
    bool_parameter_count: usize,
    raw_string_parameter_count: usize,
    panic_site_count: usize,
    function_count: usize,
    total_cyclomatic: usize,
    max_cyclomatic: usize,
    total_cognitive: usize,
    max_cognitive: usize,
    item_metrics: BTreeMap<String, SourceItemMetrics>,
}

impl RustQualitySummary {
    pub(super) fn observe(&mut self, _path: &Path, text: &str, graph: &ParsedSourceGraph) {
        let mut parser = Parser::new();
        if parser.set_language(&tree_sitter_rust::LANGUAGE.into()).is_err() {
            return;
        }
        let Some(tree) = parser.parse(text, None) else {
            return;
        };

        let source = text.as_bytes();
        let mut metrics_by_item = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics_by_item);

        for item in &graph.items {
            if let Some(metrics) = metrics_by_item.get(&item_key(item)).copied() {
                self.function_count += 1;
                self.total_cyclomatic += metrics.cyclomatic;
                self.max_cyclomatic = self.max_cyclomatic.max(metrics.cyclomatic);
                self.total_cognitive += metrics.cognitive;
                self.max_cognitive = self.max_cognitive.max(metrics.cognitive);
                self.panic_site_count += metrics.panic_site_count;
                self.item_metrics.insert(item.stable_id.clone(), metrics);
            }
            if item.public {
                self.public_function_count += 1;
            }
            if item.public
                && let Some(metrics) = metrics_by_item.get(&item_key(item))
            {
                self.parameter_count += metrics.parameter_count;
                self.bool_parameter_count += metrics.bool_parameter_count;
                self.raw_string_parameter_count += metrics.raw_string_parameter_count;
            }
        }
    }

    pub(super) fn finish_complexity(&self) -> ModuleComplexitySummary {
        ModuleComplexitySummary {
            function_count: self.function_count,
            total_cyclomatic: self.total_cyclomatic,
            max_cyclomatic: self.max_cyclomatic,
            total_cognitive: self.total_cognitive,
            max_cognitive: self.max_cognitive,
        }
    }

    pub(super) fn item_metrics(&self) -> &BTreeMap<String, SourceItemMetrics> {
        &self.item_metrics
    }

    pub(super) fn finish(self) -> ModuleQualitySummary {
        ModuleQualitySummary {
            public_function_count: self.public_function_count,
            parameter_count: self.parameter_count,
            bool_parameter_count: self.bool_parameter_count,
            raw_string_parameter_count: self.raw_string_parameter_count,
            panic_site_count: self.panic_site_count,
        }
    }
}

fn collect_callable_metrics(
    node: Node<'_>,
    source: &[u8],
    metrics_by_item: &mut BTreeMap<ItemMetricKey, SourceItemMetrics>,
) {
    if node.kind() == "function_item"
        && let Some((line, column, name, parameters)) = callable_parts(node, source)
    {
        metrics_by_item.insert((line, column, name), item_metrics(node, parameters, source));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_callable_metrics(child, source, metrics_by_item);
    }
}

fn callable_parts<'tree>(
    node: Node<'tree>,
    source: &[u8],
) -> Option<(usize, usize, String, Node<'tree>)> {
    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .to_string();
    let parameters = node
        .child_by_field_name("parameters")
        .or_else(|| first_named_child_with_kind(node, "parameters"))?;
    Some((
        node.start_position().row + 1,
        node.start_position().column,
        name,
        parameters,
    ))
}

fn item_metrics(
    callable_node: Node<'_>,
    parameter_node: Node<'_>,
    source: &[u8],
) -> SourceItemMetrics {
    let mut metrics = parameter_metrics(parameter_node, source);
    metrics.cyclomatic = cyclomatic_complexity(callable_node, source);
    metrics.cognitive = cognitive_complexity(callable_node, source);
    metrics.panic_site_count = count_panic_sites(callable_node, source);
    metrics
}

fn parameter_metrics(node: Node<'_>, source: &[u8]) -> SourceItemMetrics {
    let mut metrics = SourceItemMetrics::default();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "parameter" | "variadic_parameter" => observe_typed_parameter(&mut metrics, child, source),
            "self_parameter" => metrics.parameter_count += 1,
            _ => {}
        }
    }
    metrics
}

fn observe_typed_parameter(metrics: &mut SourceItemMetrics, node: Node<'_>, source: &[u8]) {
    metrics.parameter_count += 1;
    if parameter_contains_type(node, source, is_bool_type_text) {
        metrics.bool_parameter_count += 1;
    }
    if parameter_contains_type(node, source, is_raw_string_type_text) {
        metrics.raw_string_parameter_count += 1;
    }
}

fn parameter_contains_type(
    node: Node<'_>,
    source: &[u8],
    predicate: fn(&str) -> bool,
) -> bool {
    node.child_by_field_name("type").is_some_and(|type_node| {
        type_node
            .utf8_text(source)
            .ok()
            .is_some_and(|text| predicate(text.trim()))
    })
}

fn is_bool_type_text(text: &str) -> bool {
    text.trim_start_matches('&').trim() == "bool"
}

fn is_raw_string_type_text(text: &str) -> bool {
    let compact = text.chars().filter(|ch| !ch.is_whitespace()).collect::<String>();
    matches!(compact.as_str(), "str" | "&str" | "String" | "&String")
        || (compact.contains("Cow<") && compact.contains("str"))
}

fn count_panic_sites(node: Node<'_>, source: &[u8]) -> usize {
    let mut count = usize::from(is_panic_macro(node, source) || is_recoverability_method_call(node, source));
    let mut cursor = node.walk();
    count += node
        .named_children(&mut cursor)
        .filter(|child| !is_nested_callable_boundary(*child))
        .map(|child| count_panic_sites(child, source))
        .sum::<usize>();
    count
}

fn is_panic_macro(node: Node<'_>, source: &[u8]) -> bool {
    if node.kind() != "macro_invocation" {
        return false;
    }
    node.utf8_text(source).ok().is_some_and(|text| {
        let name = text.split('!').next().unwrap_or_default().trim();
        matches!(name, "panic" | "todo" | "unimplemented" | "unreachable")
    })
}

fn is_recoverability_method_call(node: Node<'_>, source: &[u8]) -> bool {
    if node.kind() != "call_expression" {
        return false;
    }
    node.child_by_field_name("function")
        .and_then(|function| method_call_name(function, source))
        .is_some_and(|name| matches!(name.as_str(), "unwrap" | "expect"))
}

fn method_call_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    match node.kind() {
        "field_expression" => Some(
            node.child_by_field_name("field")?
                .utf8_text(source)
                .ok()?
                .to_string(),
        ),
        "generic_function" => method_call_name(node.child_by_field_name("function")?, source),
        _ => None,
    }
}

fn cyclomatic_complexity(node: Node<'_>, source: &[u8]) -> usize {
    let mut complexity = 1;
    add_cyclomatic_complexity(node, source, &mut complexity);
    complexity
}

fn add_cyclomatic_complexity(node: Node<'_>, source: &[u8], complexity: &mut usize) {
    match node.kind() {
        "if_expression" | "for_expression" | "while_expression" | "loop_expression" => {
            *complexity += 1;
        }
        "match_arm" => {
            *complexity += 1;
        }
        "binary_expression" if is_boolean_binary_expression(node, source) => {
            *complexity += 1;
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if is_nested_callable_boundary(child) {
            continue;
        }
        add_cyclomatic_complexity(child, source, complexity);
    }
}

fn cognitive_complexity(node: Node<'_>, source: &[u8]) -> usize {
    cognitive_complexity_at(node, source, 0)
}

fn cognitive_complexity_at(node: Node<'_>, source: &[u8], nesting: usize) -> usize {
    let structural = matches!(
        node.kind(),
        "if_expression"
            | "for_expression"
            | "while_expression"
            | "loop_expression"
            | "match_expression"
    );
    let boolean = node.kind() == "binary_expression" && is_boolean_binary_expression(node, source);

    let mut score = usize::from(boolean);
    let child_nesting = if structural {
        score += 1 + nesting;
        nesting + 1
    } else {
        nesting
    };

    let mut cursor = node.walk();
    score
        + node
            .named_children(&mut cursor)
            .filter(|child| !is_nested_callable_boundary(*child))
            .map(|child| cognitive_complexity_at(child, source, child_nesting))
            .sum::<usize>()
}

fn is_nested_callable_boundary(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_item" | "closure_expression")
}

fn item_key(item: &SourceItem) -> ItemMetricKey {
    (item.span.start_line, item.span.start_column, item.name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metric<'a>(
        metrics: &'a BTreeMap<ItemMetricKey, SourceItemMetrics>,
        line: usize,
        name: &str,
    ) -> Option<&'a SourceItemMetrics> {
        metrics
            .iter()
            .find(|((item_line, _, item_name), _)| *item_line == line && item_name == name)
            .map(|(_, metrics)| metrics)
    }

    #[test]
    fn rust_item_metrics_come_from_tree_sitter_source_graph_shape() {
        let source = br#"
pub fn entry(name: &str, enabled: bool, owned: String) {
    if enabled && !name.is_empty() {
        println!("{name}");
    }
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("rust grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let entry = metric(&metrics, 2, "entry").expect("entry should be measured");
        assert_eq!(entry.parameter_count, 3);
        assert_eq!(entry.raw_string_parameter_count, 2);
        assert_eq!(entry.bool_parameter_count, 1);
        assert_eq!(entry.cyclomatic, 3);
        assert_eq!(entry.cognitive, 2);
    }

    #[test]
    fn nested_callables_do_not_inflate_outer_rust_metrics() {
        let source = br#"
pub fn outer(flag: bool) {
    let inner = || {
        if flag {
            panic!("nested");
        }
    };
    fn local(flag: bool) {
        if flag {
            unreachable!();
        }
    }
    if flag {
        inner();
        local(flag);
    }
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("rust grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let outer = metric(&metrics, 2, "outer").expect("outer should be measured");
        assert_eq!(outer.cyclomatic, 2);
        assert_eq!(outer.cognitive, 1);
        assert_eq!(outer.panic_site_count, 0);

        let local = metric(&metrics, 8, "local").expect("local function should be measured");
        assert_eq!(local.cyclomatic, 2);
        assert_eq!(local.cognitive, 1);
        assert_eq!(local.panic_site_count, 1);
    }

    #[test]
    fn same_line_same_name_rust_methods_keep_distinct_metrics() {
        let source = r#"struct A; struct B; impl A { fn m(&self) {} } impl B { fn m(&self, value: &str) { if value.is_empty() { return; } } }"#;
        let path = Path::new("src/demo.rs");
        let graph = crate::syntax::parse_source_graph(path, source).expect("source should parse");
        let mut summary = RustQualitySummary::default();
        summary.observe(path, source, &graph);

        let method_metrics = graph
            .items
            .iter()
            .filter(|item| item.name == "m")
            .filter_map(|item| summary.item_metrics().get(&item.stable_id))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(method_metrics.len(), 2);
        assert!(method_metrics.iter().any(|metrics| {
            metrics.parameter_count == 1 && metrics.cyclomatic == 1 && metrics.cognitive == 0
        }));
        assert!(method_metrics.iter().any(|metrics| {
            metrics.parameter_count == 2
                && metrics.raw_string_parameter_count == 1
                && metrics.cyclomatic == 2
                && metrics.cognitive == 1
        }));
    }
}
