/**
@module SPECIAL.DOCS
Validates documentation-to-Special relationships and renders generated markdown from docs sources.

@group SPECIAL.DOCS
Documentation relationship and docs-output behavior.

@spec SPECIAL.DOCS.LINKS
Markdown links whose destination is `documents://KIND/ID` attach the linked label text as documentation evidence for the targeted Special identifier.

@spec SPECIAL.DOCS.LINKS.POLYMORPHIC
Documentation links accept `spec`, `group`, `module`, `area`, and `pattern` targets.

@spec SPECIAL.DOCS.LINKS.OUTPUT
special docs build SOURCE OUTPUT rewrites markdown `documents://KIND/ID` links to their label text in the emitted artifact.

@spec SPECIAL.DOCS.DOCUMENTS_LINES
Documentation relationship lines `@documents KIND ID` and `@filedocuments KIND ID` attach one documentation relationship per line, are removed from docs output, and may not appear as adjacent stacked relationship lines.

@spec SPECIAL.DOCS_COMMAND.OUTPUT.AUTHORING_LINES
special docs build removes markdown architecture and pattern attachment lines from generated docs output while preserving literal examples.

@spec SPECIAL.DOCS_COMMAND
special docs validates documentation links and prints a documentation relationship view without writing files.

@spec SPECIAL.DOCS_COMMAND.TARGET
special docs --target PATH validates and prints only documentation relationships under the target file or subtree without writing files.

@spec SPECIAL.DOCS_COMMAND.PATH_SCOPE_SYNTAX
special docs follows the shared explicit path-scope syntax for validation-only path scopes.

@spec SPECIAL.DOCS_COMMAND.OUTPUT
special docs build validates documentation links and writes generated docs outputs.

@spec SPECIAL.DOCS_COMMAND.OUTPUT.DIRECTORY
special docs build SOURCE OUTPUT accepts an input directory and output directory, then mirrors the input tree relative to the source root while writing markdown output files.

@spec SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY
special docs build SOURCE OUTPUT refuses to write docs output over the input path, into an input directory, or over an existing file that still contains docs evidence.

@spec SPECIAL.DOCS_COMMAND.OUTPUT.CONFIG
special docs build uses `[[docs.outputs]]` mappings from special.toml to write configured generated docs outputs without repeating paths on the command line.

@spec SPECIAL.DOCS_COMMAND.METRICS
special docs --metrics reports documentation relationship inventory, target coverage, and generated docs graph metrics without writing files.

@spec SPECIAL.DOCS_COMMAND.METRICS.RELATIONSHIPS
special docs --metrics counts documentation relationship inventory by target kind, source shape, and generated/internal source placement without claiming cross-surface documentation coverage.

@spec SPECIAL.DOCS_COMMAND.METRICS.COVERAGE
special docs --metrics reports which specs, groups, modules, areas, and patterns have documentation evidence from generated docs, internal docs, or no docs evidence.

@spec SPECIAL.DOCS_COMMAND.METRICS.INTERCONNECTIVITY
special docs --metrics reports configured generated docs pages, markdown links among those pages, broken local docs links, orphan pages, and configured-entrypoint reachability.

@spec SPECIAL.DOCS_COMMAND.METRICS.TARGET_AUDIT
special docs --metrics --verbose reports documented target support, including planned specs, current specs without verifies or attests, current modules without implementations, and patterns without applications.

@spec SPECIAL.DOCS_COMMAND.METRICS.COVERAGE.DOCS_SOURCE_DECLARATIONS
special docs --metrics target coverage excludes module, area, and pattern targets that are declared, implemented, or applied by configured docs output source paths.
*/
// @fileimplements SPECIAL.DOCS
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

use crate::annotation_syntax::{
    ReservedSpecialAnnotation, normalize_markdown_annotation_line,
    normalize_markdown_declaration_line, reserved_special_annotation_rest,
};
use crate::cache::{load_or_parse_architecture, load_or_parse_repo};
use crate::config::{DocsOutputConfig, SpecialVersion};
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::extractor::collect_comment_blocks;
use crate::model::{
    ArchitectureKind, Diagnostic, DiagnosticSeverity, DocumentationCoverageSummary,
    DocumentationTargetCoverage, LintReport, NodeKind, SourceLocation,
};
use crate::parser::starts_markdown_fence;
use crate::source_paths::matches_scope_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DocumentTargetKind {
    Spec,
    Group,
    Module,
    Area,
    Pattern,
}

impl DocumentTargetKind {
    fn all() -> [Self; 5] {
        [
            Self::Spec,
            Self::Group,
            Self::Module,
            Self::Area,
            Self::Pattern,
        ]
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "spec" => Some(Self::Spec),
            "group" => Some(Self::Group),
            "module" => Some(Self::Module),
            "area" => Some(Self::Area),
            "pattern" => Some(Self::Pattern),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Spec => "spec",
            Self::Group => "group",
            Self::Module => "module",
            Self::Area => "area",
            Self::Pattern => "pattern",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DocumentRefSource {
    Link,
    DocumentsLine,
    FileDocumentsLine,
}

impl DocumentRefSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Link => "link",
            Self::DocumentsLine => "@documents",
            Self::FileDocumentsLine => "@filedocuments",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocumentRef {
    pub target_kind: DocumentTargetKind,
    pub target_id: String,
    pub location: SourceLocation,
    pub source: DocumentRefSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsDocument {
    pub references: Vec<DocumentRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsMetricsDocument {
    pub metrics: DocsMetricsSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsMetricsSummary {
    pub total_references: usize,
    pub link_references: usize,
    pub documents_line_references: usize,
    pub file_documents_line_references: usize,
    pub generated_pages: usize,
    pub local_doc_links: usize,
    pub broken_local_doc_links: usize,
    pub orphan_pages: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reachable_pages_from_entrypoints: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint_pages: Option<usize>,
    pub target_kinds: Vec<DocsTargetKindMetrics>,
    pub coverage: DocumentationCoverageSummary,
    pub broken_local_doc_link_details: Vec<DocsLocalLinkIssue>,
    pub orphan_page_paths: Vec<String>,
    pub target_audit: Vec<DocsTargetAudit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsTargetKindMetrics {
    pub kind: DocumentTargetKind,
    pub references: usize,
    pub link_references: usize,
    pub documents_line_references: usize,
    pub file_documents_line_references: usize,
    pub documented_targets: usize,
    pub generated: usize,
    pub internal_only: usize,
}

impl DocsTargetKindMetrics {
    fn plural_label(&self) -> &'static str {
        match self.kind {
            DocumentTargetKind::Spec => "specs",
            DocumentTargetKind::Group => "groups",
            DocumentTargetKind::Module => "modules",
            DocumentTargetKind::Area => "areas",
            DocumentTargetKind::Pattern => "patterns",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsLocalLinkIssue {
    pub source: String,
    pub line: usize,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsTargetAudit {
    pub kind: DocumentTargetKind,
    pub id: String,
    pub references: usize,
    pub generated_references: usize,
    pub internal_references: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    pub support: DocsTargetSupport,
    pub issues: Vec<String>,
    pub reference_locations: Vec<DocsTargetAuditReference>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct DocsTargetSupport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifies: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attests: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implements: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applications: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strictness: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocsTargetAuditReference {
    pub source: String,
    pub line: usize,
    pub reference_source: DocumentRefSource,
    pub generated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

pub(crate) fn build_docs_lint_report(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
) -> Result<LintReport> {
    let (_, report) = build_docs_document(root, ignore_patterns, version, &[])?;
    Ok(report)
}

pub(crate) fn build_docs_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    scope_paths: &[PathBuf],
) -> Result<(DocsDocument, LintReport)> {
    let (refs, mut diagnostics) = collect_repo_document_refs(root, ignore_patterns)?;
    let refs = scoped_refs(refs, scope_paths);
    retain_scoped_diagnostics(&mut diagnostics, scope_paths);
    diagnostics.extend(validate_document_refs(
        root,
        ignore_patterns,
        version,
        refs.clone(),
    )?);
    sort_diagnostics(&mut diagnostics);
    Ok((
        DocsDocument { references: refs },
        LintReport { diagnostics },
    ))
}

pub(crate) fn build_docs_metrics_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    scope_paths: &[PathBuf],
    outputs: &[DocsOutputConfig],
    entrypoints: &[PathBuf],
) -> Result<(DocsMetricsDocument, LintReport)> {
    let (document, report) = build_docs_document(root, ignore_patterns, version, scope_paths)?;
    let generated_sources = configured_output_sources(root, outputs);
    let generated_graph = build_generated_docs_graph(root, outputs, entrypoints)?;
    let targets = DocumentTargets::load(root, ignore_patterns, version)?;
    let metrics = docs_metrics_summary(
        root,
        &document,
        &targets,
        &generated_sources,
        generated_graph,
    );
    Ok((DocsMetricsDocument { metrics }, report))
}

pub(crate) fn write_docs_path(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    input: &Path,
    output: &Path,
) -> Result<LintReport> {
    write_docs_paths(
        root,
        ignore_patterns,
        version,
        &[(input.to_path_buf(), output.to_path_buf())],
    )
}

pub(crate) fn write_docs_paths(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    mappings: &[(PathBuf, PathBuf)],
) -> Result<LintReport> {
    let mut plan = Vec::new();
    for (input, output) in mappings {
        expand_output_mapping(input, output, &mut plan)?;
    }
    validate_output_plan(&plan)?;
    write_docs_files(root, ignore_patterns, version, &plan)
}

fn expand_output_mapping(
    input: &Path,
    output: &Path,
    plan: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<()> {
    ensure_distinct_paths(
        input,
        output,
        "docs output path must not equal the input path",
    )?;
    if input.is_dir() {
        if path_is_inside(output, input) {
            bail!("docs output directory must not be inside the input directory");
        }
        for entry in WalkBuilder::new(input).hidden(false).build() {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type().is_some_and(|kind| kind.is_file()) {
                continue;
            }
            let relative = path
                .strip_prefix(input)
                .with_context(|| format!("building relative path for {}", path.display()))?;
            let output_path = output.join(relative);
            if path_is_inside(&output_path, input) {
                bail!("docs output file must not be inside the input directory");
            }
            plan.push((path.to_path_buf(), output_path));
        }
    } else {
        plan.push((input.to_path_buf(), output.to_path_buf()));
    }
    Ok(())
}

fn write_docs_files(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    files: &[(PathBuf, PathBuf)],
) -> Result<LintReport> {
    let mut outputs = Vec::new();
    let mut refs = Vec::new();
    let mut diagnostics = Vec::new();

    for (input, output) in files {
        let content =
            fs::read_to_string(input).with_context(|| format!("reading {}", input.display()))?;
        if is_markdown_path(input) {
            let output_content =
                write_markdown_output(input, &content, &mut refs, &mut diagnostics);
            outputs.push((output.clone(), output_content.into_bytes()));
        } else {
            outputs.push((output.clone(), content.into_bytes()));
        }
    }

    diagnostics.extend(validate_document_refs(
        root,
        ignore_patterns,
        version,
        refs,
    )?);
    sort_diagnostics(&mut diagnostics);
    let report = LintReport { diagnostics };
    if report.has_errors() {
        return Ok(report);
    }

    for (output, content) in outputs {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating output directory {}", parent.display()))?;
        }
        fs::write(&output, content).with_context(|| format!("writing {}", output.display()))?;
    }

    Ok(report)
}

pub(crate) fn render_docs_text(root: &Path, document: &DocsDocument) -> String {
    if document.references.is_empty() {
        return "No docs relationships found.".to_string();
    }

    let mut output = String::new();
    output.push_str("Docs\n\n");

    let mut grouped: BTreeMap<(DocumentTargetKind, String), Vec<&DocumentRef>> = BTreeMap::new();
    for reference in &document.references {
        grouped
            .entry((reference.target_kind, reference.target_id.clone()))
            .or_default()
            .push(reference);
    }

    for ((kind, id), references) in grouped {
        output.push_str(&format!("{} {}\n", kind.as_str(), id));
        for reference in references {
            output.push_str(&format!(
                "  {}:{} {}",
                display_path(root, &reference.location.path),
                reference.location.line,
                reference.source.as_str()
            ));
            if let Some(text) = reference.text.as_deref() {
                output.push_str(&format!(": {}", text.trim()));
            }
            output.push('\n');
        }
        output.push('\n');
    }

    output.trim_end().to_string()
}

pub(crate) fn render_docs_json(document: &DocsDocument) -> Result<String> {
    Ok(serde_json::to_string_pretty(document)?)
}

pub(crate) fn render_docs_metrics_text(document: &DocsMetricsDocument, verbose: bool) -> String {
    let metrics = &document.metrics;
    let mut output = String::from("special docs metrics\n");
    output.push_str("  relationship inventory\n");
    output.push_str(&format!(
        "    total references: {}\n",
        metrics.total_references
    ));
    output.push_str(&format!(
        "      link references: {}\n",
        metrics.link_references
    ));
    output.push_str(&format!(
        "      @documents references: {}\n",
        metrics.documents_line_references
    ));
    output.push_str(&format!(
        "      @filedocuments references: {}\n",
        metrics.file_documents_line_references
    ));
    for kind in &metrics.target_kinds {
        output.push_str(&format!(
            "    {}: {} reference(s), {} referenced target(s), {} generated, {} internal-only\n",
            kind.plural_label(),
            kind.references,
            kind.documented_targets,
            kind.generated,
            kind.internal_only
        ));
        if verbose {
            output.push_str(&format!(
                "      sources: {} link, {} @documents, {} @filedocuments\n",
                kind.link_references,
                kind.documents_line_references,
                kind.file_documents_line_references
            ));
        }
    }
    output.push_str("  target coverage\n");
    for kind in &metrics.coverage.target_kinds {
        output.push_str(&format!(
            "    {}s: {} total, {} documented, {} generated, {} internal-only, {} undocumented\n",
            kind.kind,
            kind.total,
            kind.documented,
            kind.generated,
            kind.internal_only,
            kind.undocumented
        ));
    }
    output.push_str("  generated docs graph\n");
    output.push_str(&format!(
        "    generated pages: {}\n",
        metrics.generated_pages
    ));
    output.push_str(&format!(
        "    local doc links: {}\n",
        metrics.local_doc_links
    ));
    output.push_str(&format!(
        "    broken local doc links: {}\n",
        metrics.broken_local_doc_links
    ));
    for issue in metrics
        .broken_local_doc_link_details
        .iter()
        .take(if verbose { usize::MAX } else { 10 })
    {
        output.push_str(&format!(
            "    {}:{} -> {}\n",
            issue.source, issue.line, issue.target
        ));
    }
    output.push_str(&format!("    orphan pages: {}\n", metrics.orphan_pages));
    for path in metrics
        .orphan_page_paths
        .iter()
        .take(if verbose { usize::MAX } else { 10 })
    {
        output.push_str(&format!("    {path}\n"));
    }
    match (
        metrics.reachable_pages_from_entrypoints,
        metrics.entrypoint_pages,
    ) {
        (Some(reachable), Some(entrypoints)) => output.push_str(&format!(
            "    reachable from entrypoints: {}/{} page(s), {} entrypoint(s)\n",
            reachable, metrics.generated_pages, entrypoints
        )),
        _ => output.push_str("    reachable from entrypoints: not configured\n"),
    }
    if verbose && !metrics.target_audit.is_empty() {
        output.push_str("  relationship audit:\n");
        for target in &metrics.target_audit {
            output.push_str(&format!(
                "    {} {}: {} reference(s), {} generated, {} internal",
                target.kind.as_str(),
                target.id,
                target.references,
                target.generated_references,
                target.internal_references
            ));
            if !target.issues.is_empty() {
                output.push_str(&format!(" [{}]", target.issues.join(", ")));
            }
            output.push('\n');
            output.push_str(&target.support.describe());
            for reference in &target.reference_locations {
                output.push_str(&format!(
                    "      {}:{} {}{}\n",
                    reference.source,
                    reference.line,
                    reference.reference_source.as_str(),
                    if reference.generated {
                        " generated"
                    } else {
                        " internal"
                    }
                ));
            }
        }
    }
    output.trim_end().to_string()
}

impl DocsTargetSupport {
    fn describe(&self) -> String {
        let mut parts = Vec::new();
        if let Some(planned) = self.planned {
            parts.push(if planned { "planned" } else { "current" }.to_string());
        }
        if self.deprecated == Some(true) {
            parts.push("deprecated".to_string());
        }
        if let Some(verifies) = self.verifies {
            parts.push(format!("{verifies} verifies"));
        }
        if let Some(attests) = self.attests {
            parts.push(format!("{attests} attests"));
        }
        if let Some(implements) = self.implements {
            parts.push(format!("{implements} implements"));
        }
        if let Some(applications) = self.applications {
            parts.push(format!("{applications} applications"));
        }
        if let Some(strictness) = self.strictness.as_deref() {
            parts.push(format!("strictness {strictness}"));
        }
        if parts.is_empty() {
            return String::new();
        }
        format!("      support: {}\n", parts.join(", "))
    }
}

pub(crate) fn render_docs_metrics_json(document: &DocsMetricsDocument) -> Result<String> {
    Ok(serde_json::to_string_pretty(document)?)
}

struct GeneratedDocsGraph {
    pages: BTreeSet<PathBuf>,
    local_links: Vec<GeneratedDocsLink>,
    broken_links: Vec<DocsLocalLinkIssue>,
    orphan_pages: Vec<String>,
    reachable_pages_from_entrypoints: Option<usize>,
    entrypoint_pages: Option<usize>,
}

struct GeneratedDocsLink {
    source: PathBuf,
    target: PathBuf,
}

fn docs_metrics_summary(
    root: &Path,
    document: &DocsDocument,
    targets: &DocumentTargets,
    generated_sources: &[PathBuf],
    generated_graph: GeneratedDocsGraph,
) -> DocsMetricsSummary {
    DocsMetricsSummary {
        total_references: document.references.len(),
        link_references: document
            .references
            .iter()
            .filter(|reference| reference.source == DocumentRefSource::Link)
            .count(),
        documents_line_references: document
            .references
            .iter()
            .filter(|reference| reference.source == DocumentRefSource::DocumentsLine)
            .count(),
        file_documents_line_references: document
            .references
            .iter()
            .filter(|reference| reference.source == DocumentRefSource::FileDocumentsLine)
            .count(),
        generated_pages: generated_graph.pages.len(),
        local_doc_links: generated_graph.local_links.len(),
        broken_local_doc_links: generated_graph.broken_links.len(),
        orphan_pages: generated_graph.orphan_pages.len(),
        reachable_pages_from_entrypoints: generated_graph.reachable_pages_from_entrypoints,
        entrypoint_pages: generated_graph.entrypoint_pages,
        target_kinds: DocumentTargetKind::all()
            .into_iter()
            .map(|kind| target_kind_metrics(kind, document, generated_sources))
            .collect(),
        coverage: documentation_coverage_summary(document, targets, generated_sources),
        broken_local_doc_link_details: generated_graph.broken_links,
        orphan_page_paths: generated_graph.orphan_pages,
        target_audit: docs_target_audit(root, document, targets, generated_sources),
    }
}

fn docs_target_audit(
    root: &Path,
    document: &DocsDocument,
    targets: &DocumentTargets,
    generated_sources: &[PathBuf],
) -> Vec<DocsTargetAudit> {
    let mut references_by_target =
        BTreeMap::<(DocumentTargetKind, String), Vec<&DocumentRef>>::new();
    for reference in &document.references {
        references_by_target
            .entry((reference.target_kind, reference.target_id.clone()))
            .or_default()
            .push(reference);
    }

    references_by_target
        .into_iter()
        .map(|((kind, id), references)| {
            let generated_references = references
                .iter()
                .filter(|reference| {
                    is_generated_doc_source(&reference.location.path, generated_sources)
                })
                .count();
            let support = targets.support(kind, &id);
            let issues = target_audit_issues(kind, &support);
            let reference_locations = references
                .iter()
                .map(|reference| DocsTargetAuditReference {
                    source: display_path(root, &reference.location.path),
                    line: reference.location.line,
                    reference_source: reference.source,
                    generated: is_generated_doc_source(&reference.location.path, generated_sources),
                    text: reference.text.clone(),
                })
                .collect::<Vec<_>>();
            let location = targets.target_location(kind, &id).cloned();
            DocsTargetAudit {
                kind,
                id,
                references: references.len(),
                generated_references,
                internal_references: references.len() - generated_references,
                location,
                support,
                issues,
                reference_locations,
            }
        })
        .collect()
}

fn target_audit_issues(kind: DocumentTargetKind, support: &DocsTargetSupport) -> Vec<String> {
    let mut issues = Vec::new();
    if support.text.is_none() {
        issues.push("unknown_target".to_string());
        return issues;
    }
    match kind {
        DocumentTargetKind::Spec => {
            if support.planned == Some(true) {
                issues.push("planned_spec".to_string());
            }
            if support.planned == Some(false)
                && support.deprecated == Some(false)
                && support.verifies.unwrap_or_default() == 0
                && support.attests.unwrap_or_default() == 0
            {
                issues.push("current_spec_without_support".to_string());
            }
        }
        DocumentTargetKind::Module => {
            if support.planned == Some(false) && support.implements.unwrap_or_default() == 0 {
                issues.push("current_module_without_implements".to_string());
            }
        }
        DocumentTargetKind::Pattern => {
            if support.applications.unwrap_or_default() == 0 {
                issues.push("pattern_without_applications".to_string());
            }
        }
        DocumentTargetKind::Group | DocumentTargetKind::Area => {}
    }
    issues
}

fn target_kind_metrics(
    kind: DocumentTargetKind,
    document: &DocsDocument,
    generated_sources: &[PathBuf],
) -> DocsTargetKindMetrics {
    let mut documented = BTreeSet::new();
    let mut generated = BTreeSet::new();
    let mut internal = BTreeSet::new();
    let mut references = 0;
    let mut link_references = 0;
    let mut documents_line_references = 0;
    let mut file_documents_line_references = 0;

    for reference in &document.references {
        if reference.target_kind != kind {
            continue;
        }
        references += 1;
        match reference.source {
            DocumentRefSource::Link => link_references += 1,
            DocumentRefSource::DocumentsLine => documents_line_references += 1,
            DocumentRefSource::FileDocumentsLine => file_documents_line_references += 1,
        }
        documented.insert(reference.target_id.clone());
        if is_generated_doc_source(&reference.location.path, generated_sources) {
            generated.insert(reference.target_id.clone());
        } else {
            internal.insert(reference.target_id.clone());
        }
    }

    let internal_only = documented
        .iter()
        .filter(|id| internal.contains(*id) && !generated.contains(*id))
        .count();

    DocsTargetKindMetrics {
        kind,
        references,
        link_references,
        documents_line_references,
        file_documents_line_references,
        documented_targets: documented.len(),
        generated: generated.len(),
        internal_only,
    }
}

fn documentation_coverage_summary(
    document: &DocsDocument,
    targets: &DocumentTargets,
    generated_sources: &[PathBuf],
) -> DocumentationCoverageSummary {
    DocumentationCoverageSummary {
        target_kinds: DocumentTargetKind::all()
            .into_iter()
            .map(|kind| documentation_target_coverage(kind, document, targets, generated_sources))
            .collect(),
    }
}

fn documentation_target_coverage(
    kind: DocumentTargetKind,
    document: &DocsDocument,
    targets: &DocumentTargets,
    generated_sources: &[PathBuf],
) -> DocumentationTargetCoverage {
    let target_ids = targets.coverage_ids(kind, generated_sources);
    let mut documented = BTreeSet::new();
    let mut generated = BTreeSet::new();
    let mut internal = BTreeSet::new();

    for reference in &document.references {
        if reference.target_kind != kind {
            continue;
        }
        documented.insert(reference.target_id.clone());
        if is_generated_doc_source(&reference.location.path, generated_sources) {
            generated.insert(reference.target_id.clone());
        } else {
            internal.insert(reference.target_id.clone());
        }
    }

    let undocumented_ids = target_ids
        .iter()
        .filter(|id| !documented.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    let internal_only = target_ids
        .iter()
        .filter(|id| internal.contains(*id) && !generated.contains(*id))
        .count();

    DocumentationTargetCoverage {
        kind: kind.as_str().to_string(),
        total: target_ids.len(),
        documented: target_ids
            .iter()
            .filter(|id| documented.contains(*id))
            .count(),
        generated: target_ids
            .iter()
            .filter(|id| generated.contains(*id))
            .count(),
        internal_only,
        undocumented: undocumented_ids.len(),
        undocumented_ids,
    }
}

fn configured_output_sources(root: &Path, outputs: &[DocsOutputConfig]) -> Vec<PathBuf> {
    outputs
        .iter()
        .map(|output| configured_docs_path(root, &output.source))
        .collect()
}

fn configured_docs_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn is_generated_doc_source(path: &Path, generated_sources: &[PathBuf]) -> bool {
    generated_sources.iter().any(|source| {
        if source.is_file() || is_markdown_path(source) {
            path == source
        } else {
            path.starts_with(source)
        }
    })
}

fn build_generated_docs_graph(
    root: &Path,
    outputs: &[DocsOutputConfig],
    entrypoints: &[PathBuf],
) -> Result<GeneratedDocsGraph> {
    let mut plan = Vec::new();
    for output in outputs {
        expand_output_mapping(
            &configured_docs_path(root, &output.source),
            &configured_docs_path(root, &output.output),
            &mut plan,
        )?;
    }
    let markdown_plan = plan
        .into_iter()
        .filter(|(source, output)| is_markdown_path(source) && is_markdown_path(output))
        .collect::<Vec<_>>();
    let pages = markdown_plan
        .iter()
        .map(|(_, output)| output.clone())
        .collect::<BTreeSet<_>>();
    let page_lookup = pages.iter().cloned().collect::<BTreeSet<_>>();
    let mut local_links = Vec::new();
    let mut broken_links = Vec::new();

    for (source, output) in &markdown_plan {
        let content =
            fs::read_to_string(source).with_context(|| format!("reading {}", source.display()))?;
        for link in collect_local_markdown_links(&content) {
            let target = resolve_local_doc_link(output, &link.target);
            let display_target = display_path(root, &target);
            if page_lookup.contains(&target) {
                local_links.push(GeneratedDocsLink {
                    source: output.clone(),
                    target,
                });
            } else {
                broken_links.push(DocsLocalLinkIssue {
                    source: display_path(root, output),
                    line: link.line,
                    target: display_target,
                });
            }
        }
    }

    let incoming = incoming_link_counts(&pages, &local_links);
    let entrypoint_pages = entrypoints
        .iter()
        .map(|entrypoint| configured_docs_path(root, entrypoint))
        .filter(|entrypoint| pages.contains(entrypoint))
        .collect::<BTreeSet<_>>();
    let orphan_pages = pages
        .iter()
        .filter(|page| !entrypoint_pages.contains(*page))
        .filter(|page| incoming.get(*page).copied().unwrap_or_default() == 0)
        .map(|page| display_path(root, page))
        .collect::<Vec<_>>();
    let reachable_pages_from_entrypoints =
        (!entrypoints.is_empty()).then(|| reachable_pages(&entrypoint_pages, &local_links).len());
    let entrypoint_pages_count = (!entrypoints.is_empty()).then_some(entrypoint_pages.len());

    Ok(GeneratedDocsGraph {
        pages,
        local_links,
        broken_links,
        orphan_pages,
        reachable_pages_from_entrypoints,
        entrypoint_pages: entrypoint_pages_count,
    })
}

#[derive(Debug)]
struct LocalMarkdownLink {
    line: usize,
    target: String,
}

fn collect_local_markdown_links(content: &str) -> Vec<LocalMarkdownLink> {
    let mut links = Vec::new();
    let mut in_code_fence = false;
    for (index, line) in content.lines().enumerate() {
        if starts_markdown_fence(line) {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }
        collect_local_markdown_links_from_line(line, index + 1, &mut links);
    }
    links
}

fn collect_local_markdown_links_from_line(
    line: &str,
    line_number: usize,
    links: &mut Vec<LocalMarkdownLink>,
) {
    let bytes = line.as_bytes();
    let mut cursor = 0;
    while let Some(label_start_offset) = line[cursor..].find('[') {
        let label_start = cursor + label_start_offset;
        if label_start > 0 && bytes[label_start - 1] == b'!' {
            cursor = label_start + 1;
            continue;
        }
        let Some(label_end_offset) = line[label_start + 1..].find(']') else {
            break;
        };
        let label_end = label_start + 1 + label_end_offset;
        if !line[label_end + 1..].starts_with('(') {
            cursor = label_end + 1;
            continue;
        }
        let target_start = label_end + 2;
        let Some(target_end_offset) = line[target_start..].find(')') else {
            break;
        };
        let target_end = target_start + target_end_offset;
        let target = line[target_start..target_end].trim();
        if is_local_docs_link_target(target) {
            links.push(LocalMarkdownLink {
                line: line_number,
                target: target.to_string(),
            });
        }
        cursor = target_end + 1;
    }
}

fn is_local_docs_link_target(target: &str) -> bool {
    !target.is_empty()
        && !target.starts_with('#')
        && !target.starts_with("documents://")
        && !target.starts_with("special://")
        && !target.starts_with("http://")
        && !target.starts_with("https://")
        && !target.starts_with("mailto:")
}

fn resolve_local_doc_link(source_output: &Path, target: &str) -> PathBuf {
    let without_fragment = target.split('#').next().unwrap_or(target);
    let target_path = Path::new(without_fragment);
    let resolved = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        source_output
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(target_path)
    };
    normalize_lexical_path(&resolved)
}

fn normalize_lexical_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn incoming_link_counts(
    pages: &BTreeSet<PathBuf>,
    links: &[GeneratedDocsLink],
) -> BTreeMap<PathBuf, usize> {
    let mut incoming = pages
        .iter()
        .map(|page| (page.clone(), 0))
        .collect::<BTreeMap<_, _>>();
    for link in links {
        if let Some(count) = incoming.get_mut(&link.target) {
            *count += 1;
        }
    }
    incoming
}

fn reachable_pages(
    entrypoints: &BTreeSet<PathBuf>,
    links: &[GeneratedDocsLink],
) -> BTreeSet<PathBuf> {
    let mut adjacency: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for link in links {
        adjacency
            .entry(link.source.clone())
            .or_default()
            .push(link.target.clone());
    }
    let mut visited = BTreeSet::new();
    let mut pending = entrypoints.iter().cloned().collect::<VecDeque<_>>();
    while let Some(page) = pending.pop_front() {
        if !visited.insert(page.clone()) {
            continue;
        }
        if let Some(targets) = adjacency.get(&page) {
            pending.extend(targets.iter().cloned());
        }
    }
    visited
}

fn collect_repo_document_refs(
    root: &Path,
    ignore_patterns: &[String],
) -> Result<(Vec<DocumentRef>, Vec<Diagnostic>)> {
    let mut refs = Vec::new();
    let mut diagnostics = Vec::new();
    collect_source_document_refs(root, ignore_patterns, &mut refs, &mut diagnostics)?;
    collect_markdown_document_refs(root, ignore_patterns, &mut refs, &mut diagnostics)?;
    Ok((refs, diagnostics))
}

fn scoped_refs(refs: Vec<DocumentRef>, scope_paths: &[PathBuf]) -> Vec<DocumentRef> {
    if scope_paths.is_empty() {
        return refs;
    }
    refs.into_iter()
        .filter(|reference| matches_scope_path(&reference.location.path, scope_paths))
        .collect()
}

fn retain_scoped_diagnostics(diagnostics: &mut Vec<Diagnostic>, scope_paths: &[PathBuf]) {
    if scope_paths.is_empty() {
        return;
    }
    diagnostics.retain(|diagnostic| matches_scope_path(&diagnostic.path, scope_paths));
}

fn collect_source_document_refs(
    root: &Path,
    ignore_patterns: &[String],
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
    for block in collect_comment_blocks(root, ignore_patterns)? {
        let mut previous_docs_line = false;
        for entry in &block.lines {
            let parsed = parse_documents_annotation_line(
                entry.text.trim(),
                &block.path,
                entry.line,
                refs,
                diagnostics,
            );
            if parsed && previous_docs_line {
                push_stacked_document_line_diagnostic(&block.path, entry.line, diagnostics);
            }
            previous_docs_line = parsed;
        }
    }
    Ok(())
}

fn collect_markdown_document_refs(
    root: &Path,
    ignore_patterns: &[String],
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
    for path in discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .markdown_files
    {
        let content =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        collect_markdown_refs(&path, &content, refs, diagnostics);
    }
    Ok(())
}

fn collect_markdown_refs(
    path: &Path,
    content: &str,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut in_code_fence = false;
    let mut previous_docs_line = false;
    for (index, line) in content.lines().enumerate() {
        let line_number = index + 1;
        if starts_markdown_fence(line) {
            in_code_fence = !in_code_fence;
            previous_docs_line = false;
            continue;
        }
        if in_code_fence {
            continue;
        }
        let parsed = parse_markdown_documents_line(line, path, line_number, refs, diagnostics);
        if parsed && previous_docs_line {
            push_stacked_document_line_diagnostic(path, line_number, diagnostics);
        }
        previous_docs_line = parsed;
        collect_document_link_refs(path, line, line_number, refs, diagnostics);
        collect_reserved_special_link_diagnostics(path, line, line_number, diagnostics);
    }
}

fn write_markdown_output(
    path: &Path,
    content: &str,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    let mut output = String::new();
    let mut in_code_fence = false;
    let mut previous_docs_line = false;

    for (index, line) in content.split_inclusive('\n').enumerate() {
        let line_number = index + 1;
        let raw = line.trim_end_matches('\n').trim_end_matches('\r');
        if starts_markdown_fence(raw) {
            in_code_fence = !in_code_fence;
            previous_docs_line = false;
            output.push_str(line);
            continue;
        }
        if in_code_fence {
            output.push_str(line);
            continue;
        }

        if parse_markdown_documents_line(raw, path, line_number, refs, diagnostics) {
            if previous_docs_line {
                push_stacked_document_line_diagnostic(path, line_number, diagnostics);
            }
            previous_docs_line = true;
            continue;
        }
        previous_docs_line = false;

        if is_markdown_authoring_annotation_line(raw) {
            continue;
        }

        collect_reserved_special_link_diagnostics(path, line, line_number, diagnostics);
        output.push_str(&write_document_link_output(
            path,
            line,
            line_number,
            refs,
            diagnostics,
        ));
    }

    output
}

fn parse_markdown_documents_line(
    line: &str,
    path: &Path,
    line_number: usize,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    if parse_source_annotations::is_whole_line_code_span(line) {
        return false;
    }
    let Some(trimmed) = normalize_markdown_declaration_line(line) else {
        return false;
    };
    parse_documents_annotation_line(trimmed, path, line_number, refs, diagnostics)
}

fn is_markdown_authoring_annotation_line(line: &str) -> bool {
    if parse_source_annotations::is_whole_line_code_span(line) {
        return false;
    }
    let Some(trimmed) = normalize_markdown_annotation_line(line) else {
        return false;
    };
    [
        ReservedSpecialAnnotation::Implements,
        ReservedSpecialAnnotation::FileImplements,
        ReservedSpecialAnnotation::Applies,
        ReservedSpecialAnnotation::FileApplies,
    ]
    .into_iter()
    .any(|annotation| reserved_special_annotation_rest(trimmed, annotation).is_some())
}

fn push_stacked_document_line_diagnostic(
    path: &Path,
    line_number: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.push(Diagnostic {
        severity: DiagnosticSeverity::Error,
        path: path.to_path_buf(),
        line: line_number,
        message:
            "documentation relationship lines may not be stacked; use local documents:// links"
                .to_string(),
    });
}

fn write_document_link_output(
    path: &Path,
    line: &str,
    line_number: usize,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    let mut output = String::new();
    let mut cursor = 0;

    for link in document_markdown_link_candidates(line) {
        output.push_str(&line[cursor..link.span.start]);
        output.push_str(link.label);
        push_document_link_ref(path, line_number, link, refs, diagnostics);
        cursor = link.span.end;
    }

    output.push_str(&line[cursor..]);
    output
}

fn collect_document_link_refs(
    path: &Path,
    line: &str,
    line_number: usize,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for link in document_markdown_link_candidates(line) {
        push_document_link_ref(path, line_number, link, refs, diagnostics);
    }
}

fn collect_reserved_special_link_diagnostics(
    path: &Path,
    line: &str,
    line_number: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for link in parse_source_annotations::special_markdown_link_candidates(line) {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: format!(
                "`special://` is reserved; use `documents://kind/ID` for documentation evidence links, got `{}`",
                link.uri
            ),
        });
    }
}

fn push_document_link_ref(
    path: &Path,
    line_number: usize,
    link: DocumentMarkdownLinkCandidate<'_>,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(target) = link.target else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: format!("malformed documents URI `{}`", link.uri),
        });
        return;
    };

    match DocumentTargetKind::parse(target.kind) {
        Some(target_kind) => refs.push(DocumentRef {
            target_kind,
            target_id: target.id.to_string(),
            location: SourceLocation {
                path: path.to_path_buf(),
                line: line_number,
            },
            source: DocumentRefSource::Link,
            text: Some(link.label.to_string()),
        }),
        None => diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: format!("unknown documents target kind `{}`", target.kind),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DocumentMarkdownLinkCandidate<'a> {
    label: &'a str,
    uri: &'a str,
    target: Option<DocumentLinkTarget<'a>>,
    span: parse_source_annotations::TextSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DocumentLinkTarget<'a> {
    kind: &'a str,
    id: &'a str,
}

fn document_markdown_link_candidates(line: &str) -> Vec<DocumentMarkdownLinkCandidate<'_>> {
    let mut links = Vec::new();
    let mut cursor = 0;

    while let Some(open_rel) = line[cursor..].find('[') {
        let open = cursor + open_rel;
        if is_inside_inline_code_span(line, open) {
            cursor = open + 1;
            continue;
        }
        let Some(close_rel) = line[open + 1..].find("](") else {
            break;
        };
        let close = open + 1 + close_rel;
        let target_start = close + 2;
        let Some(target_end_rel) = line[target_start..].find(')') else {
            break;
        };
        let target_end = target_start + target_end_rel;
        let uri = &line[target_start..target_end];
        if uri.starts_with("documents://") {
            links.push(DocumentMarkdownLinkCandidate {
                label: &line[open + 1..close],
                uri,
                target: document_link_target(uri),
                span: parse_source_annotations::TextSpan {
                    start: open,
                    end: target_end + 1,
                },
            });
        }
        cursor = target_end + 1;
    }

    links
}

fn document_link_target(uri: &str) -> Option<DocumentLinkTarget<'_>> {
    let rest = uri.strip_prefix("documents://")?;
    let (kind, id) = rest.split_once('/')?;
    if kind.is_empty() || id.trim().is_empty() {
        return None;
    }
    Some(DocumentLinkTarget { kind, id })
}

fn is_inside_inline_code_span(line: &str, byte_index: usize) -> bool {
    let mut in_code = false;
    for (index, character) in line.char_indices() {
        if index >= byte_index {
            break;
        }
        if character == '`' {
            in_code = !in_code;
        }
    }
    in_code
}

fn parse_documents_annotation_line(
    line: &str,
    path: &Path,
    line_number: usize,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    parse_documents_line(
        line,
        path,
        line_number,
        DocumentRefSource::DocumentsLine,
        refs,
        diagnostics,
    ) || parse_filedocuments_line(
        line,
        path,
        line_number,
        DocumentRefSource::FileDocumentsLine,
        refs,
        diagnostics,
    )
}

fn parse_documents_line(
    line: &str,
    path: &Path,
    line_number: usize,
    source: DocumentRefSource,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    let Some(rest) = reserved_special_annotation_rest(line, ReservedSpecialAnnotation::Documents)
    else {
        return false;
    };
    parse_documents_rest(rest, path, line_number, source, refs, diagnostics);
    true
}

fn parse_filedocuments_line(
    line: &str,
    path: &Path,
    line_number: usize,
    source: DocumentRefSource,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    let Some(rest) =
        reserved_special_annotation_rest(line, ReservedSpecialAnnotation::FileDocuments)
    else {
        return false;
    };
    parse_documents_rest(rest, path, line_number, source, refs, diagnostics);
    true
}

fn parse_documents_rest(
    rest: &str,
    path: &Path,
    line_number: usize,
    source: DocumentRefSource,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut parts = rest.split_whitespace();
    let Some(kind) = parts.next() else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: "missing docs target kind after documentation annotation".to_string(),
        });
        return;
    };
    let Some(target_kind) = DocumentTargetKind::parse(kind) else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: format!("unknown docs target kind `{kind}`"),
        });
        return;
    };
    let Some(id) = parts.next() else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: "missing docs target id after documentation annotation".to_string(),
        });
        return;
    };
    if parts.next().is_some() {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: "unexpected trailing content after documentation target id".to_string(),
        });
        return;
    }
    refs.push(DocumentRef {
        target_kind,
        target_id: id.to_string(),
        location: SourceLocation {
            path: path.to_path_buf(),
            line: line_number,
        },
        source,
        text: None,
    });
}

fn validate_document_refs(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    refs: Vec<DocumentRef>,
) -> Result<Vec<Diagnostic>> {
    let targets = DocumentTargets::load(root, ignore_patterns, version)?;
    Ok(refs
        .into_iter()
        .filter_map(|reference| {
            if targets.contains(reference.target_kind, &reference.target_id) {
                None
            } else {
                Some(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: reference.location.path,
                    line: reference.location.line,
                    message: format!(
                        "unknown documentation target {} `{}`",
                        reference.target_kind.as_str(),
                        reference.target_id
                    ),
                })
            }
        })
        .collect())
}

struct DocumentTargets {
    specs: BTreeMap<String, SourceLocation>,
    groups: BTreeMap<String, SourceLocation>,
    modules: BTreeMap<String, SourceLocation>,
    areas: BTreeMap<String, SourceLocation>,
    patterns: BTreeMap<String, SourceLocation>,
    spec_support: BTreeMap<String, DocsTargetSupport>,
    group_support: BTreeMap<String, DocsTargetSupport>,
    module_support: BTreeMap<String, DocsTargetSupport>,
    area_support: BTreeMap<String, DocsTargetSupport>,
    pattern_support: BTreeMap<String, DocsTargetSupport>,
    module_implementation_locations: BTreeMap<String, Vec<SourceLocation>>,
    pattern_application_locations: BTreeMap<String, Vec<SourceLocation>>,
}

impl DocumentTargets {
    fn load(root: &Path, ignore_patterns: &[String], version: SpecialVersion) -> Result<Self> {
        let parsed_repo = load_or_parse_repo(root, ignore_patterns, version)?;
        let parsed_architecture = load_or_parse_architecture(root, ignore_patterns)?;
        let verifies_by_spec = count_spec_ids(
            parsed_repo
                .verifies
                .iter()
                .map(|reference| reference.spec_id.as_str()),
        );
        let attests_by_spec = count_spec_ids(
            parsed_repo
                .attests
                .iter()
                .map(|reference| reference.spec_id.as_str()),
        );
        let module_implementation_locations =
            collect_module_implementation_locations(&parsed_architecture.implements);
        let pattern_application_locations =
            collect_pattern_application_locations(&parsed_architecture.pattern_applications);
        Ok(Self {
            specs: parsed_repo
                .specs
                .iter()
                .filter(|decl| decl.kind() == NodeKind::Spec)
                .map(|decl| (decl.id.clone(), decl.location.clone()))
                .collect(),
            groups: parsed_repo
                .specs
                .iter()
                .filter(|decl| decl.kind() == NodeKind::Group)
                .map(|decl| (decl.id.clone(), decl.location.clone()))
                .collect(),
            modules: parsed_architecture
                .modules
                .iter()
                .filter(|decl| decl.kind() == ArchitectureKind::Module)
                .map(|decl| (decl.id.clone(), decl.location.clone()))
                .collect(),
            areas: parsed_architecture
                .modules
                .iter()
                .filter(|decl| decl.kind() == ArchitectureKind::Area)
                .map(|decl| (decl.id.clone(), decl.location.clone()))
                .collect(),
            patterns: parsed_architecture
                .patterns
                .iter()
                .map(|definition| (definition.pattern_id.clone(), definition.location.clone()))
                .collect(),
            spec_support: parsed_repo
                .specs
                .iter()
                .filter(|decl| decl.kind() == NodeKind::Spec)
                .map(|decl| {
                    (
                        decl.id.clone(),
                        DocsTargetSupport {
                            text: Some(decl.text.clone()),
                            planned: Some(decl.is_planned()),
                            deprecated: Some(decl.is_deprecated()),
                            verifies: Some(*verifies_by_spec.get(&decl.id).unwrap_or(&0)),
                            attests: Some(*attests_by_spec.get(&decl.id).unwrap_or(&0)),
                            ..DocsTargetSupport::default()
                        },
                    )
                })
                .collect(),
            group_support: parsed_repo
                .specs
                .iter()
                .filter(|decl| decl.kind() == NodeKind::Group)
                .map(|decl| {
                    (
                        decl.id.clone(),
                        DocsTargetSupport {
                            text: Some(decl.text.clone()),
                            ..DocsTargetSupport::default()
                        },
                    )
                })
                .collect(),
            module_support: parsed_architecture
                .modules
                .iter()
                .filter(|decl| decl.kind() == ArchitectureKind::Module)
                .map(|decl| {
                    (
                        decl.id.clone(),
                        DocsTargetSupport {
                            text: Some(decl.text.clone()),
                            planned: Some(decl.is_planned()),
                            implements: Some(
                                module_implementation_locations
                                    .get(&decl.id)
                                    .map(Vec::len)
                                    .unwrap_or_default(),
                            ),
                            ..DocsTargetSupport::default()
                        },
                    )
                })
                .collect(),
            area_support: parsed_architecture
                .modules
                .iter()
                .filter(|decl| decl.kind() == ArchitectureKind::Area)
                .map(|decl| {
                    (
                        decl.id.clone(),
                        DocsTargetSupport {
                            text: Some(decl.text.clone()),
                            ..DocsTargetSupport::default()
                        },
                    )
                })
                .collect(),
            pattern_support: parsed_architecture
                .patterns
                .iter()
                .map(|definition| {
                    (
                        definition.pattern_id.clone(),
                        DocsTargetSupport {
                            text: Some(definition.text.clone()),
                            applications: Some(
                                pattern_application_locations
                                    .get(&definition.pattern_id)
                                    .map(Vec::len)
                                    .unwrap_or_default(),
                            ),
                            strictness: Some(definition.strictness.as_str().to_string()),
                            ..DocsTargetSupport::default()
                        },
                    )
                })
                .collect(),
            module_implementation_locations,
            pattern_application_locations,
        })
    }

    fn contains(&self, kind: DocumentTargetKind, id: &str) -> bool {
        match kind {
            DocumentTargetKind::Spec => self.specs.contains_key(id),
            DocumentTargetKind::Group => self.groups.contains_key(id),
            DocumentTargetKind::Module => self.modules.contains_key(id),
            DocumentTargetKind::Area => self.areas.contains_key(id),
            DocumentTargetKind::Pattern => self.patterns.contains_key(id),
        }
    }

    fn targets(&self, kind: DocumentTargetKind) -> &BTreeMap<String, SourceLocation> {
        match kind {
            DocumentTargetKind::Spec => &self.specs,
            DocumentTargetKind::Group => &self.groups,
            DocumentTargetKind::Module => &self.modules,
            DocumentTargetKind::Area => &self.areas,
            DocumentTargetKind::Pattern => &self.patterns,
        }
    }

    fn target_location(&self, kind: DocumentTargetKind, id: &str) -> Option<&SourceLocation> {
        self.targets(kind).get(id)
    }

    fn support(&self, kind: DocumentTargetKind, id: &str) -> DocsTargetSupport {
        match kind {
            DocumentTargetKind::Spec => &self.spec_support,
            DocumentTargetKind::Group => &self.group_support,
            DocumentTargetKind::Module => &self.module_support,
            DocumentTargetKind::Area => &self.area_support,
            DocumentTargetKind::Pattern => &self.pattern_support,
        }
        .get(id)
        .cloned()
        .unwrap_or_default()
    }

    fn coverage_ids(
        &self,
        kind: DocumentTargetKind,
        generated_sources: &[PathBuf],
    ) -> BTreeSet<String> {
        self.targets(kind)
            .iter()
            .filter(|(id, location)| {
                !self.is_docs_source_target_excluded_from_coverage(
                    kind,
                    id,
                    location,
                    generated_sources,
                )
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    fn is_docs_source_target_excluded_from_coverage(
        &self,
        kind: DocumentTargetKind,
        id: &str,
        location: &SourceLocation,
        generated_sources: &[PathBuf],
    ) -> bool {
        match kind {
            DocumentTargetKind::Spec | DocumentTargetKind::Group => false,
            DocumentTargetKind::Module => {
                self.module_is_docs_source_target(id, location, generated_sources)
            }
            DocumentTargetKind::Area => {
                self.area_is_docs_source_target(id, location, generated_sources)
            }
            DocumentTargetKind::Pattern => {
                self.pattern_is_docs_source_target(id, location, generated_sources)
            }
        }
    }

    fn module_is_docs_source_target(
        &self,
        id: &str,
        location: &SourceLocation,
        generated_sources: &[PathBuf],
    ) -> bool {
        is_generated_doc_source(&location.path, generated_sources)
            || self
                .module_implementation_locations
                .get(id)
                .is_some_and(|locations| {
                    locations
                        .iter()
                        .any(|location| is_generated_doc_source(&location.path, generated_sources))
                })
    }

    fn area_is_docs_source_target(
        &self,
        id: &str,
        location: &SourceLocation,
        generated_sources: &[PathBuf],
    ) -> bool {
        is_generated_doc_source(&location.path, generated_sources)
            || self.modules.iter().any(|(module_id, module_location)| {
                module_id.starts_with(&format!("{id}."))
                    && self.module_is_docs_source_target(
                        module_id,
                        module_location,
                        generated_sources,
                    )
            })
    }

    fn pattern_is_docs_source_target(
        &self,
        id: &str,
        location: &SourceLocation,
        generated_sources: &[PathBuf],
    ) -> bool {
        is_generated_doc_source(&location.path, generated_sources)
            || self
                .pattern_application_locations
                .get(id)
                .is_some_and(|locations| {
                    locations
                        .iter()
                        .any(|location| is_generated_doc_source(&location.path, generated_sources))
                })
    }
}

fn collect_module_implementation_locations(
    implementations: &[crate::model::ImplementRef],
) -> BTreeMap<String, Vec<SourceLocation>> {
    let mut locations = BTreeMap::<String, Vec<SourceLocation>>::new();
    for implementation in implementations {
        locations
            .entry(implementation.module_id.clone())
            .or_default()
            .push(
                implementation
                    .body_location
                    .clone()
                    .unwrap_or_else(|| implementation.location.clone()),
            );
    }
    locations
}

fn count_spec_ids<'a>(refs: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for spec_id in refs {
        *counts.entry(spec_id.to_string()).or_default() += 1;
    }
    counts
}

fn collect_pattern_application_locations(
    applications: &[crate::model::PatternApplication],
) -> BTreeMap<String, Vec<SourceLocation>> {
    let mut locations = BTreeMap::<String, Vec<SourceLocation>>::new();
    for application in applications {
        locations
            .entry(application.pattern_id.clone())
            .or_default()
            .push(
                application
                    .body_location
                    .clone()
                    .unwrap_or_else(|| application.location.clone()),
            );
    }
    locations
}

fn is_markdown_path(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("md"))
}

fn validate_output_plan(files: &[(PathBuf, PathBuf)]) -> Result<()> {
    let mut outputs = BTreeSet::new();
    for (input, output) in files {
        ensure_distinct_paths(
            input,
            output,
            "docs output path must not equal an input path",
        )?;
        let normalized_output = normalize_existing_path(output);
        if !outputs.insert(normalized_output) {
            bail!("docs output maps multiple inputs to {}", output.display());
        }
        if output.exists() && existing_file_contains_docs_evidence(output)? {
            bail!(
                "refusing to overwrite docs evidence in {}; choose a separate output path",
                output.display()
            );
        }
    }
    Ok(())
}

fn existing_file_contains_docs_evidence(path: &Path) -> Result<bool> {
    let content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut in_fenced_code = false;
    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_fenced_code = !in_fenced_code;
            continue;
        }
        if in_fenced_code {
            continue;
        }
        let evidence_line = remove_inline_code_spans(line);
        if evidence_line.contains("documents://")
            || evidence_line.contains("special://")
            || evidence_line.contains("@documents")
            || evidence_line.contains("@filedocuments")
            || is_markdown_authoring_annotation_line(line)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn remove_inline_code_spans(line: &str) -> String {
    let mut output = String::new();
    let mut in_code = false;
    for character in line.chars() {
        if character == '`' {
            in_code = !in_code;
            continue;
        }
        if !in_code {
            output.push(character);
        }
    }
    output
}

fn ensure_distinct_paths(left: &Path, right: &Path, message: &str) -> Result<()> {
    if normalize_existing_path(left) == normalize_existing_path(right) {
        bail!("{message}");
    }
    Ok(())
}

fn path_is_inside(path: &Path, parent: &Path) -> bool {
    let path = normalize_existing_path(path);
    let parent = normalize_existing_path(parent);
    path != parent && path.starts_with(parent)
}

fn normalize_existing_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.line.cmp(&right.line))
            .then(left.message.cmp(&right.message))
    });
}
