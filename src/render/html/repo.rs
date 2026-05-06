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
        "special health metrics",
        &[
            HtmlCount {
                label: "duplicate items",
                value: metrics.duplicate_items.to_string(),
            },
            HtmlCount {
                label: "unowned items",
                value: metrics.unowned_items.to_string(),
            },
        ],
    );
    html.push_str(&render_grouped_metrics_section_html(
        "duplicate items by file",
        &metrics.duplicate_items_by_file,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "unowned items by file",
        &metrics.unowned_items_by_file,
    ));
    if let Some(documentation) = &metrics.documentation {
        let documented = documentation
            .target_kinds
            .iter()
            .map(|kind| kind.documented)
            .sum::<usize>();
        let undocumented = documentation
            .target_kinds
            .iter()
            .map(|kind| kind.undocumented)
            .sum::<usize>();
        html.push_str(&render_metrics_section_html(
            "documentation coverage",
            &[
                HtmlCount {
                    label: "documented targets",
                    value: documented.to_string(),
                },
                HtmlCount {
                    label: "undocumented targets",
                    value: undocumented.to_string(),
                },
            ],
        ));
    }
    if let Some(traceability) = &metrics.traceability {
        html.push_str(&format_repo_traceability_metrics_html(traceability));
    }
    html
}
