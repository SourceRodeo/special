/**
@module SPECIAL.INDEX
Coordinates dialect selection, lint assembly, and spec-tree materialization over parsed repo annotations.

@spec SPECIAL.PARSE.MULTI_FILE_TREE
special builds one spec tree from declarations spread across multiple files.

@spec SPECIAL.PARSE.MULTI_FILE_TREE.MIXED_FILE_TYPES
special builds one spec tree from declarations spread across supported file types.

@spec SPECIAL.GROUPS
special supports structural group nodes that organize claims without making direct claims of their own.

@spec SPECIAL.GROUPS.STRUCTURAL_ONLY
special treats @group as structure-only and does not require verifies, attests, or planned markers on group nodes.

@spec SPECIAL.GROUPS.SPEC_MAY_HAVE_CHILDREN
special allows @spec nodes to have children while still remaining direct claims that need their own verifies, attests, or planned marker.

@spec SPECIAL.GROUPS.MUTUALLY_EXCLUSIVE
special does not allow the same node id to be declared as both @spec and @group.
*/
// @fileimplements SPECIAL.INDEX
use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

use crate::cache::load_or_parse_repo;
use crate::config::SpecialVersion;
use crate::model::{
    AttestScope, GroupedCount, LintReport, ParsedRepo, SpecDocument, SpecFilter, SpecMetricsSummary,
};

mod lint;
mod materialize;

use self::lint::lint_from_parsed;
use self::materialize::materialize_spec;

pub fn build_spec_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    filter: SpecFilter,
    metrics: bool,
) -> Result<(SpecDocument, LintReport)> {
    let parsed = load_or_parse_repo(root, ignore_patterns, version)?;
    let lint = lint_from_parsed(&parsed);
    let document = materialize_spec(&parsed, filter, metrics, Some(root));
    Ok((document, lint))
}

pub fn build_lint_report(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
) -> Result<LintReport> {
    let parsed = load_or_parse_repo(root, ignore_patterns, version)?;
    Ok(lint_from_parsed(&parsed))
}

pub(crate) fn build_spec_document_from_parsed(
    parsed: &ParsedRepo,
    filter: SpecFilter,
    metrics: bool,
) -> SpecDocument {
    materialize_spec(parsed, filter, metrics, None)
}

pub(crate) fn build_lint_report_from_parsed(parsed: &ParsedRepo) -> LintReport {
    lint_from_parsed(parsed)
}

pub(crate) fn build_spec_metrics(
    root: Option<&Path>,
    nodes: &[crate::model::SpecNode],
) -> SpecMetricsSummary {
    let spec_nodes = collect_spec_nodes(nodes);
    SpecMetricsSummary {
        total_specs: spec_nodes.len(),
        planned_specs: spec_nodes.iter().filter(|node| node.is_planned()).count(),
        deprecated_specs: spec_nodes
            .iter()
            .filter(|node| node.is_deprecated())
            .count(),
        unverified_specs: spec_nodes
            .iter()
            .filter(|node| node.is_unverified())
            .count(),
        verified_specs: spec_nodes
            .iter()
            .filter(|node| !node.verifies.is_empty())
            .count(),
        attested_specs: spec_nodes
            .iter()
            .filter(|node| !node.attests.is_empty())
            .count(),
        specs_with_both_supports: spec_nodes
            .iter()
            .filter(|node| !node.verifies.is_empty() && !node.attests.is_empty())
            .count(),
        verifies: spec_nodes.iter().map(|node| node.verifies.len()).sum(),
        item_scoped_verifies: spec_nodes
            .iter()
            .flat_map(|node| node.verifies.iter())
            .filter(|verify| verify.body_location.is_some())
            .count(),
        file_scoped_verifies: spec_nodes
            .iter()
            .flat_map(|node| node.verifies.iter())
            .filter(|verify| verify.body_location.is_none() && verify.body.is_some())
            .count(),
        unattached_verifies: spec_nodes
            .iter()
            .flat_map(|node| node.verifies.iter())
            .filter(|verify| verify.body_location.is_none() && verify.body.is_none())
            .count(),
        attests: spec_nodes.iter().map(|node| node.attests.len()).sum(),
        block_attests: spec_nodes
            .iter()
            .flat_map(|node| node.attests.iter())
            .filter(|attest| attest.scope == AttestScope::Block)
            .count(),
        file_attests: spec_nodes
            .iter()
            .flat_map(|node| node.attests.iter())
            .filter(|attest| attest.scope == AttestScope::File)
            .count(),
        specs_by_file: grouped_counts(
            spec_nodes
                .iter()
                .map(|node| relative_path_display(root, &node.location.path)),
        ),
        current_specs_by_top_level_id: grouped_counts(
            spec_nodes
                .iter()
                .filter(|node| !node.is_planned() && !node.is_deprecated())
                .map(|node| top_level_id(&node.id)),
        ),
    }
}

fn relative_path_display(root: Option<&Path>, path: &Path) -> String {
    root.and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn collect_spec_nodes(nodes: &[crate::model::SpecNode]) -> Vec<&crate::model::SpecNode> {
    let mut collected = Vec::new();
    append_spec_nodes(nodes, &mut collected);
    collected
}

fn append_spec_nodes<'a>(
    nodes: &'a [crate::model::SpecNode],
    collected: &mut Vec<&'a crate::model::SpecNode>,
) {
    nodes.iter().for_each(|node| {
        if node.kind() == crate::model::NodeKind::Spec {
            collected.push(node);
        }
        append_spec_nodes(&node.children, collected);
    });
}

fn top_level_id(id: &str) -> String {
    id.split('.').next().unwrap_or(id).to_string()
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
#[cfg(test)]
mod tests;
