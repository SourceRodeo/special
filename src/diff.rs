/**
@module SPECIAL.DIFF
Explicit relationship fingerprint inventory for review-oriented drift workflows.
*/
// @fileimplements SPECIAL.DIFF
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::cache::{load_or_parse_architecture, load_or_parse_repo};
use crate::config::SpecialVersion;
use crate::docs::{DocumentRefSource, DocumentTargetKind, build_docs_document};
use crate::model::{ParsedArchitecture, ParsedRepo, SourceLocation};

#[derive(Debug, Clone, Default)]
pub(crate) struct RelationshipDiffOptions {
    pub target_paths: Vec<PathBuf>,
    pub changed_paths: Vec<PathBuf>,
    pub id: Option<String>,
    pub symbol: Option<String>,
    pub include_content: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RelationshipDiffDocument {
    pub summary: RelationshipDiffSummary,
    pub relationships: Vec<RelationshipFingerprint>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RelationshipDiffSummary {
    pub current_relationships: usize,
    pub affected_relationships: usize,
    pub changed_paths: usize,
    pub missing_targets: usize,
    pub source_endpoints: usize,
    pub target_endpoints: usize,
    pub elapsed_ms: u128,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relationship_kinds: Vec<RelationshipMetric>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub target_kinds: Vec<RelationshipMetric>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub top_source_paths: Vec<RelationshipMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RelationshipMetric {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RelationshipFingerprint {
    pub relationship_kind: String,
    pub target_kind: String,
    pub target_id: String,
    pub source: EndpointFingerprint,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<EndpointFingerprint>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub affected_by_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EndpointFingerprint {
    pub path: String,
    pub line: usize,
    pub fingerprint: String,
    pub normalized_bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

pub(crate) fn build_relationship_diff_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    options: RelationshipDiffOptions,
) -> Result<RelationshipDiffDocument> {
    let started = Instant::now();
    let parsed_repo = load_or_parse_repo(root, ignore_patterns, version)?;
    let parsed_architecture = load_or_parse_architecture(root, ignore_patterns)?;
    let (docs_document, _) =
        build_docs_document(root, ignore_patterns, version, &options.target_paths)?;
    let targets = TargetIndex::new(&parsed_repo, &parsed_architecture);
    let mut current_relationships = Vec::new();

    for verify in &parsed_repo.verifies {
        push_relationship(
            root,
            &options,
            &targets,
            &mut current_relationships,
            RelationshipInput {
                relationship_kind: "@verifies",
                target_kind: target_kind_for_spec(&targets, &verify.spec_id),
                target_id: &verify.spec_id,
                source_location: verify.body_location.as_ref().unwrap_or(&verify.location),
                source_text: verify.body.as_deref().unwrap_or(""),
            },
        );
    }

    for attest in &parsed_repo.attests {
        let source_text = format!(
            "{} {} {} {} {}",
            attest.artifact,
            attest.owner,
            attest.last_reviewed,
            attest
                .review_interval_days
                .map(|days| days.to_string())
                .unwrap_or_default(),
            attest.body.as_deref().unwrap_or("")
        );
        push_relationship(
            root,
            &options,
            &targets,
            &mut current_relationships,
            RelationshipInput {
                relationship_kind: attest.scope.as_annotation(),
                target_kind: target_kind_for_spec(&targets, &attest.spec_id),
                target_id: &attest.spec_id,
                source_location: &attest.location,
                source_text: &source_text,
            },
        );
    }

    for implementation in &parsed_architecture.implements {
        push_relationship(
            root,
            &options,
            &targets,
            &mut current_relationships,
            RelationshipInput {
                relationship_kind: "@implements",
                target_kind: target_kind_for_architecture(&targets, &implementation.module_id),
                target_id: &implementation.module_id,
                source_location: implementation
                    .body_location
                    .as_ref()
                    .unwrap_or(&implementation.location),
                source_text: implementation.body.as_deref().unwrap_or(""),
            },
        );
    }

    for application in &parsed_architecture.pattern_applications {
        push_relationship(
            root,
            &options,
            &targets,
            &mut current_relationships,
            RelationshipInput {
                relationship_kind: "@applies",
                target_kind: "pattern",
                target_id: &application.pattern_id,
                source_location: application
                    .body_location
                    .as_ref()
                    .unwrap_or(&application.location),
                source_text: application.body.as_deref().unwrap_or(""),
            },
        );
    }

    for doc_ref in &docs_document.references {
        let relationship_kind = match doc_ref.source {
            DocumentRefSource::Link => "documents://",
            DocumentRefSource::DocumentsLine => "@documents",
            DocumentRefSource::FileDocumentsLine => "@filedocuments",
        };
        push_relationship(
            root,
            &options,
            &targets,
            &mut current_relationships,
            RelationshipInput {
                relationship_kind,
                target_kind: docs_target_kind_label(doc_ref.target_kind),
                target_id: &doc_ref.target_id,
                source_location: &doc_ref.location,
                source_text: doc_ref.text.as_deref().unwrap_or(""),
            },
        );
    }

    current_relationships.sort_by(|left, right| {
        left.source
            .path
            .cmp(&right.source.path)
            .then(left.source.line.cmp(&right.source.line))
            .then(left.relationship_kind.cmp(&right.relationship_kind))
            .then(left.target_kind.cmp(&right.target_kind))
            .then(left.target_id.cmp(&right.target_id))
    });

    let current_relationship_count = current_relationships.len();
    let changed_paths = options
        .changed_paths
        .iter()
        .map(|path| display_path(root, path))
        .collect::<Vec<_>>();
    let relationships = current_relationships
        .into_iter()
        .filter_map(|mut relationship| {
            relationship.affected_by_paths =
                relationship_changed_paths(root, &relationship, &options.changed_paths);
            (!relationship.affected_by_paths.is_empty()).then_some(relationship)
        })
        .collect::<Vec<_>>();

    let source_endpoints = relationships
        .iter()
        .map(|relationship| (&relationship.source.path, relationship.source.line))
        .collect::<HashSet<_>>()
        .len();
    let target_endpoints = relationships
        .iter()
        .filter_map(|relationship| relationship.target.as_ref())
        .map(|target| (&target.path, target.line))
        .collect::<HashSet<_>>()
        .len();
    let missing_targets = relationships
        .iter()
        .filter(|relationship| relationship.target.is_none())
        .count();
    let relationship_kinds = count_by(
        relationships
            .iter()
            .map(|relationship| relationship.relationship_kind.as_str()),
    );
    let target_kinds = count_by(
        relationships
            .iter()
            .map(|relationship| relationship.target_kind.as_str()),
    );
    let top_source_paths = count_by(
        relationships
            .iter()
            .map(|relationship| relationship.source.path.as_str()),
    )
    .into_iter()
    .take(10)
    .collect();
    Ok(RelationshipDiffDocument {
        summary: RelationshipDiffSummary {
            current_relationships: current_relationship_count,
            affected_relationships: relationships.len(),
            changed_paths: changed_paths.len(),
            missing_targets,
            source_endpoints,
            target_endpoints,
            elapsed_ms: started.elapsed().as_millis(),
            relationship_kinds,
            target_kinds,
            top_source_paths,
        },
        relationships,
    })
}

pub(crate) fn render_relationship_diff_json(document: &RelationshipDiffDocument) -> Result<String> {
    Ok(serde_json::to_string_pretty(document)?)
}

pub(crate) fn render_relationship_diff_text(
    document: &RelationshipDiffDocument,
    verbose: bool,
    metrics: bool,
) -> String {
    let summary = &document.summary;
    let mut output = String::new();
    output.push_str("relationship diff\n");
    output.push_str(&format!("  changed paths: {}\n", summary.changed_paths));
    output.push_str(&format!(
        "  affected relationships: {}\n",
        summary.affected_relationships
    ));
    output.push_str(&format!(
        "  current relationships: {}\n",
        summary.current_relationships
    ));
    output.push_str(&format!("  missing targets: {}\n", summary.missing_targets));
    output.push_str(&format!("  elapsed: {} ms\n", summary.elapsed_ms));
    if metrics {
        output.push_str("  relationship kinds:\n");
        append_metrics(&mut output, &summary.relationship_kinds, 4);
        output.push_str("  target kinds:\n");
        append_metrics(&mut output, &summary.target_kinds, 4);
        output.push_str("  top source paths:\n");
        append_metrics(&mut output, &summary.top_source_paths, 4);
    }
    for relationship in document
        .relationships
        .iter()
        .take(if verbose { usize::MAX } else { 20 })
    {
        output.push_str(&format!(
            "  {} {} {} at {}:{}",
            relationship.relationship_kind,
            relationship.target_kind,
            relationship.target_id,
            relationship.source.path,
            relationship.source.line
        ));
        if relationship.target.is_none() {
            output.push_str(" [missing target]");
        }
        if !relationship.affected_by_paths.is_empty() {
            output.push_str(&format!(
                " [affected by {}]",
                relationship.affected_by_paths.join(", ")
            ));
        }
        output.push('\n');
        if verbose {
            append_endpoint_content(&mut output, "source", &relationship.source);
            if let Some(target) = &relationship.target {
                append_endpoint_content(&mut output, "target", target);
            }
        }
    }
    if !verbose && document.relationships.len() > 20 {
        output.push_str(&format!(
            "  ... {} more relationship(s)\n",
            document.relationships.len() - 20
        ));
    }
    output.trim_end().to_string()
}

struct RelationshipInput<'a> {
    relationship_kind: &'static str,
    target_kind: &'static str,
    target_id: &'a str,
    source_location: &'a SourceLocation,
    source_text: &'a str,
}

fn push_relationship(
    root: &Path,
    options: &RelationshipDiffOptions,
    targets: &TargetIndex<'_>,
    relationships: &mut Vec<RelationshipFingerprint>,
    input: RelationshipInput<'_>,
) {
    let target = targets.get(input.target_kind, input.target_id);
    if !matches_scope(
        root,
        options,
        input.target_id,
        input.source_location,
        input.source_text,
        target,
    ) {
        return;
    }

    relationships.push(RelationshipFingerprint {
        relationship_kind: input.relationship_kind.to_string(),
        target_kind: input.target_kind.to_string(),
        target_id: input.target_id.to_string(),
        source: endpoint_fingerprint(
            root,
            input.source_location,
            input.source_text,
            options.include_content,
        ),
        target: target.map(|target| {
            endpoint_fingerprint(root, target.location, target.text, options.include_content)
        }),
        affected_by_paths: Vec::new(),
    });
}

fn matches_scope(
    root: &Path,
    options: &RelationshipDiffOptions,
    target_id: &str,
    source_location: &SourceLocation,
    source_text: &str,
    target: Option<TargetEndpoint<'_>>,
) -> bool {
    if let Some(id) = &options.id {
        if target_id != id && !target_id.starts_with(&format!("{id}.")) {
            return false;
        }
    }
    if let Some(symbol) = &options.symbol {
        if !source_text.contains(symbol) && !target_id.contains(symbol) {
            return false;
        }
    }
    if options.target_paths.is_empty() {
        return true;
    }
    path_in_scope(root, &source_location.path, &options.target_paths)
        || target
            .map(|target| path_in_scope(root, &target.location.path, &options.target_paths))
            .unwrap_or(false)
}

fn path_in_scope(root: &Path, path: &Path, scopes: &[PathBuf]) -> bool {
    let candidate_path = absolute_path(root, path);
    scopes.iter().any(|scope| {
        let absolute_scope = absolute_path(root, scope);
        candidate_path == absolute_scope || candidate_path.starts_with(&absolute_scope)
    })
}

fn relationship_changed_paths(
    root: &Path,
    relationship: &RelationshipFingerprint,
    changed_paths: &[PathBuf],
) -> Vec<String> {
    changed_paths
        .iter()
        .filter(|changed_path| {
            let changed_display = display_path(root, changed_path);
            endpoint_path_matches(&relationship.source.path, &changed_display)
                || relationship
                    .target
                    .as_ref()
                    .map(|target| endpoint_path_matches(&target.path, &changed_display))
                    .unwrap_or(false)
        })
        .map(|path| display_path(root, path))
        .collect()
}

fn endpoint_path_matches(endpoint_path: &str, changed_path: &str) -> bool {
    endpoint_path == changed_path
        || endpoint_path.starts_with(&format!("{changed_path}/"))
        || changed_path.starts_with(&format!("{endpoint_path}/"))
}

fn count_by<'a>(values: impl Iterator<Item = &'a str>) -> Vec<RelationshipMetric> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        *counts.entry(value.to_string()).or_default() += 1;
    }
    let mut metrics = counts
        .into_iter()
        .map(|(name, count)| RelationshipMetric { name, count })
        .collect::<Vec<_>>();
    metrics.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.name.cmp(&right.name))
    });
    metrics
}

fn append_metrics(output: &mut String, metrics: &[RelationshipMetric], indent: usize) {
    if metrics.is_empty() {
        output.push_str(&format!("{}none\n", " ".repeat(indent)));
        return;
    }
    for metric in metrics {
        output.push_str(&format!(
            "{}{}: {}\n",
            " ".repeat(indent),
            metric.name,
            metric.count
        ));
    }
}

fn append_endpoint_content(output: &mut String, label: &str, endpoint: &EndpointFingerprint) {
    let Some(content) = endpoint.content.as_deref() else {
        return;
    };
    output.push_str(&format!(
        "    {label} {}:{} [{} bytes]\n",
        endpoint.path, endpoint.line, endpoint.normalized_bytes
    ));
    if content.is_empty() {
        output.push_str("      <empty>\n");
    } else {
        output.push_str(&format!("      {content}\n"));
    }
}

fn endpoint_fingerprint(
    root: &Path,
    location: &SourceLocation,
    text: &str,
    include_content: bool,
) -> EndpointFingerprint {
    let normalized = normalize_text(text);
    EndpointFingerprint {
        path: display_path(root, &location.path),
        line: location.line,
        fingerprint: stable_fingerprint(&normalized),
        normalized_bytes: normalized.len(),
        content: include_content.then_some(normalized),
    }
}

fn stable_fingerprint(text: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn display_path(root: &Path, path: &Path) -> String {
    let absolute = absolute_path(root, path);
    absolute
        .strip_prefix(root)
        .unwrap_or(&absolute)
        .to_string_lossy()
        .to_string()
}

fn absolute_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

#[derive(Clone, Copy)]
struct TargetEndpoint<'a> {
    location: &'a SourceLocation,
    text: &'a str,
}

struct TargetIndex<'a> {
    specs: BTreeMap<&'a str, TargetEndpoint<'a>>,
    groups: BTreeMap<&'a str, TargetEndpoint<'a>>,
    modules: BTreeMap<&'a str, TargetEndpoint<'a>>,
    areas: BTreeMap<&'a str, TargetEndpoint<'a>>,
    patterns: BTreeMap<&'a str, TargetEndpoint<'a>>,
}

impl<'a> TargetIndex<'a> {
    fn new(parsed_repo: &'a ParsedRepo, parsed_architecture: &'a ParsedArchitecture) -> Self {
        let mut specs = BTreeMap::new();
        let mut groups = BTreeMap::new();
        for spec in &parsed_repo.specs {
            let endpoint = TargetEndpoint {
                location: &spec.location,
                text: &spec.text,
            };
            match spec.kind() {
                crate::model::NodeKind::Spec => {
                    specs.insert(spec.id.as_str(), endpoint);
                }
                crate::model::NodeKind::Group => {
                    groups.insert(spec.id.as_str(), endpoint);
                }
            }
        }

        let mut modules = BTreeMap::new();
        let mut areas = BTreeMap::new();
        for module in &parsed_architecture.modules {
            let endpoint = TargetEndpoint {
                location: &module.location,
                text: &module.text,
            };
            match module.kind() {
                crate::model::ArchitectureKind::Module => {
                    modules.insert(module.id.as_str(), endpoint);
                }
                crate::model::ArchitectureKind::Area => {
                    areas.insert(module.id.as_str(), endpoint);
                }
            }
        }

        let patterns = parsed_architecture
            .patterns
            .iter()
            .map(|pattern| {
                (
                    pattern.pattern_id.as_str(),
                    TargetEndpoint {
                        location: &pattern.location,
                        text: &pattern.text,
                    },
                )
            })
            .collect();

        Self {
            specs,
            groups,
            modules,
            areas,
            patterns,
        }
    }

    fn get(&self, kind: &str, id: &str) -> Option<TargetEndpoint<'a>> {
        match kind {
            "spec" => self.specs.get(id).copied(),
            "group" => self.groups.get(id).copied(),
            "module" => self.modules.get(id).copied(),
            "area" => self.areas.get(id).copied(),
            "pattern" => self.patterns.get(id).copied(),
            _ => None,
        }
    }
}

fn target_kind_for_spec(targets: &TargetIndex<'_>, id: &str) -> &'static str {
    if targets.groups.contains_key(id) {
        "group"
    } else {
        "spec"
    }
}

fn target_kind_for_architecture(targets: &TargetIndex<'_>, id: &str) -> &'static str {
    if targets.areas.contains_key(id) {
        "area"
    } else {
        "module"
    }
}

fn docs_target_kind_label(kind: DocumentTargetKind) -> &'static str {
    match kind {
        DocumentTargetKind::Spec => "spec",
        DocumentTargetKind::Group => "group",
        DocumentTargetKind::Module => "module",
        DocumentTargetKind::Area => "area",
        DocumentTargetKind::Pattern => "pattern",
    }
}
