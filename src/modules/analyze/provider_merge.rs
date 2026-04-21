/**
@module SPECIAL.MODULES.ANALYZE.PROVIDER_MERGE
Owns shared merging of provider-level analysis summaries into module and repo analysis views.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.PROVIDER_MERGE
use std::collections::BTreeMap;

use crate::model::{
    ArchitectureTraceabilitySummary, ModuleComplexitySummary, ModuleDependencySummary,
    ModuleDependencyTargetSummary, ModuleItemSignal, ModuleItemSignalsSummary,
    ModuleMetricsSummary, ModuleQualitySummary, ModuleTraceabilitySummary,
};

use super::ProviderModuleAnalysis;
use super::coupling;

pub(crate) fn build_dependency_summary(
    targets: &BTreeMap<String, usize>,
) -> ModuleDependencySummary {
    ModuleDependencySummary {
        reference_count: targets.values().sum(),
        distinct_targets: targets.len(),
        targets: targets
            .iter()
            .map(|(path, count)| ModuleDependencyTargetSummary {
                path: path.clone(),
                count: *count,
            })
            .collect(),
    }
}

pub(super) fn merge_provider_module_analysis(
    summary: &mut ProviderModuleAnalysis,
    delta: ProviderModuleAnalysis,
) {
    merge_metrics(&mut summary.metrics, delta.metrics);
    merge_optional_complexity(&mut summary.complexity, delta.complexity);
    merge_optional_quality(&mut summary.quality, delta.quality);
    merge_optional_item_signals(&mut summary.item_signals, delta.item_signals);
    merge_optional_traceability(&mut summary.traceability, delta.traceability);
    if summary.traceability_unavailable_reason.is_none() {
        summary.traceability_unavailable_reason = delta.traceability_unavailable_reason;
    }
    merge_optional_coupling_input(&mut summary.coupling, delta.coupling);
    merge_optional_dependencies(&mut summary.dependencies, delta.dependencies);
}

pub(super) fn merge_optional_repo_traceability(
    summary: &mut Option<ArchitectureTraceabilitySummary>,
    delta: Option<ArchitectureTraceabilitySummary>,
) {
    let Some(delta) = delta else {
        return;
    };
    let target = summary.get_or_insert_with(ArchitectureTraceabilitySummary::default);
    target.extend_from(delta);
}

fn merge_metrics(summary: &mut ModuleMetricsSummary, delta: ModuleMetricsSummary) {
    summary.public_items += delta.public_items;
    summary.internal_items += delta.internal_items;
}

fn merge_optional_complexity(
    summary: &mut Option<ModuleComplexitySummary>,
    delta: Option<ModuleComplexitySummary>,
) {
    let Some(delta) = delta else {
        return;
    };
    let target = summary.get_or_insert_with(ModuleComplexitySummary::default);
    target.function_count += delta.function_count;
    target.total_cyclomatic += delta.total_cyclomatic;
    target.max_cyclomatic = target.max_cyclomatic.max(delta.max_cyclomatic);
    target.total_cognitive += delta.total_cognitive;
    target.max_cognitive = target.max_cognitive.max(delta.max_cognitive);
}

fn merge_optional_quality(
    summary: &mut Option<ModuleQualitySummary>,
    delta: Option<ModuleQualitySummary>,
) {
    let Some(delta) = delta else {
        return;
    };
    let target = summary.get_or_insert_with(ModuleQualitySummary::default);
    target.public_function_count += delta.public_function_count;
    target.parameter_count += delta.parameter_count;
    target.bool_parameter_count += delta.bool_parameter_count;
    target.raw_string_parameter_count += delta.raw_string_parameter_count;
    target.panic_site_count += delta.panic_site_count;
}

fn merge_optional_item_signals(
    summary: &mut Option<ModuleItemSignalsSummary>,
    delta: Option<ModuleItemSignalsSummary>,
) {
    let Some(delta) = delta else {
        return;
    };
    let target = summary.get_or_insert_with(ModuleItemSignalsSummary::default);
    target.analyzed_items += delta.analyzed_items;
    target.unreached_item_count += delta.unreached_item_count;
    merge_item_signal_group(&mut target.connected_items, delta.connected_items);
    merge_item_signal_group(&mut target.outbound_heavy_items, delta.outbound_heavy_items);
    merge_item_signal_group(&mut target.isolated_items, delta.isolated_items);
    merge_item_signal_group(&mut target.unreached_items, delta.unreached_items);
    merge_item_signal_group(
        &mut target.highest_complexity_items,
        delta.highest_complexity_items,
    );
    merge_item_signal_group(
        &mut target.parameter_heavy_items,
        delta.parameter_heavy_items,
    );
    merge_item_signal_group(
        &mut target.stringly_boundary_items,
        delta.stringly_boundary_items,
    );
    merge_item_signal_group(&mut target.panic_heavy_items, delta.panic_heavy_items);
}

fn merge_item_signal_group(target: &mut Vec<ModuleItemSignal>, mut delta: Vec<ModuleItemSignal>) {
    target.append(&mut delta);
}

fn merge_optional_traceability(
    summary: &mut Option<ModuleTraceabilitySummary>,
    delta: Option<ModuleTraceabilitySummary>,
) {
    if summary.is_none() {
        *summary = delta;
    }
}

fn merge_optional_coupling_input(
    summary: &mut Option<coupling::ModuleCouplingInput>,
    delta: Option<coupling::ModuleCouplingInput>,
) {
    let Some(delta) = delta else {
        return;
    };
    let target = summary.get_or_insert_with(coupling::ModuleCouplingInput::default);
    target.internal_files.extend(delta.internal_files);
    target.external_targets.extend(delta.external_targets);
}

fn merge_optional_dependencies(
    summary: &mut Option<ModuleDependencySummary>,
    delta: Option<ModuleDependencySummary>,
) {
    let Some(delta) = delta else {
        return;
    };
    let mut merged = summary
        .take()
        .map(|summary| {
            summary
                .targets
                .into_iter()
                .map(|target| (target.path, target.count))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    for target in delta.targets {
        *merged.entry(target.path).or_default() += target.count;
    }
    let reference_count = merged.values().sum();
    let distinct_targets = merged.len();
    *summary = Some(ModuleDependencySummary {
        reference_count,
        distinct_targets,
        targets: merged
            .into_iter()
            .map(|(path, count)| ModuleDependencyTargetSummary { path, count })
            .collect(),
    });
}

#[cfg(test)]
mod tests {
    use crate::model::{ModuleDependencyTargetSummary, ModuleTraceabilitySummary};

    use super::*;

    #[test]
    fn module_traceability_merge_keeps_first_provider_summary() {
        let mut summary = ProviderModuleAnalysis {
            metrics: ModuleMetricsSummary::default(),
            traceability: Some(ModuleTraceabilitySummary {
                analyzed_items: 1,
                ..ModuleTraceabilitySummary::default()
            }),
            traceability_unavailable_reason: Some("first".to_string()),
            ..ProviderModuleAnalysis::default()
        };

        merge_provider_module_analysis(
            &mut summary,
            ProviderModuleAnalysis {
                metrics: ModuleMetricsSummary::default(),
                traceability: Some(ModuleTraceabilitySummary {
                    analyzed_items: 2,
                    ..ModuleTraceabilitySummary::default()
                }),
                traceability_unavailable_reason: Some("second".to_string()),
                ..ProviderModuleAnalysis::default()
            },
        );

        assert_eq!(
            summary
                .traceability
                .as_ref()
                .map(|traceability| traceability.analyzed_items),
            Some(1)
        );
        assert_eq!(
            summary.traceability_unavailable_reason.as_deref(),
            Some("first")
        );
    }

    #[test]
    fn dependency_merge_accumulates_counts_by_target() {
        let mut summary = ProviderModuleAnalysis {
            metrics: ModuleMetricsSummary::default(),
            dependencies: Some(ModuleDependencySummary {
                reference_count: 2,
                distinct_targets: 2,
                targets: vec![
                    ModuleDependencyTargetSummary {
                        path: "crate::shared".to_string(),
                        count: 1,
                    },
                    ModuleDependencyTargetSummary {
                        path: "serde".to_string(),
                        count: 1,
                    },
                ],
            }),
            ..ProviderModuleAnalysis::default()
        };

        merge_provider_module_analysis(
            &mut summary,
            ProviderModuleAnalysis {
                metrics: ModuleMetricsSummary::default(),
                dependencies: Some(ModuleDependencySummary {
                    reference_count: 3,
                    distinct_targets: 2,
                    targets: vec![
                        ModuleDependencyTargetSummary {
                            path: "crate::shared".to_string(),
                            count: 2,
                        },
                        ModuleDependencyTargetSummary {
                            path: "tokio".to_string(),
                            count: 1,
                        },
                    ],
                }),
                ..ProviderModuleAnalysis::default()
            },
        );

        let dependencies = summary
            .dependencies
            .expect("merged dependency summary should be present");
        assert_eq!(dependencies.reference_count, 5);
        assert_eq!(dependencies.distinct_targets, 3);
        assert_eq!(
            dependencies
                .targets
                .into_iter()
                .map(|target| (target.path, target.count))
                .collect::<BTreeMap<_, _>>(),
            BTreeMap::from([
                ("crate::shared".to_string(), 3),
                ("serde".to_string(), 1),
                ("tokio".to_string(), 1),
            ])
        );
    }
}
