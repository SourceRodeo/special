/**
@module SPECIAL.RENDER.HTML.REPO
Renders repo-wide health documents into HTML pages and metric sections.
*/
// @fileimplements SPECIAL.RENDER.HTML.REPO
use crate::model::{RepoDocument, RepoMetricsSummary};
use crate::render::html_common::SPEC_HTML_STYLE;
use crate::render::projection::{
    ProjectedRepoMetricSection, project_repo_health_metric_sections, project_repo_signals_view,
    project_repo_traceability_view,
};

use super::super::templates::render_template;
use super::{
    ExplanationsHtmlTemplate, HtmlCount, RepoPageHtmlTemplate, format_repo_signals_html,
    format_repo_traceability_html, projected_explanation, render_metrics_section_html,
};

pub(in crate::render) fn render_repo_html(document: &RepoDocument, verbose: bool) -> String {
    let document = crate::render::projection::project_repo_document(document, verbose);
    let metrics_html = document
        .metrics
        .as_ref()
        .map(|metrics| format_repo_metrics_html(metrics, verbose))
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
    render_template(&RepoPageHtmlTemplate {
        style: SPEC_HTML_STYLE,
        body_html: metrics_html + &repo_signals_html + &traceability_html,
    })
}

fn format_repo_metrics_html(metrics: &RepoMetricsSummary, verbose: bool) -> String {
    project_repo_health_metric_sections(metrics, verbose)
        .into_iter()
        .map(render_projected_metric_section_html)
        .collect()
}

fn render_projected_metric_section_html(section: ProjectedRepoMetricSection) -> String {
    let counts = section
        .counts
        .into_iter()
        .map(|count| HtmlCount {
            label: count.label,
            value: count.value,
        })
        .collect::<Vec<_>>();
    if section.explanations.is_empty() {
        return render_metrics_section_html(section.title, &counts);
    }
    let counts_html = render_template(&super::CountsSectionHtmlTemplate { counts: &counts });
    let explanations = section
        .explanations
        .iter()
        .map(projected_explanation)
        .collect::<Vec<_>>();
    render_template(&super::MetricsSectionHtmlTemplate {
        title: section.title.to_string(),
        counts_html,
        explanations_html: render_template(&ExplanationsHtmlTemplate {
            explanations: &explanations,
        }),
    })
}
