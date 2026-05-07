/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.COMPLEXITY
Extracts lightweight Rust function complexity summaries from owned Rust implementation for architecture evidence without turning the numbers into architecture verdicts.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.COMPLEXITY
use syn::{ImplItemFn, Item};

use crate::model::ModuleComplexitySummary;

use super::item_metrics::{RustItemObserver, function_metrics, method_metrics};

#[derive(Debug, Default)]
pub(super) struct RustComplexitySummary {
    function_count: usize,
    total_cyclomatic: usize,
    max_cyclomatic: usize,
    total_cognitive: usize,
    max_cognitive: usize,
}

impl RustComplexitySummary {
    pub(super) fn finish(self) -> ModuleComplexitySummary {
        ModuleComplexitySummary {
            function_count: self.function_count,
            total_cyclomatic: self.total_cyclomatic,
            max_cyclomatic: self.max_cyclomatic,
            total_cognitive: self.total_cognitive,
            max_cognitive: self.max_cognitive,
        }
    }

    fn observe_method(&mut self, method: &ImplItemFn) {
        let metrics = method_metrics(method);
        self.observe_complexity(metrics.cyclomatic, metrics.cognitive);
    }

    fn observe_complexity(&mut self, cyclomatic: usize, cognitive: usize) {
        self.function_count += 1;
        self.total_cyclomatic += cyclomatic;
        self.max_cyclomatic = self.max_cyclomatic.max(cyclomatic);
        self.total_cognitive += cognitive;
        self.max_cognitive = self.max_cognitive.max(cognitive);
    }
}

impl RustItemObserver for RustComplexitySummary {
    fn observe_item(&mut self, item: &Item) {
        match item {
            Item::Fn(function) => {
                let metrics =
                    function_metrics(&function.vis, &function.sig.inputs, &function.block);
                self.observe_complexity(metrics.cyclomatic, metrics.cognitive);
            }
            Item::Impl(item_impl) => {
                for impl_item in &item_impl.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        self.observe_method(method);
                    }
                }
            }
            Item::Mod(item_mod) => {
                if let Some((_, nested)) = &item_mod.content {
                    for item in nested {
                        self.observe_item(item);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::{ImplItem, ItemImpl};

    use super::*;

    fn first_method(source: &str) -> ImplItemFn {
        let item = syn::parse_str::<ItemImpl>(source).expect("impl should parse");
        item.items
            .into_iter()
            .find_map(|item| match item {
                ImplItem::Fn(method) => Some(method),
                _ => None,
            })
            .expect("impl should contain a method")
    }

    #[test]
    fn provider_complexity_summary_observes_methods_through_shared_metrics() {
        let method = first_method("impl W { pub fn run(&self) { if true {} } }");
        let mut summary = RustComplexitySummary::default();

        summary.observe_method(&method);
        summary.observe_complexity(3, 2);
        let metrics = summary.finish();

        assert_eq!(metrics.function_count, 2);
        assert_eq!(metrics.max_cyclomatic, 3);
        assert_eq!(metrics.total_cognitive, 3);
    }
}
