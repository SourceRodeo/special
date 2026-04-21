/**
@module SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.MODULE_VIEW
Projects per-module analysis summaries into shared count, explanation, and detail rows for renderers.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.MODULE_VIEW
use crate::model::ModuleNode;

#[path = "module_view/complexity.rs"]
mod complexity;
#[path = "module_view/coupling.rs"]
mod coupling;
#[path = "module_view/coverage_metrics.rs"]
mod coverage_metrics;
#[path = "module_view/dependencies.rs"]
mod dependencies;
#[path = "module_view/item_signals.rs"]
mod item_signals;
#[path = "module_view/quality.rs"]
mod quality;
#[path = "module_view/support.rs"]
mod support;
#[path = "module_view/traceability.rs"]
mod traceability;

use super::{ProjectedCount, ProjectedExplanation, ProjectedMetaLine, ProjectedModuleAnalysis};

pub(in crate::render) fn project_module_analysis_view(
    node: &ModuleNode,
    verbose: bool,
) -> Option<ProjectedModuleAnalysis> {
    let analysis = node.analysis.as_ref()?;
    let mut counts = Vec::<ProjectedCount>::new();
    let mut meta_lines = Vec::<ProjectedMetaLine>::new();
    let mut explanations = Vec::<ProjectedExplanation>::new();

    coverage_metrics::append_coverage(analysis, &mut counts);
    coverage_metrics::append_metrics(analysis, &mut counts);
    complexity::append_complexity(analysis, verbose, &mut counts, &mut explanations);
    quality::append_quality(analysis, verbose, &mut counts, &mut explanations);
    item_signals::append_item_signals(
        analysis,
        verbose,
        &mut counts,
        &mut meta_lines,
        &mut explanations,
    );
    traceability::append_traceability(analysis, verbose, &mut meta_lines);
    coupling::append_coupling(analysis, verbose, &mut counts, &mut explanations);
    dependencies::append_dependencies(analysis, verbose, &mut counts, &mut meta_lines);

    Some(ProjectedModuleAnalysis {
        counts,
        meta_lines,
        explanations,
    })
}
