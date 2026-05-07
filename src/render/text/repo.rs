/**
@module SPECIAL.RENDER.TEXT.REPO
Renders repo-wide health and traceability views into human-readable text output.
*/
// @fileimplements SPECIAL.RENDER.TEXT.REPO
use crate::model::RepoDocument;
use crate::render::projection::{
    project_repo_document, project_repo_signals_view, project_repo_traceability_view,
};

use super::analysis::{format_repo_signal_details, format_repo_signals, format_repo_traceability};
use super::render_repo_metrics_text;

pub(in crate::render) fn render_repo_text(document: &RepoDocument, verbose: bool) -> String {
    let document = project_repo_document(document, verbose);
    let mut output = String::from("special health\n");
    if let Some(metrics) = &document.metrics {
        output.push_str(&render_repo_metrics_text(metrics));
    }
    if let Some(repo_signals) = document
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.repo_signals.as_ref())
    {
        let projected = project_repo_signals_view(repo_signals, verbose);
        if document.metrics.is_some() {
            output.push_str(&format_repo_signal_details(&projected));
        } else {
            output.push_str(&format_repo_signals(&projected));
        }
    }
    if let Some(analysis) = document.analysis.as_ref() {
        output.push_str(&format_repo_traceability(&project_repo_traceability_view(
            analysis.traceability.as_ref(),
            analysis.traceability_unavailable_reason.as_deref(),
        )));
    }
    output
}
