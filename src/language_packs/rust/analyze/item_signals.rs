/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.ITEM_SIGNALS
Adapts normalized Rust source graphs and Rust item metrics into the shared source-item signal engine so Rust item connectivity, unreached-code, complexity, and craftsmanship evidence uses the same summarization rules as other built-in packs.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.ITEM_SIGNALS
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;

use syn::{ImplItemFn, Item};

use crate::model::ModuleItemSignalsSummary;
use crate::modules::analyze::source_item_signals::{
    SourceItemMetrics, summarize_source_item_signals_with_metrics,
};
use crate::syntax::{SourceItem, SourceItemKind, parse_source_graph};

use super::item_metrics::{RustItemMetrics, function_metrics, method_metrics};

#[derive(Default)]
pub(super) struct RustItemSignalsSummary {
    items: Vec<SourceItem>,
    metrics_by_stable_id: BTreeMap<String, SourceItemMetrics>,
}

impl RustItemSignalsSummary {
    pub(super) fn observe(&mut self, path: &Path, text: &str) {
        let Some(graph) = parse_source_graph(path, text) else {
            return;
        };
        let mut metrics_by_item = collect_item_metrics(text);
        for item in graph.items {
            if let Some(metrics) = metrics_by_item
                .get_mut(&(item.name.clone(), item.kind))
                .and_then(VecDeque::pop_front)
            {
                self.metrics_by_stable_id
                    .insert(item.stable_id.clone(), metrics);
            }
            self.items.push(item);
        }
    }

    pub(super) fn finish(self) -> ModuleItemSignalsSummary {
        summarize_source_item_signals_with_metrics(&self.items, &self.metrics_by_stable_id)
    }
}

fn collect_item_metrics(text: &str) -> BTreeMap<(String, SourceItemKind), VecDeque<SourceItemMetrics>> {
    let mut metrics = BTreeMap::new();
    if let Ok(file) = syn::parse_file(text) {
        collect_metrics_from_items(&file.items, &mut metrics);
        return metrics;
    }

    if let Ok(item) = syn::parse_str::<Item>(text) {
        collect_metrics_from_items(std::slice::from_ref(&item), &mut metrics);
    }
    metrics
}

fn collect_metrics_from_items(
    items: &[Item],
    metrics: &mut BTreeMap<(String, SourceItemKind), VecDeque<SourceItemMetrics>>,
) {
    for item in items {
        match item {
            Item::Fn(function) => {
                metrics
                    .entry((function.sig.ident.to_string(), SourceItemKind::Function))
                    .or_default()
                    .push_back(source_metrics(function_metrics(
                        &function.vis,
                        &function.sig.inputs,
                        &function.block,
                    )));
            }
            Item::Impl(item_impl) => {
                for impl_item in &item_impl.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        collect_method_metrics(method, metrics);
                    }
                }
            }
            Item::Mod(item_mod) => {
                if let Some((_, nested)) = &item_mod.content {
                    collect_metrics_from_items(nested, metrics);
                }
            }
            _ => {}
        }
    }
}

fn collect_method_metrics(
    method: &ImplItemFn,
    metrics: &mut BTreeMap<(String, SourceItemKind), VecDeque<SourceItemMetrics>>,
) {
    metrics
        .entry((method.sig.ident.to_string(), SourceItemKind::Method))
        .or_default()
        .push_back(source_metrics(method_metrics(method)));
}

fn source_metrics(metrics: RustItemMetrics) -> SourceItemMetrics {
    SourceItemMetrics {
        parameter_count: metrics.parameter_count,
        bool_parameter_count: metrics.bool_parameter_count,
        raw_string_parameter_count: metrics.raw_string_parameter_count,
        cyclomatic: metrics.cyclomatic,
        cognitive: metrics.cognitive,
        panic_site_count: metrics.panic_site_count,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::RustItemSignalsSummary;

    #[test]
    fn provider_item_signals_use_shared_source_graph_calls_and_rust_metrics() {
        let mut summary = RustItemSignalsSummary::default();
        summary.observe(
            Path::new("src/lib.rs"),
            r#"
pub fn entry(name: &str, enabled: bool) {
    helper(name);
    println!("{enabled}");
}

fn helper(name: &str) {
    println!("{name}");
}

fn isolated() {
    println!("alone");
}
"#,
        );

        let signals = summary.finish();
        assert_eq!(signals.analyzed_items, 3);
        assert_eq!(signals.unreached_item_count, 1);
        assert!(
            signals
                .connected_items
                .iter()
                .any(|item| item.name == "helper" && item.inbound_internal_refs == 1)
        );
        assert!(
            signals
                .parameter_heavy_items
                .iter()
                .any(|item| item.name == "entry" && item.bool_parameter_count == 1)
        );
    }
}
