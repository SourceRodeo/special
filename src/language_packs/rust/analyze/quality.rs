/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.QUALITY
Extracts Rust quality evidence from owned Rust implementation, focusing on public API parameter shape, stringly typed boundaries, and recoverability signals without turning those signals into verdicts.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.QUALITY
use syn::{ImplItemFn, Item};

use crate::model::ModuleQualitySummary;

use super::item_metrics::{RustItemMetrics, RustItemObserver, function_metrics, method_metrics};

#[derive(Debug, Default)]
pub(super) struct RustQualitySummary {
    public_function_count: usize,
    parameter_count: usize,
    bool_parameter_count: usize,
    raw_string_parameter_count: usize,
    panic_site_count: usize,
}

impl RustQualitySummary {
    pub(super) fn finish(self) -> ModuleQualitySummary {
        ModuleQualitySummary {
            public_function_count: self.public_function_count,
            parameter_count: self.parameter_count,
            bool_parameter_count: self.bool_parameter_count,
            raw_string_parameter_count: self.raw_string_parameter_count,
            panic_site_count: self.panic_site_count,
        }
    }

    fn observe_method(&mut self, method: &ImplItemFn) {
        self.observe_metrics(method_metrics(method));
    }

    fn observe_metrics(&mut self, metrics: RustItemMetrics) {
        if metrics.public {
            self.public_function_count += 1;
            self.parameter_count += metrics.parameter_count;
            self.bool_parameter_count += metrics.bool_parameter_count;
            self.raw_string_parameter_count += metrics.raw_string_parameter_count;
        }
        self.panic_site_count += metrics.panic_site_count;
    }
}

impl RustItemObserver for RustQualitySummary {
    fn observe_item(&mut self, item: &Item) {
        match item {
            Item::Fn(function) => {
                self.observe_metrics(function_metrics(
                    &function.vis,
                    &function.sig.inputs,
                    &function.block,
                ));
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
    fn provider_quality_summary_observes_methods_through_shared_metrics() {
        let method = first_method("impl W { pub fn run(&self, flag: bool, name: &str) { panic!(); } }");
        let mut summary = RustQualitySummary::default();

        summary.observe_method(&method);
        summary.observe_metrics(RustItemMetrics {
            public: true,
            parameter_count: 1,
            bool_parameter_count: 0,
            raw_string_parameter_count: 0,
            cyclomatic: 1,
            cognitive: 0,
            panic_site_count: 0,
        });
        let metrics = summary.finish();

        assert_eq!(metrics.public_function_count, 2);
        assert_eq!(metrics.parameter_count, 4);
        assert_eq!(metrics.bool_parameter_count, 1);
        assert_eq!(metrics.raw_string_parameter_count, 1);
        assert_eq!(metrics.panic_site_count, 1);
    }
}
