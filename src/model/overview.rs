/**
@module SPECIAL.MODEL.OVERVIEW
Rendered document, filter, metrics, and lint report domain types.
*/
// @fileimplements SPECIAL.MODEL.OVERVIEW
use serde::Serialize;
use std::collections::BTreeMap;

use super::{ArchitectureAnalysisSummary, Diagnostic, DiagnosticSeverity, ModuleNode, SpecNode};

#[derive(Debug, Clone, Serialize)]
pub struct SpecDocument {
    pub nodes: Vec<SpecNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<SpecMetricsSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<ArchitectureMetricsSummary>,
    #[serde(skip)]
    pub scoped: bool,
    pub nodes: Vec<ModuleNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<RepoMetricsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<ArchitectureAnalysisSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchitectureMetricsSummary {
    pub total_modules: usize,
    pub total_areas: usize,
    pub unimplemented_modules: usize,
    pub file_scoped_implements: usize,
    pub item_scoped_implements: usize,
    pub owned_lines: usize,
    pub public_items: usize,
    pub internal_items: usize,
    pub complexity_functions: usize,
    pub total_cyclomatic: usize,
    pub max_cyclomatic: usize,
    pub total_cognitive: usize,
    pub max_cognitive: usize,
    pub quality_public_functions: usize,
    pub quality_parameters: usize,
    pub quality_bool_params: usize,
    pub quality_raw_string_params: usize,
    pub quality_panic_sites: usize,
    pub unreached_items: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modules_by_area: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub owned_lines_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub max_cyclomatic_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub max_cognitive_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub panic_sites_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unreached_items_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fan_in_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fan_out_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ambiguous_internal_targets_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unresolved_internal_targets_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub external_dependency_targets_by_module: Vec<GroupedCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecMetricsSummary {
    pub total_specs: usize,
    pub planned_specs: usize,
    pub deprecated_specs: usize,
    pub unverified_specs: usize,
    pub verified_specs: usize,
    pub attested_specs: usize,
    pub specs_with_both_supports: usize,
    pub verifies: usize,
    pub item_scoped_verifies: usize,
    pub file_scoped_verifies: usize,
    pub unattached_verifies: usize,
    pub attests: usize,
    pub block_attests: usize,
    pub file_attests: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub specs_by_file: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub current_specs_by_top_level_id: Vec<GroupedCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoMetricsSummary {
    pub specs: RepoSpecHealthMetrics,
    pub architecture: RepoArchitectureHealthMetrics,
    pub patterns: RepoPatternHealthMetrics,
    pub docs: RepoDocsHealthMetrics,
    pub tests: RepoTestHealthMetrics,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceability: Option<RepoTraceabilityMetrics>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoSpecHealthMetrics {
    pub untraced_implementation: usize,
    pub test_covered_unlinked_implementation: usize,
    pub planned_or_deprecated_only_implementation: usize,
    pub statically_mediated_implementation: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub untraced_implementation_by_file: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub untraced_review_surface_by_file: Vec<GroupedCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoArchitectureHealthMetrics {
    pub source_outside_architecture: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_outside_architecture_by_file: Vec<GroupedCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoPatternHealthMetrics {
    pub duplicate_source_shapes: usize,
    pub possible_pattern_clusters: usize,
    pub possible_missing_applications: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub duplicate_source_shapes_by_file: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub possible_missing_applications_by_file: Vec<GroupedCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoDocsHealthMetrics {
    pub long_prose_outside_docs: usize,
    pub undocumented_current_specs: usize,
    pub undocumented_modules: usize,
    pub undocumented_patterns: usize,
    pub internal_only_documented_targets: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub long_prose_outside_docs_by_file: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub undocumented_current_spec_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub undocumented_module_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub undocumented_pattern_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub internal_only_documented_target_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoTestHealthMetrics {
    pub exact_long_prose_assertions: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exact_long_prose_assertions_by_file: Vec<GroupedCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupedCount {
    pub value: String,
    pub count: usize,
}

pub fn grouped_counts(values: impl Iterator<Item = String>) -> Vec<GroupedCount> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    grouped_count_map(counts)
}

pub fn grouped_count_map(counts: BTreeMap<String, usize>) -> Vec<GroupedCount> {
    counts
        .into_iter()
        .map(|(value, count)| GroupedCount { value, count })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoTraceabilityMetrics {
    pub analyzed_items: usize,
    pub current_spec_items: usize,
    pub statically_mediated_items: usize,
    pub unverified_test_items: usize,
    pub unexplained_items: usize,
    pub unexplained_review_surface_items: usize,
    pub unexplained_public_items: usize,
    pub unexplained_internal_items: usize,
    pub unexplained_module_backed_items: usize,
    pub unexplained_module_connected_items: usize,
    pub unexplained_module_isolated_items: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unexplained_items_by_file: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unexplained_review_surface_items_by_file: Vec<GroupedCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LintReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl LintReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}
