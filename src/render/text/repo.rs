/**
@module SPECIAL.RENDER.TEXT.REPO
Renders repo-wide health and traceability views into human-readable text output.
*/
// @fileimplements SPECIAL.RENDER.TEXT.REPO
use std::collections::BTreeMap;
use std::fmt::Write;

use crate::model::{ArchitectureAnalysisSummary, RepoDocument};
use crate::render::projection::{
    project_repo_document, project_repo_signals_view, project_repo_traceability_view,
};

use super::analysis::{format_repo_signal_details, format_repo_signals, format_repo_traceability};
use super::render_repo_metrics_text;

pub(in crate::render) fn render_repo_text(document: &RepoDocument, verbose: bool) -> String {
    let cleanup_targets = (!verbose && document.metrics.is_some())
        .then(|| render_cleanup_targets(document.analysis.as_ref()))
        .unwrap_or_default();
    let document = project_repo_document(document, verbose);
    let mut output = String::from("special health\n");
    if let Some(metrics) = &document.metrics {
        output.push_str(&render_repo_metrics_text(metrics, verbose));
    }
    output.push_str(&cleanup_targets);
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

fn render_cleanup_targets(analysis: Option<&ArchitectureAnalysisSummary>) -> String {
    let Some(analysis) = analysis else {
        return String::new();
    };
    let mut sections = Vec::new();
    if let Some(traceability) = &analysis.traceability {
        let review_surface = traceability
            .unexplained_items
            .iter()
            .filter(|item| item.review_surface)
            .collect::<Vec<_>>();
        if !review_surface.is_empty() {
            sections.push(cleanup_target_section(
                "untraced review-surface implementation",
                "move product behavior behind ordinary module facades with direct spec-backed tests, or mark generated/script plumbing outside the review path.",
                top_items_by_file(&review_surface, |item| {
                    (item.path.display().to_string(), item.name.clone())
                }),
            ));
        }
    }
    if let Some(signals) = &analysis.repo_signals {
        if !signals.unowned_item_details.is_empty() {
            sections.push(cleanup_target_section(
                "source outside architecture",
                "add module ownership for real implementation boundaries, or exclude generated and fixture-heavy paths deliberately.",
                top_items_by_file(&signals.unowned_item_details, |item| {
                    (item.path.display().to_string(), item.name.clone())
                }),
            ));
        }
        if !signals.duplicate_item_details.is_empty() {
            sections.push(cleanup_target_section(
                "duplicate source shapes",
                "decide whether each cluster wants a helper extraction, an adopted pattern, or no action.",
                top_items_by_file(&signals.duplicate_item_details, |item| {
                    (
                        item.path.display().to_string(),
                        format!("{} ({})", item.name, item.module_id),
                    )
                }),
            ));
        }
        if !signals.possible_pattern_cluster_details.is_empty() {
            let lines = signals
                .possible_pattern_cluster_details
                .iter()
                .take(5)
                .map(|cluster| {
                    let representative = cluster
                        .items
                        .first()
                        .map(|item| {
                            format!(
                                "{} at {}:{}",
                                item.item_name,
                                item.location.path.display(),
                                item.location.line
                            )
                        })
                        .unwrap_or_else(|| "no representative item".to_string());
                    format!(
                        "{} item(s), score {:.3}, suggested strictness {}, {}; {}",
                        cluster.item_count,
                        cluster.score,
                        cluster.suggested_strictness.as_str(),
                        cluster.interpretation.label(),
                        representative,
                    )
                })
                .collect();
            sections.push(cleanup_target_section(
                "possible pattern clusters",
                "inspect the representative cluster before defining a pattern; leave accidental or one-off similarity unnamed.",
                lines,
            ));
        }
        if !signals.long_prose_outside_docs_details.is_empty() {
            sections.push(cleanup_target_section(
                "uncaptured prose outside docs",
                "move reader-facing prose into docs source or add docs evidence when the prose is a durable repo fact.",
                top_items_by_file(&signals.long_prose_outside_docs_details, |item| {
                    (
                        item.path.display().to_string(),
                        format!("line {} ({} words)", item.line, item.word_count),
                    )
                }),
            ));
        }
        if !signals.long_exact_prose_assertion_details.is_empty() {
            sections.push(cleanup_target_section(
                "long prose test literals",
                "replace inline prose fixtures with structured assertions or named fixtures when exact prose is not the product contract.",
                top_items_by_file(&signals.long_exact_prose_assertion_details, |item| {
                    (
                        item.path.display().to_string(),
                        format!(
                            "line {} ({} {}, {} words)",
                            item.line, item.language, item.callee, item.word_count
                        ),
                    )
                }),
            ));
        }
    }
    let sections = sections
        .into_iter()
        .filter(|section| !section.lines.is_empty())
        .collect::<Vec<_>>();
    if sections.is_empty() {
        return String::new();
    }
    let mut output = String::new();
    writeln!(output, "cleanup targets").expect("string writes should succeed");
    for section in sections {
        writeln!(output, "  {}", section.title).expect("string writes should succeed");
        writeln!(output, "    next: {}", section.next).expect("string writes should succeed");
        for line in section.lines {
            writeln!(output, "    {line}").expect("string writes should succeed");
        }
    }
    output
}

struct CleanupTargetSection {
    title: &'static str,
    next: &'static str,
    lines: Vec<String>,
}

fn cleanup_target_section(
    title: &'static str,
    next: &'static str,
    lines: Vec<String>,
) -> CleanupTargetSection {
    CleanupTargetSection { title, next, lines }
}

fn top_items_by_file<T>(
    items: impl IntoIterator<Item = T>,
    item_summary: impl Fn(T) -> (String, String),
) -> Vec<String> {
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for item in items {
        let (path, label) = item_summary(item);
        grouped.entry(path).or_default().push(label);
    }
    top_group_lines(grouped)
}

fn top_group_lines(grouped: BTreeMap<String, Vec<String>>) -> Vec<String> {
    let mut groups = grouped.into_iter().collect::<Vec<_>>();
    groups.sort_by(|(left_path, left_items), (right_path, right_items)| {
        right_items
            .len()
            .cmp(&left_items.len())
            .then_with(|| left_path.cmp(right_path))
    });
    groups
        .into_iter()
        .take(5)
        .map(|(path, mut items)| {
            items.sort();
            let total = items.len();
            let shown = items.into_iter().take(4).collect::<Vec<_>>();
            let suffix = total
                .checked_sub(shown.len())
                .filter(|remaining| *remaining > 0)
                .map(|remaining| format!(" (+{remaining} more)"))
                .unwrap_or_default();
            format!("{path}: {}{}", shown.join(", "), suffix)
        })
        .collect()
}
