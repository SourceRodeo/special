use std::collections::BTreeMap;
/**
@module SPECIAL.MODULES
Coordinates architecture parsing, lint, and projection over discovered source-local and markdown-backed `@module`/`@area` declarations. This subsystem owns the architecture tree and its ownership joins. It may project shared code-analysis evidence onto modules, but repo-wide code analysis must not require module ownership before code becomes visible.
*/
// @fileimplements SPECIAL.MODULES
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::cache::{
    load_or_build_architecture_analysis, load_or_build_bounded_repo_analysis_summary,
    load_or_build_repo_analysis_summary, load_or_build_scoped_repo_analysis_summary,
    load_or_parse_architecture, load_or_parse_repo,
};
use crate::config::{DocsOutputConfig, PatternMetricBenchmarks, SpecialVersion};
use crate::model::{
    ArchitectureAnalysisSummary, ArchitectureKind, ArchitectureMetricsSummary, GroupedCount,
    LintReport, ModuleAnalysisOptions, ModuleDocument, ModuleFilter, ModuleNode,
    ParsedArchitecture, PatternFilter, RepoArchitectureHealthMetrics, RepoDocsHealthMetrics,
    RepoDocument, RepoMetricsSummary, RepoPatternHealthMetrics, RepoSpecHealthMetrics,
    RepoTestHealthMetrics, RepoTraceabilityMetrics,
};

pub(crate) mod analyze;
mod lint;
mod materialize;
mod parse;
mod parse_markdown;

#[derive(Clone, Copy, Debug, Default)]
pub struct RepoDocumentOptions<'a> {
    pub metrics: bool,
    pub health_ignore_unexplained_patterns: &'a [String],
    pub docs_outputs: &'a [DocsOutputConfig],
    pub pattern_benchmarks: PatternMetricBenchmarks,
    pub target_scope_paths: Option<&'a [PathBuf]>,
    pub within_scope_paths: Option<&'a [PathBuf]>,
    pub symbol: Option<&'a str>,
}

pub fn build_module_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    filter: ModuleFilter,
    analysis_options: ModuleAnalysisOptions,
) -> Result<(ModuleDocument, LintReport)> {
    let analysis_options = analysis_options.normalized();
    let parsed = load_or_parse_architecture(root, ignore_patterns)?;
    let lint = lint::build_module_lint_report(&parsed);
    let parsed_repo = analysis_options
        .any()
        .then(|| load_or_parse_repo(root, ignore_patterns, version))
        .transpose()?;
    let analysis = if analysis_options.any() {
        Some(load_or_build_architecture_analysis(
            root,
            ignore_patterns,
            version,
            &parsed,
            parsed_repo.as_ref(),
            analysis_options,
        )?)
    } else {
        None
    };
    let mut document =
        materialize::build_module_document(&parsed, filter, analysis.as_ref().map(|a| &a.modules));
    if analysis_options.metrics {
        document.metrics = Some(build_architecture_metrics(&document.nodes));
    }
    Ok((document, lint))
}

pub(crate) fn build_module_document_from_parsed(
    parsed: &ParsedArchitecture,
    filter: ModuleFilter,
) -> ModuleDocument {
    materialize::build_module_document(parsed, filter, None)
}

pub fn build_repo_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    options: RepoDocumentOptions<'_>,
) -> Result<(RepoDocument, LintReport)> {
    let parsed = load_or_parse_architecture(root, ignore_patterns)?;
    let lint = lint::build_module_lint_report(&parsed);
    let parsed_repo = load_or_parse_repo(root, ignore_patterns, version)?;
    let mut summary = if let Some(within_paths) = options.within_scope_paths {
        load_or_build_bounded_repo_analysis_summary(
            root,
            ignore_patterns,
            version,
            &parsed,
            &parsed_repo,
            within_paths,
        )?
    } else if let Some(scoped_paths) = options.target_scope_paths {
        load_or_build_scoped_repo_analysis_summary(
            root,
            ignore_patterns,
            version,
            &parsed,
            &parsed_repo,
            scoped_paths,
        )?
    } else {
        load_or_build_repo_analysis_summary(root, ignore_patterns, version, &parsed, &parsed_repo)?
    };
    if let Some(target_scope_paths) = options.target_scope_paths
        && Some(target_scope_paths) != options.within_scope_paths
    {
        analyze::filter_repo_analysis_summary_to_paths(
            root,
            ignore_patterns,
            target_scope_paths,
            &mut summary,
        )?;
    }
    if let Some(symbol) = options.symbol {
        analyze::filter_repo_analysis_summary_to_symbol(symbol, &mut summary);
    }
    apply_health_ignore_unexplained(
        root,
        options.health_ignore_unexplained_patterns,
        &mut summary,
    )?;
    apply_pattern_opportunity_signals(root, ignore_patterns, &parsed, options, &mut summary)?;
    analyze::apply_long_prose_outside_docs_summary(
        root,
        ignore_patterns,
        options.docs_outputs,
        health_signal_scope_paths(options),
        summary
            .repo_signals
            .as_mut()
            .expect("repo health should always include repo signals"),
    )?;

    Ok((
        RepoDocument {
            metrics: options
                .metrics
                .then(|| {
                    build_repo_metrics(
                        root,
                        ignore_patterns,
                        version,
                        &summary,
                        options.docs_outputs,
                    )
                })
                .transpose()?,
            analysis: Some(summary),
        },
        lint,
    ))
}

pub(crate) fn build_repo_document_from_parsed(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed: &ParsedArchitecture,
    parsed_repo: &crate::model::ParsedRepo,
    options: RepoDocumentOptions<'_>,
) -> Result<RepoDocument> {
    let mut summary = if let Some(within_paths) = options.within_scope_paths {
        load_or_build_bounded_repo_analysis_summary(
            root,
            ignore_patterns,
            version,
            parsed,
            parsed_repo,
            within_paths,
        )?
    } else if let Some(scoped_paths) = options.target_scope_paths {
        load_or_build_scoped_repo_analysis_summary(
            root,
            ignore_patterns,
            version,
            parsed,
            parsed_repo,
            scoped_paths,
        )?
    } else {
        load_or_build_repo_analysis_summary(root, ignore_patterns, version, parsed, parsed_repo)?
    };
    if let Some(target_scope_paths) = options.target_scope_paths
        && Some(target_scope_paths) != options.within_scope_paths
    {
        analyze::filter_repo_analysis_summary_to_paths(
            root,
            ignore_patterns,
            target_scope_paths,
            &mut summary,
        )?;
    }
    if let Some(symbol) = options.symbol {
        analyze::filter_repo_analysis_summary_to_symbol(symbol, &mut summary);
    }
    apply_health_ignore_unexplained(
        root,
        options.health_ignore_unexplained_patterns,
        &mut summary,
    )?;
    apply_pattern_opportunity_signals(root, ignore_patterns, parsed, options, &mut summary)?;
    analyze::apply_long_prose_outside_docs_summary(
        root,
        ignore_patterns,
        options.docs_outputs,
        health_signal_scope_paths(options),
        summary
            .repo_signals
            .as_mut()
            .expect("repo health should always include repo signals"),
    )?;

    Ok(RepoDocument {
        metrics: options
            .metrics
            .then(|| {
                build_repo_metrics(
                    root,
                    ignore_patterns,
                    version,
                    &summary,
                    options.docs_outputs,
                )
            })
            .transpose()?,
        analysis: Some(summary),
    })
}

fn apply_health_ignore_unexplained(
    root: &Path,
    patterns: &[String],
    summary: &mut ArchitectureAnalysisSummary,
) -> Result<()> {
    if patterns.is_empty() {
        return Ok(());
    }
    let Some(traceability) = summary.traceability.as_mut() else {
        return Ok(());
    };

    let mut retained = Vec::with_capacity(traceability.unexplained_items.len());
    for item in traceability.unexplained_items.drain(..) {
        if !crate::discovery::path_matches_patterns(root, &item.path, patterns)? {
            retained.push(item);
        }
    }
    traceability.unexplained_items = retained;
    Ok(())
}

fn apply_pattern_opportunity_signals(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &ParsedArchitecture,
    options: RepoDocumentOptions<'_>,
    summary: &mut ArchitectureAnalysisSummary,
) -> Result<()> {
    let Some(repo_signals) = summary.repo_signals.as_mut() else {
        return Ok(());
    };
    let filter = PatternFilter {
        scope: None,
        metrics: true,
        target_paths: health_signal_scope_paths(options)
            .map(|paths| paths.to_vec())
            .unwrap_or_default(),
        comparison_paths: options
            .within_scope_paths
            .map(|paths| paths.to_vec())
            .unwrap_or_default(),
        symbol: options.symbol.map(ToString::to_string),
    };
    let candidates = crate::patterns::pattern_metric_candidates(
        root,
        ignore_patterns,
        parsed,
        &filter,
        options.pattern_benchmarks,
    )?;
    repo_signals.possible_missing_pattern_applications =
        candidates.possible_missing_applications.len();
    repo_signals.possible_missing_pattern_application_details =
        candidates.possible_missing_applications;
    repo_signals.possible_pattern_clusters = candidates.possible_pattern_clusters.len();
    repo_signals.possible_pattern_cluster_details = candidates.possible_pattern_clusters;
    Ok(())
}

fn health_signal_scope_paths(options: RepoDocumentOptions<'_>) -> Option<&[PathBuf]> {
    options.target_scope_paths.or_else(|| {
        options
            .within_scope_paths
            .filter(|_| options.target_scope_paths.is_none())
    })
}

fn build_repo_metrics(
    _root: &Path,
    _ignore_patterns: &[String],
    _version: SpecialVersion,
    summary: &crate::model::ArchitectureAnalysisSummary,
    _docs_outputs: &[DocsOutputConfig],
) -> Result<RepoMetricsSummary> {
    let signals = summary.repo_signals.as_ref();
    let duplicate_source_shapes_by_file = signals
        .map(|signals| {
            grouped_counts(
                signals
                    .duplicate_item_details
                    .iter()
                    .map(|item| item.path.display().to_string()),
            )
        })
        .unwrap_or_default();
    let source_outside_architecture_by_file = signals
        .map(|signals| {
            grouped_counts(
                signals
                    .unowned_item_details
                    .iter()
                    .map(|item| item.path.display().to_string()),
            )
        })
        .unwrap_or_default();
    let possible_missing_applications_by_file = signals
        .map(|signals| {
            grouped_counts(
                signals
                    .possible_missing_pattern_application_details
                    .iter()
                    .map(|item| item.location.path.display().to_string()),
            )
        })
        .unwrap_or_default();
    let long_prose_outside_docs_by_file = signals
        .map(|signals| {
            grouped_counts(
                signals
                    .long_prose_outside_docs_details
                    .iter()
                    .map(|item| item.path.display().to_string()),
            )
        })
        .unwrap_or_default();
    let exact_long_prose_assertions_by_file = signals
        .map(|signals| {
            grouped_counts(
                signals
                    .long_exact_prose_assertion_details
                    .iter()
                    .map(|item| item.path.display().to_string()),
            )
        })
        .unwrap_or_default();
    let untraced_implementation_by_file = summary
        .traceability
        .as_ref()
        .map(|traceability| {
            grouped_counts(
                traceability
                    .unexplained_items
                    .iter()
                    .map(|item| item.path.display().to_string()),
            )
        })
        .unwrap_or_default();
    let untraced_review_surface_by_file = summary
        .traceability
        .as_ref()
        .map(|traceability| {
            grouped_counts(
                traceability
                    .unexplained_items
                    .iter()
                    .filter(|item| item.review_surface)
                    .map(|item| item.path.display().to_string()),
            )
        })
        .unwrap_or_default();
    let traceability = summary
        .traceability
        .as_ref()
        .map(|traceability| RepoTraceabilityMetrics {
            analyzed_items: traceability.analyzed_items,
            current_spec_items: traceability.current_spec_items.len(),
            statically_mediated_items: traceability.statically_mediated_items.len(),
            unverified_test_items: traceability.unverified_test_items.len(),
            unexplained_items: traceability.unexplained_items.len(),
            unexplained_review_surface_items: traceability.unexplained_review_surface_items(),
            unexplained_public_items: traceability.unexplained_public_items(),
            unexplained_internal_items: traceability.unexplained_internal_items(),
            unexplained_module_backed_items: traceability.unexplained_module_backed_items(),
            unexplained_module_connected_items: traceability.unexplained_module_connected_items(),
            unexplained_module_isolated_items: traceability.unexplained_module_isolated_items(),
            unexplained_items_by_file: untraced_implementation_by_file.clone(),
            unexplained_review_surface_items_by_file: untraced_review_surface_by_file.clone(),
        });
    let specs = RepoSpecHealthMetrics {
        untraced_implementation: traceability
            .as_ref()
            .map(|metrics| metrics.unexplained_items)
            .unwrap_or_default(),
        test_covered_unlinked_implementation: traceability
            .as_ref()
            .map(|metrics| metrics.unverified_test_items)
            .unwrap_or_default(),
        planned_or_deprecated_only_implementation: summary
            .traceability
            .as_ref()
            .map(|traceability| {
                traceability.planned_only_items.len() + traceability.deprecated_only_items.len()
            })
            .unwrap_or_default(),
        statically_mediated_implementation: traceability
            .as_ref()
            .map(|metrics| metrics.statically_mediated_items)
            .unwrap_or_default(),
        untraced_implementation_by_file,
        untraced_review_surface_by_file,
    };
    let architecture = RepoArchitectureHealthMetrics {
        source_outside_architecture: signals
            .map(|signals| signals.unowned_items)
            .unwrap_or_default(),
        source_outside_architecture_by_file,
    };
    let patterns = RepoPatternHealthMetrics {
        duplicate_source_shapes: signals
            .map(|signals| signals.duplicate_items)
            .unwrap_or_default(),
        possible_pattern_clusters: signals
            .map(|signals| signals.possible_pattern_clusters)
            .unwrap_or_default(),
        possible_missing_applications: signals
            .map(|signals| signals.possible_missing_pattern_applications)
            .unwrap_or_default(),
        duplicate_source_shapes_by_file,
        possible_missing_applications_by_file,
    };
    let docs = RepoDocsHealthMetrics {
        long_prose_outside_docs: signals
            .map(|signals| signals.long_prose_outside_docs)
            .unwrap_or_default(),
        long_prose_outside_docs_by_file,
    };
    let tests = RepoTestHealthMetrics {
        exact_long_prose_assertions: signals
            .map(|signals| signals.long_exact_prose_assertions)
            .unwrap_or_default(),
        exact_long_prose_assertions_by_file,
    };
    Ok(RepoMetricsSummary {
        specs,
        architecture,
        patterns,
        docs,
        tests,
        traceability,
    })
}

fn build_architecture_metrics(nodes: &[ModuleNode]) -> ArchitectureMetricsSummary {
    let mut summary = empty_architecture_metrics_summary();
    let mut groups = ArchitectureMetricGroups::default();

    append_architecture_metrics(nodes, &mut summary, &mut groups);
    groups.finish(&mut summary);

    summary
}

fn append_architecture_metrics(
    nodes: &[ModuleNode],
    summary: &mut ArchitectureMetricsSummary,
    groups: &mut ArchitectureMetricGroups,
) {
    for node in nodes {
        record_node_kind_metrics(node, summary, groups);
        if let Some(analysis) = &node.analysis {
            record_node_analysis_metrics(node, analysis, summary, groups);
        }

        append_architecture_metrics(&node.children, summary, groups);
    }
}

#[derive(Default)]
struct ArchitectureMetricGroups {
    modules_by_area: BTreeMap<String, usize>,
    owned_lines_by_module: BTreeMap<String, usize>,
    max_cyclomatic_by_module: BTreeMap<String, usize>,
    max_cognitive_by_module: BTreeMap<String, usize>,
    panic_sites_by_module: BTreeMap<String, usize>,
    unreached_items_by_module: BTreeMap<String, usize>,
    fan_in_by_module: BTreeMap<String, usize>,
    fan_out_by_module: BTreeMap<String, usize>,
    ambiguous_internal_targets_by_module: BTreeMap<String, usize>,
    unresolved_internal_targets_by_module: BTreeMap<String, usize>,
    external_dependency_targets_by_module: BTreeMap<String, usize>,
}

impl ArchitectureMetricGroups {
    fn finish(self, summary: &mut ArchitectureMetricsSummary) {
        summary.modules_by_area = grouped_count_map(self.modules_by_area);
        summary.owned_lines_by_module = grouped_count_map(self.owned_lines_by_module);
        summary.max_cyclomatic_by_module = grouped_count_map(self.max_cyclomatic_by_module);
        summary.max_cognitive_by_module = grouped_count_map(self.max_cognitive_by_module);
        summary.panic_sites_by_module = grouped_count_map(self.panic_sites_by_module);
        summary.unreached_items_by_module = grouped_count_map(self.unreached_items_by_module);
        summary.fan_in_by_module = grouped_count_map(self.fan_in_by_module);
        summary.fan_out_by_module = grouped_count_map(self.fan_out_by_module);
        summary.ambiguous_internal_targets_by_module =
            grouped_count_map(self.ambiguous_internal_targets_by_module);
        summary.unresolved_internal_targets_by_module =
            grouped_count_map(self.unresolved_internal_targets_by_module);
        summary.external_dependency_targets_by_module =
            grouped_count_map(self.external_dependency_targets_by_module);
    }
}

fn empty_architecture_metrics_summary() -> ArchitectureMetricsSummary {
    ArchitectureMetricsSummary {
        total_modules: 0,
        total_areas: 0,
        unimplemented_modules: 0,
        file_scoped_implements: 0,
        item_scoped_implements: 0,
        owned_lines: 0,
        public_items: 0,
        internal_items: 0,
        complexity_functions: 0,
        total_cyclomatic: 0,
        max_cyclomatic: 0,
        total_cognitive: 0,
        max_cognitive: 0,
        quality_public_functions: 0,
        quality_parameters: 0,
        quality_bool_params: 0,
        quality_raw_string_params: 0,
        quality_panic_sites: 0,
        unreached_items: 0,
        modules_by_area: Vec::new(),
        owned_lines_by_module: Vec::new(),
        max_cyclomatic_by_module: Vec::new(),
        max_cognitive_by_module: Vec::new(),
        panic_sites_by_module: Vec::new(),
        unreached_items_by_module: Vec::new(),
        fan_in_by_module: Vec::new(),
        fan_out_by_module: Vec::new(),
        ambiguous_internal_targets_by_module: Vec::new(),
        unresolved_internal_targets_by_module: Vec::new(),
        external_dependency_targets_by_module: Vec::new(),
    }
}

fn record_node_kind_metrics(
    node: &ModuleNode,
    summary: &mut ArchitectureMetricsSummary,
    groups: &mut ArchitectureMetricGroups,
) {
    match node.kind() {
        ArchitectureKind::Area => {
            summary.total_areas += 1;
            let descendant_modules = count_descendant_modules(&node.children);
            if descendant_modules > 0 {
                groups
                    .modules_by_area
                    .insert(node.id.clone(), descendant_modules);
            }
        }
        ArchitectureKind::Module => {
            summary.total_modules += 1;
            if node.is_unimplemented() {
                summary.unimplemented_modules += 1;
            }
        }
    }
}

fn record_node_analysis_metrics(
    node: &ModuleNode,
    analysis: &crate::model::ModuleAnalysisSummary,
    summary: &mut ArchitectureMetricsSummary,
    groups: &mut ArchitectureMetricGroups,
) {
    record_coverage_metrics(analysis, summary);
    record_owned_item_metrics(node, analysis, summary, groups);
    record_complexity_metrics(node, analysis, summary, groups);
    record_quality_metrics(node, analysis, summary, groups);
    record_item_signal_metrics(node, analysis, summary, groups);
    record_coupling_metrics(node, analysis, groups);
}

fn record_coverage_metrics(
    analysis: &crate::model::ModuleAnalysisSummary,
    summary: &mut ArchitectureMetricsSummary,
) {
    if let Some(coverage) = &analysis.coverage {
        summary.file_scoped_implements += coverage.file_scoped_implements;
        summary.item_scoped_implements += coverage.item_scoped_implements;
    }
}

fn record_owned_item_metrics(
    node: &ModuleNode,
    analysis: &crate::model::ModuleAnalysisSummary,
    summary: &mut ArchitectureMetricsSummary,
    groups: &mut ArchitectureMetricGroups,
) {
    let Some(metrics) = &analysis.metrics else {
        return;
    };
    summary.owned_lines += metrics.owned_lines;
    summary.public_items += metrics.public_items;
    summary.internal_items += metrics.internal_items;
    if node.kind() == ArchitectureKind::Module && metrics.owned_lines > 0 {
        groups
            .owned_lines_by_module
            .insert(node.id.clone(), metrics.owned_lines);
    }
}

fn record_complexity_metrics(
    node: &ModuleNode,
    analysis: &crate::model::ModuleAnalysisSummary,
    summary: &mut ArchitectureMetricsSummary,
    groups: &mut ArchitectureMetricGroups,
) {
    let Some(complexity) = &analysis.complexity else {
        return;
    };
    summary.complexity_functions += complexity.function_count;
    summary.total_cyclomatic += complexity.total_cyclomatic;
    summary.max_cyclomatic = summary.max_cyclomatic.max(complexity.max_cyclomatic);
    summary.total_cognitive += complexity.total_cognitive;
    summary.max_cognitive = summary.max_cognitive.max(complexity.max_cognitive);
    if node.kind() == ArchitectureKind::Module {
        if complexity.max_cyclomatic > 0 {
            groups
                .max_cyclomatic_by_module
                .insert(node.id.clone(), complexity.max_cyclomatic);
        }
        if complexity.max_cognitive > 0 {
            groups
                .max_cognitive_by_module
                .insert(node.id.clone(), complexity.max_cognitive);
        }
    }
}

fn record_quality_metrics(
    node: &ModuleNode,
    analysis: &crate::model::ModuleAnalysisSummary,
    summary: &mut ArchitectureMetricsSummary,
    groups: &mut ArchitectureMetricGroups,
) {
    let Some(quality) = &analysis.quality else {
        return;
    };
    summary.quality_public_functions += quality.public_function_count;
    summary.quality_parameters += quality.parameter_count;
    summary.quality_bool_params += quality.bool_parameter_count;
    summary.quality_raw_string_params += quality.raw_string_parameter_count;
    summary.quality_panic_sites += quality.panic_site_count;
    if node.kind() == ArchitectureKind::Module && quality.panic_site_count > 0 {
        groups
            .panic_sites_by_module
            .insert(node.id.clone(), quality.panic_site_count);
    }
}

fn record_item_signal_metrics(
    node: &ModuleNode,
    analysis: &crate::model::ModuleAnalysisSummary,
    summary: &mut ArchitectureMetricsSummary,
    groups: &mut ArchitectureMetricGroups,
) {
    let Some(item_signals) = &analysis.item_signals else {
        return;
    };
    summary.unreached_items += item_signals.unreached_item_count;
    if node.kind() == ArchitectureKind::Module && item_signals.unreached_item_count > 0 {
        groups
            .unreached_items_by_module
            .insert(node.id.clone(), item_signals.unreached_item_count);
    }
}

fn record_coupling_metrics(
    node: &ModuleNode,
    analysis: &crate::model::ModuleAnalysisSummary,
    groups: &mut ArchitectureMetricGroups,
) {
    let Some(coupling) = &analysis.coupling else {
        return;
    };
    if node.kind() != ArchitectureKind::Module {
        return;
    }
    if coupling.fan_in > 0 {
        groups
            .fan_in_by_module
            .insert(node.id.clone(), coupling.fan_in);
    }
    if coupling.fan_out > 0 {
        groups
            .fan_out_by_module
            .insert(node.id.clone(), coupling.fan_out);
    }
    if coupling.ambiguous_internal_target_count > 0 {
        groups
            .ambiguous_internal_targets_by_module
            .insert(node.id.clone(), coupling.ambiguous_internal_target_count);
    }
    if coupling.unresolved_internal_target_count > 0 {
        groups
            .unresolved_internal_targets_by_module
            .insert(node.id.clone(), coupling.unresolved_internal_target_count);
    }
    if coupling.external_target_count > 0 {
        groups
            .external_dependency_targets_by_module
            .insert(node.id.clone(), coupling.external_target_count);
    }
}

fn count_descendant_modules(nodes: &[ModuleNode]) -> usize {
    nodes
        .iter()
        .map(|node| {
            usize::from(node.kind() == ArchitectureKind::Module)
                + count_descendant_modules(&node.children)
        })
        .sum()
}

fn grouped_counts(values: impl Iterator<Item = String>) -> Vec<GroupedCount> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(value, count)| GroupedCount { value, count })
        .collect()
}

fn grouped_count_map(counts: BTreeMap<String, usize>) -> Vec<GroupedCount> {
    counts
        .into_iter()
        .map(|(value, count)| GroupedCount { value, count })
        .collect()
}

pub fn build_module_lint_report(root: &Path, ignore_patterns: &[String]) -> Result<LintReport> {
    let parsed = parse_architecture(root, ignore_patterns)?;
    Ok(lint::build_module_lint_report(&parsed))
}

pub(crate) fn build_module_lint_report_from_parsed(parsed: &ParsedArchitecture) -> LintReport {
    lint::build_module_lint_report(parsed)
}

pub(crate) fn parse_architecture(
    root: &Path,
    ignore_patterns: &[String],
) -> Result<ParsedArchitecture> {
    parse::parse_architecture(root, ignore_patterns)
}
