/**
@module SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS
Projects module analysis into a shared view model of counts, explanation rows, and supporting detail lines that text and HTML renderers can present differently.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS
use crate::modules::analyze::explain::{MetricExplanationKey, metric_explanation};

mod module_view;
mod repo_signals;
mod repo_traceability;

pub(in crate::render) use self::module_view::project_module_analysis_view;
pub(in crate::render) use self::repo_signals::project_repo_signals_view;
pub(in crate::render) use self::repo_traceability::project_repo_traceability_view;

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedCount {
    pub(in crate::render) label: &'static str,
    pub(in crate::render) value: String,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedMetaLine {
    pub(in crate::render) label: &'static str,
    pub(in crate::render) value: String,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedExplanation {
    pub(in crate::render) label: &'static str,
    pub(in crate::render) plain: &'static str,
    pub(in crate::render) precise: &'static str,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedRepoSignals {
    pub(in crate::render) counts: Vec<ProjectedCount>,
    pub(in crate::render) explanations: Vec<ProjectedExplanation>,
    pub(in crate::render) unowned_items: Vec<String>,
    pub(in crate::render) duplicate_items: Vec<String>,
    pub(in crate::render) long_exact_prose_assertions: Vec<String>,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedArchitectureTraceability {
    pub(in crate::render) counts: Vec<ProjectedCount>,
    pub(in crate::render) explanations: Vec<ProjectedExplanation>,
    pub(in crate::render) items: Vec<ProjectedMetaLine>,
    pub(in crate::render) unavailable_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedModuleAnalysis {
    pub(in crate::render) counts: Vec<ProjectedCount>,
    pub(in crate::render) meta_lines: Vec<ProjectedMetaLine>,
    pub(in crate::render) explanations: Vec<ProjectedExplanation>,
}

pub(super) fn explanation(label: &'static str, key: MetricExplanationKey) -> ProjectedExplanation {
    let explanation = metric_explanation(key);
    ProjectedExplanation {
        label,
        plain: explanation.plain,
        precise: explanation.precise,
    }
}

pub(super) fn count(label: &'static str, value: usize) -> ProjectedCount {
    ProjectedCount {
        label,
        value: value.to_string(),
    }
}
