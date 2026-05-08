/**
@module SPECIAL.RENDER.TEXT
Renders projected specs, modules, repo health, and lint diagnostics into human-readable text output while centralizing shared text metric helpers.
*/
// @fileimplements SPECIAL.RENDER.TEXT
use crate::model::{
    ArchitectureMetricsSummary, GroupedCount, PatternDocument, PatternNode, RepoMetricsSummary,
    SpecMetricsSummary,
};

mod analysis;
mod attachments;
#[path = "text/lint.rs"]
mod lint;
#[path = "text/module.rs"]
mod module;
#[path = "text/repo.rs"]
mod repo;
#[path = "text/spec.rs"]
mod spec;

pub(super) use self::lint::render_lint_text;
pub(super) use self::module::render_module_text;
pub(super) use self::repo::render_repo_text;
pub(super) use self::spec::render_spec_text;

pub(super) fn render_pattern_text(document: &PatternDocument, verbose: bool) -> String {
    let mut output = String::new();
    if let Some(metrics) = &document.metrics {
        output.push_str("special patterns metrics\n");
        output.push_str(&format!("  total patterns: {}\n", metrics.total_patterns));
        output.push_str(&format!(
            "  total definitions: {}\n",
            metrics.total_definitions
        ));
        output.push_str(&format!(
            "  total applications: {}\n",
            metrics.total_applications
        ));
        output.push_str(&format!(
            "  modules with applications: {}\n",
            metrics.modules_with_applications
        ));
    }

    for pattern in &document.patterns {
        append_pattern_node_text(&mut output, pattern, 0, verbose);
    }

    if output.is_empty() {
        "No patterns found.".to_string()
    } else {
        output.trim_end().to_string()
    }
}

fn append_pattern_node_text(
    output: &mut String,
    pattern: &PatternNode,
    depth: usize,
    verbose: bool,
) {
    let indent = "  ".repeat(depth);
    let detail_indent = "  ".repeat(depth + 1);
    output.push_str(&format!("{indent}{}\n", pattern.id));
    output.push_str(&format!(
        "{detail_indent}definition: {}\n",
        if pattern.definition.is_some() {
            "present"
        } else {
            "missing"
        }
    ));
    if let Some(definition) = &pattern.definition {
        output.push_str(&format!(
            "{detail_indent}strictness: {}\n",
            definition.strictness.as_str()
        ));
    }
    if verbose && let Some(definition) = &pattern.definition {
        output.push_str(&format!(
            "{detail_indent}  {}:{}\n",
            definition.location.path.display(),
            definition.location.line
        ));
        if !definition.text.is_empty() {
            output.push_str(&format!("{detail_indent}    {}\n", definition.text));
        }
    }
    if let Some(metrics) = &pattern.metrics {
        output.push_str(&format!(
            "{detail_indent}similarity: {} scored application(s), {} pair(s)\n",
            metrics.scored_applications, metrics.pair_count
        ));
        if let Some(mean) = metrics.mean_similarity {
            output.push_str(&format!("{detail_indent}  mean: {mean:.3}\n"));
        }
        if let (Some(min), Some(max)) = (metrics.min_similarity, metrics.max_similarity) {
            output.push_str(&format!("{detail_indent}  range: {min:.3}-{max:.3}\n"));
        }
        if let Some(expected) = metrics.expected_similarity {
            output.push_str(&format!("{detail_indent}  expected: {expected:.3}\n"));
        }
        if let Some(benchmark) = metrics.benchmark_estimate {
            output.push_str(&format!(
                "{detail_indent}  benchmark estimate: {}\n",
                benchmark.label()
            ));
        }
    }
    output.push_str(&format!(
        "{detail_indent}applications: {}\n",
        pattern.applications.len()
    ));
    for application in &pattern.applications {
        let owner = application
            .module_id
            .as_deref()
            .map(|id| format!("{id} at "))
            .unwrap_or_default();
        output.push_str(&format!(
            "{detail_indent}  {}{}:{}\n",
            owner,
            application.location.path.display(),
            application.location.line
        ));
        if verbose && let Some(body) = &application.body {
            for line in body.lines() {
                output.push_str(&format!("{detail_indent}    {line}\n"));
            }
        }
    }
    output.push_str(&format!(
        "{detail_indent}modules: {}\n",
        pattern.modules.len()
    ));
    for module in &pattern.modules {
        output.push_str(&format!(
            "{detail_indent}  {} at {}:{}\n",
            module.id,
            module.location.path.display(),
            module.location.line
        ));
    }
    for child in &pattern.children {
        append_pattern_node_text(output, child, depth + 1, verbose);
    }
}

pub(super) fn render_spec_metrics_text(metrics: &SpecMetricsSummary) -> String {
    let mut output = String::from("special specs metrics\n");
    output.push_str(&format!("  total specs: {}\n", metrics.total_specs));
    output.push_str(&format!(
        "  unverified specs: {}\n",
        metrics.unverified_specs
    ));
    output.push_str(&format!("  planned specs: {}\n", metrics.planned_specs));
    output.push_str(&format!(
        "  deprecated specs: {}\n",
        metrics.deprecated_specs
    ));
    output.push_str(&format!("  verified specs: {}\n", metrics.verified_specs));
    output.push_str(&format!("  attested specs: {}\n", metrics.attested_specs));
    output.push_str(&format!(
        "  specs with both supports: {}\n",
        metrics.specs_with_both_supports
    ));
    output.push_str(&format!("  verifies: {}\n", metrics.verifies));
    output.push_str(&format!(
        "    item-scoped verifies: {}\n",
        metrics.item_scoped_verifies
    ));
    output.push_str(&format!(
        "    file-scoped verifies: {}\n",
        metrics.file_scoped_verifies
    ));
    output.push_str(&format!(
        "    unattached verifies: {}\n",
        metrics.unattached_verifies
    ));
    output.push_str(&format!("  attests: {}\n", metrics.attests));
    output.push_str(&format!("    block attests: {}\n", metrics.block_attests));
    output.push_str(&format!("    file attests: {}\n", metrics.file_attests));
    append_grouped_counts_text(&mut output, "specs by file", &metrics.specs_by_file);
    append_grouped_counts_text(
        &mut output,
        "current specs by top-level id",
        &metrics.current_specs_by_top_level_id,
    );
    output
}

pub(super) fn render_repo_metrics_text(metrics: &RepoMetricsSummary) -> String {
    let mut output = String::new();
    for section in crate::render::projection::project_repo_health_metric_sections(metrics) {
        output.push_str(section.title);
        output.push('\n');
        for count in section.counts {
            output.push_str(&format!("  {}: {}\n", count.label, count.value));
        }
    }
    output
}

pub(super) fn render_arch_metrics_text(metrics: &ArchitectureMetricsSummary) -> String {
    let mut output = String::from("special arch metrics\n");
    output.push_str(&format!("  total modules: {}\n", metrics.total_modules));
    output.push_str(&format!("  total areas: {}\n", metrics.total_areas));
    output.push_str(&format!(
        "  unimplemented modules: {}\n",
        metrics.unimplemented_modules
    ));
    output.push_str(&format!(
        "  file-scoped implements: {}\n",
        metrics.file_scoped_implements
    ));
    output.push_str(&format!(
        "  item-scoped implements: {}\n",
        metrics.item_scoped_implements
    ));
    output.push_str(&format!("  owned lines: {}\n", metrics.owned_lines));
    output.push_str(&format!("  public items: {}\n", metrics.public_items));
    output.push_str(&format!("  internal items: {}\n", metrics.internal_items));
    output.push_str(&format!(
        "  complexity functions: {}\n",
        metrics.complexity_functions
    ));
    output.push_str(&format!(
        "  total cyclomatic: {}\n",
        metrics.total_cyclomatic
    ));
    output.push_str(&format!("  max cyclomatic: {}\n", metrics.max_cyclomatic));
    output.push_str(&format!("  total cognitive: {}\n", metrics.total_cognitive));
    output.push_str(&format!("  max cognitive: {}\n", metrics.max_cognitive));
    output.push_str(&format!(
        "  quality public functions: {}\n",
        metrics.quality_public_functions
    ));
    output.push_str(&format!(
        "  quality parameters: {}\n",
        metrics.quality_parameters
    ));
    output.push_str(&format!(
        "  quality bool params: {}\n",
        metrics.quality_bool_params
    ));
    output.push_str(&format!(
        "  quality raw string params: {}\n",
        metrics.quality_raw_string_params
    ));
    output.push_str(&format!(
        "  quality panic sites: {}\n",
        metrics.quality_panic_sites
    ));
    output.push_str(&format!("  unreached items: {}\n", metrics.unreached_items));
    append_grouped_counts_text(&mut output, "modules by area", &metrics.modules_by_area);
    append_grouped_counts_text(
        &mut output,
        "owned lines by module",
        &metrics.owned_lines_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "max cyclomatic by module",
        &metrics.max_cyclomatic_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "max cognitive by module",
        &metrics.max_cognitive_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "panic sites by module",
        &metrics.panic_sites_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "unreached items by module",
        &metrics.unreached_items_by_module,
    );
    append_grouped_counts_text(&mut output, "fan in by module", &metrics.fan_in_by_module);
    append_grouped_counts_text(&mut output, "fan out by module", &metrics.fan_out_by_module);
    append_grouped_counts_text(
        &mut output,
        "ambiguous internal dependency targets by module",
        &metrics.ambiguous_internal_targets_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "unresolved internal dependency targets by module",
        &metrics.unresolved_internal_targets_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "external dependency targets by module",
        &metrics.external_dependency_targets_by_module,
    );
    output
}

fn append_grouped_counts_text(output: &mut String, label: &str, counts: &[GroupedCount]) {
    if counts.is_empty() {
        return;
    }
    output.push_str(&format!("  {label}\n"));
    counts.iter().for_each(|group| {
        output.push_str(&format!("    {}: {}\n", group.value, group.count));
    });
}
