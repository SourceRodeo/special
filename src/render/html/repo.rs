/**
@module SPECIAL.RENDER.HTML.REPO
Renders repo-wide health documents into HTML pages and metric sections.
*/
// @fileimplements SPECIAL.RENDER.HTML.REPO
use crate::model::{RepoDocument, RepoMetricsSummary};
use crate::render::html_common::SPEC_HTML_STYLE;
use crate::render::projection::{project_repo_signals_view, project_repo_traceability_view};

use super::{
    HtmlCount, format_repo_signals_html, format_repo_traceability_html,
    format_repo_traceability_metrics_html, render_grouped_metrics_section_html,
    render_metrics_section_html,
};

pub(in crate::render) fn render_repo_html(document: &RepoDocument, verbose: bool) -> String {
    let document = crate::render::projection::project_repo_document(document, verbose);
    let metrics_html = document
        .metrics
        .as_ref()
        .map(format_repo_metrics_html)
        .unwrap_or_default();
    let repo_signals_html = document
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.repo_signals.as_ref())
        .map(|signals| format_repo_signals_html(&project_repo_signals_view(signals, verbose)))
        .unwrap_or_default();
    let traceability_html = document
        .analysis
        .as_ref()
        .map(|analysis| {
            format_repo_traceability_html(&project_repo_traceability_view(
                analysis.traceability.as_ref(),
                analysis.traceability_unavailable_reason.as_deref(),
            ))
        })
        .unwrap_or_default();
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>special health</title><style>{}</style></head><body><main><h1>special health</h1><p class=\"lede\">Repo-wide quality signals for the current repository.</p>{}{}</main></body></html>",
        SPEC_HTML_STYLE,
        metrics_html + &repo_signals_html,
        traceability_html
    )
}

fn format_repo_metrics_html(metrics: &RepoMetricsSummary) -> String {
    let mut html = render_metrics_section_html(
        "health summary",
        &[
            HtmlCount {
                label: "raw investigation queues",
                value: metrics.global.raw_investigation_queues.to_string(),
            },
            HtmlCount {
                label: "source outside architecture",
                value: metrics.architecture.source_outside_architecture.to_string(),
            },
            HtmlCount {
                label: "untraced implementation",
                value: metrics.specs.untraced_implementation.to_string(),
            },
            HtmlCount {
                label: "duplicate source shapes",
                value: metrics.patterns.duplicate_source_shapes.to_string(),
            },
            HtmlCount {
                label: "possible pattern clusters",
                value: metrics.patterns.possible_pattern_clusters.to_string(),
            },
            HtmlCount {
                label: "possible missing pattern applications",
                value: metrics.patterns.possible_missing_applications.to_string(),
            },
            HtmlCount {
                label: "long prose outside docs",
                value: metrics.docs.long_prose_outside_docs.to_string(),
            },
            HtmlCount {
                label: "exact long-prose test assertions",
                value: metrics.tests.exact_long_prose_assertions.to_string(),
            },
        ],
    );
    html.push_str(&render_grouped_metrics_section_html(
        "source outside architecture by file",
        &metrics.architecture.source_outside_architecture_by_file,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "untraced implementation by file",
        &metrics.specs.untraced_implementation_by_file,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "duplicate source shapes by file",
        &metrics.patterns.duplicate_source_shapes_by_file,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "possible missing pattern applications by file",
        &metrics.patterns.possible_missing_applications_by_file,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "long prose outside docs by file",
        &metrics.docs.long_prose_outside_docs_by_file,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "exact long-prose test assertions by file",
        &metrics.tests.exact_long_prose_assertions_by_file,
    ));
    if let Some(traceability) = &metrics.traceability {
        html.push_str(&format_repo_traceability_metrics_html(traceability));
    }
    html
}
