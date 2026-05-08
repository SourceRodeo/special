/**
@module SPECIAL.RENDER.HTML
Renders projected specs and modules into HTML views with shared styling and best-effort code highlighting.
*/
// @fileimplements SPECIAL.RENDER.HTML
use askama::Template;

use crate::model::{GroupedCount, SpecMetricsSummary};

use super::html_common::escape_html;
pub(super) use super::labels::{attest_label, implementation_label, verify_label};
use super::projection::{
    ProjectedArchitectureTraceability, ProjectedCount, ProjectedExplanation, ProjectedMetaLine,
    ProjectedRepoSignals,
};
use super::templates::render_template;

#[path = "html/module.rs"]
mod module;
#[path = "html/repo.rs"]
mod repo;
#[path = "html/spec.rs"]
mod spec;

pub(super) use self::module::render_module_html;
pub(super) use self::repo::render_repo_html;
pub(super) use self::spec::render_spec_html;

#[derive(Clone)]
pub(super) struct HtmlCount {
    pub(super) label: String,
    pub(super) value: String,
}

#[derive(Clone)]
pub(super) struct HtmlDetailSection {
    pub(super) label: &'static str,
    pub(super) location: String,
    pub(super) body_at: Option<String>,
    pub(super) body_html: Option<String>,
    pub(super) language_class: String,
}

#[derive(Clone)]
pub(super) struct HtmlMetaLine {
    pub(super) label: String,
    pub(super) value: String,
}

#[derive(Clone)]
pub(super) struct HtmlExplanationSection {
    pub(super) label: &'static str,
    pub(super) plain: String,
    pub(super) precise: String,
}

#[derive(Template)]
#[template(path = "render/counts_section.html")]
pub(super) struct CountsSectionHtmlTemplate<'a> {
    pub(super) counts: &'a [HtmlCount],
}

#[derive(Template)]
#[template(path = "render/metrics_section.html")]
pub(super) struct MetricsSectionHtmlTemplate {
    pub(super) title: String,
    pub(super) counts_html: String,
    pub(super) explanations_html: String,
}

#[derive(Template)]
#[template(path = "render/grouped_metrics_section.html")]
pub(super) struct GroupedMetricsSectionHtmlTemplate {
    pub(super) title: String,
    pub(super) lines_html: String,
}

#[derive(Template)]
#[template(path = "render/detail_sections.html")]
pub(super) struct DetailSectionsHtmlTemplate<'a> {
    pub(super) details: &'a [HtmlDetailSection],
}

#[derive(Template)]
#[template(path = "render/meta_lines.html")]
pub(super) struct MetaLinesHtmlTemplate<'a> {
    pub(super) lines: &'a [HtmlMetaLine],
}

#[derive(Template)]
#[template(path = "render/explanations.html")]
pub(super) struct ExplanationsHtmlTemplate<'a> {
    pub(super) explanations: &'a [HtmlExplanationSection],
}

#[derive(Template)]
#[template(path = "render/spec_verbose.html")]
pub(super) struct SpecVerboseHtmlTemplate {
    pub(super) declared_at: String,
    pub(super) verifies_html: String,
    pub(super) attests_html: String,
}

#[derive(Template)]
#[template(path = "render/module_verbose.html")]
pub(super) struct ModuleVerboseHtmlTemplate {
    pub(super) implementations_html: String,
    pub(super) meta_lines_html: String,
    pub(super) explanations_html: String,
}

#[derive(Template)]
#[template(path = "render/repo_page.html")]
pub(super) struct RepoPageHtmlTemplate {
    pub(super) style: &'static str,
    pub(super) body_html: String,
}

#[derive(Template)]
#[template(path = "render/coverage_section.html")]
pub(super) struct CoverageSectionHtmlTemplate<'a> {
    pub(super) counts_html: String,
    pub(super) explanations_html: String,
    pub(super) verbose: bool,
    pub(super) unowned_items: &'a [String],
    pub(super) duplicate_items: &'a [String],
    pub(super) possible_missing_pattern_applications: &'a [String],
    pub(super) possible_pattern_clusters: &'a [String],
    pub(super) long_prose_outside_docs: &'a [String],
    pub(super) long_exact_prose_assertions: &'a [String],
}

pub(super) fn format_spec_metrics_html(metrics: &SpecMetricsSummary) -> String {
    let mut html = render_metrics_section_html(
        "special specs metrics",
        &[
            HtmlCount {
                label: "total specs".to_string(),
                value: metrics.total_specs.to_string(),
            },
            HtmlCount {
                label: "unverified specs".to_string(),
                value: metrics.unverified_specs.to_string(),
            },
            HtmlCount {
                label: "planned specs".to_string(),
                value: metrics.planned_specs.to_string(),
            },
            HtmlCount {
                label: "deprecated specs".to_string(),
                value: metrics.deprecated_specs.to_string(),
            },
            HtmlCount {
                label: "verifies".to_string(),
                value: metrics.verifies.to_string(),
            },
            HtmlCount {
                label: "attests".to_string(),
                value: metrics.attests.to_string(),
            },
        ],
    );
    html.push_str(&render_metrics_section_html(
        "spec support buckets",
        &[
            HtmlCount {
                label: "verified specs".to_string(),
                value: metrics.verified_specs.to_string(),
            },
            HtmlCount {
                label: "attested specs".to_string(),
                value: metrics.attested_specs.to_string(),
            },
            HtmlCount {
                label: "specs with both supports".to_string(),
                value: metrics.specs_with_both_supports.to_string(),
            },
            HtmlCount {
                label: "item-scoped verifies".to_string(),
                value: metrics.item_scoped_verifies.to_string(),
            },
            HtmlCount {
                label: "file-scoped verifies".to_string(),
                value: metrics.file_scoped_verifies.to_string(),
            },
            HtmlCount {
                label: "unattached verifies".to_string(),
                value: metrics.unattached_verifies.to_string(),
            },
            HtmlCount {
                label: "block attests".to_string(),
                value: metrics.block_attests.to_string(),
            },
            HtmlCount {
                label: "file attests".to_string(),
                value: metrics.file_attests.to_string(),
            },
        ],
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "specs by file",
        &metrics.specs_by_file,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "current specs by top-level id",
        &metrics.current_specs_by_top_level_id,
    ));
    html
}

pub(super) fn render_metrics_section_html(title: &str, counts: &[HtmlCount]) -> String {
    let counts_html = render_template(&CountsSectionHtmlTemplate { counts });
    render_template(&MetricsSectionHtmlTemplate {
        title: title.to_string(),
        counts_html,
        explanations_html: String::new(),
    })
}

pub(super) fn render_grouped_metrics_section_html(title: &str, counts: &[GroupedCount]) -> String {
    if counts.is_empty() {
        return String::new();
    }
    let lines = counts
        .iter()
        .map(|group| HtmlMetaLine {
            label: group.value.clone(),
            value: group.count.to_string(),
        })
        .collect::<Vec<_>>();
    let lines_html = render_template(&MetaLinesHtmlTemplate { lines: &lines });
    render_template(&GroupedMetricsSectionHtmlTemplate {
        title: title.to_string(),
        lines_html,
    })
}

pub(super) fn format_repo_signals_html(coverage: &ProjectedRepoSignals) -> String {
    if coverage.counts.is_empty()
        && coverage.unowned_items.is_empty()
        && coverage.duplicate_items.is_empty()
        && coverage.possible_missing_pattern_applications.is_empty()
        && coverage.possible_pattern_clusters.is_empty()
        && coverage.long_prose_outside_docs.is_empty()
        && coverage.long_exact_prose_assertions.is_empty()
    {
        return String::new();
    }

    render_template(&CoverageSectionHtmlTemplate {
        counts_html: render_template(&CountsSectionHtmlTemplate {
            counts: &coverage
                .counts
                .iter()
                .map(projected_count)
                .collect::<Vec<_>>(),
        }),
        explanations_html: render_template(&ExplanationsHtmlTemplate {
            explanations: &coverage
                .explanations
                .iter()
                .map(projected_explanation)
                .collect::<Vec<_>>(),
        }),
        verbose: !coverage.unowned_items.is_empty()
            || !coverage.duplicate_items.is_empty()
            || !coverage.possible_missing_pattern_applications.is_empty()
            || !coverage.possible_pattern_clusters.is_empty()
            || !coverage.long_prose_outside_docs.is_empty()
            || !coverage.long_exact_prose_assertions.is_empty(),
        unowned_items: &coverage.unowned_items,
        duplicate_items: &coverage.duplicate_items,
        possible_missing_pattern_applications: &coverage.possible_missing_pattern_applications,
        possible_pattern_clusters: &coverage.possible_pattern_clusters,
        long_prose_outside_docs: &coverage.long_prose_outside_docs,
        long_exact_prose_assertions: &coverage.long_exact_prose_assertions,
    })
}

pub(super) fn format_repo_traceability_html(
    traceability: &ProjectedArchitectureTraceability,
) -> String {
    if traceability.counts.is_empty()
        && traceability.items.is_empty()
        && traceability.explanations.is_empty()
        && traceability.unavailable_reason.is_none()
    {
        return String::new();
    }

    let counts_html = render_template(&CountsSectionHtmlTemplate {
        counts: &traceability
            .counts
            .iter()
            .map(projected_count)
            .collect::<Vec<_>>(),
    });
    let explanations_html = render_template(&ExplanationsHtmlTemplate {
        explanations: &traceability
            .explanations
            .iter()
            .map(projected_explanation)
            .collect::<Vec<_>>(),
    });
    let details_html = traceability
        .items
        .iter()
        .map(|item| {
            format!(
                "<li><strong>{}</strong>: {}</li>",
                item.label,
                escape_html(&item.value)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let unavailable_html = traceability
        .unavailable_reason
        .as_ref()
        .map(|reason| {
            format!(
                "<p><strong>unavailable</strong>: {}</p>",
                escape_html(reason)
            )
        })
        .unwrap_or_default();
    format!(
        "<section class=\"coverage\"><h2>traceability</h2>{unavailable_html}{counts_html}{explanations_html}<details><summary>traceability detail</summary><ul>{details_html}</ul></details></section>"
    )
}

pub(super) fn projected_count(count: &ProjectedCount) -> HtmlCount {
    HtmlCount {
        label: count.label.to_string(),
        value: count.value.clone(),
    }
}

pub(super) fn projected_meta_line(line: &ProjectedMetaLine) -> HtmlMetaLine {
    HtmlMetaLine {
        label: line.label.to_string(),
        value: line.value.clone(),
    }
}

pub(super) fn projected_explanation(explanation: &ProjectedExplanation) -> HtmlExplanationSection {
    HtmlExplanationSection {
        label: explanation.label,
        plain: explanation.plain.to_string(),
        precise: explanation.precise.to_string(),
    }
}
