/**
@module SPECIAL.DOCS
Validates documentation-to-Special relationships and renders public markdown from docs sources.

@group SPECIAL.DOCS
Documentation relationship and docs-output behavior.

@spec SPECIAL.DOCS.LINKS
Markdown links whose destination is `special://KIND/ID` attach the linked label text as documentation evidence for the targeted Special identifier.

@spec SPECIAL.DOCS.LINKS.POLYMORPHIC
Documentation links accept `spec`, `group`, `module`, `area`, and `pattern` targets.

@spec SPECIAL.DOCS.LINKS.OUTPUT
special docs --target PATH --output PATH rewrites markdown `special://KIND/ID` links to their label text in the emitted artifact.

@spec SPECIAL.DOCS.DOCUMENTS_LINES
Documentation relationship lines `@documents KIND ID` and `@filedocuments KIND ID` can be stacked and are removed from docs output.

@spec SPECIAL.DOCS_COMMAND
special docs validates documentation links and prints a documentation relationship view without writing files.

@spec SPECIAL.DOCS_COMMAND.TARGET
special docs --target PATH validates and prints only documentation relationships under the target file or subtree without writing files.

@spec SPECIAL.DOCS_COMMAND.PATH_SCOPE_SYNTAX
special docs rejects hidden positional path scopes and requires path scopes to use --target PATH.

@spec SPECIAL.DOCS_COMMAND.OUTPUT
special docs --target PATH --output PATH validates documentation links and writes public docs.

@spec SPECIAL.DOCS_COMMAND.OUTPUT.DIRECTORY
special docs --target PATH --output PATH accepts an input directory and output directory, then mirrors the input tree relative to the target root while writing markdown output files.

@spec SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY
special docs --target PATH --output PATH refuses to write docs output over the input path, into an input directory, or over an existing file that still contains docs evidence.

@spec SPECIAL.DOCS_COMMAND.OUTPUT.CONFIG
special docs --output uses `[[docs.outputs]]` mappings from special.toml to write configured public docs without repeating paths on the command line.
*/
// @fileimplements SPECIAL.DOCS
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

use crate::annotation_syntax::{
    ReservedSpecialAnnotation, normalize_markdown_declaration_line,
    reserved_special_annotation_rest,
};
use crate::cache::{load_or_parse_architecture, load_or_parse_repo};
use crate::config::SpecialVersion;
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::extractor::collect_comment_blocks;
use crate::model::{
    ArchitectureKind, Diagnostic, DiagnosticSeverity, LintReport, NodeKind, SourceLocation,
};
use crate::parser::starts_markdown_fence;

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
            plan.push((path.to_path_buf(), output.join(relative)));
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
        .filter(|reference| scope_matches(&reference.location.path, scope_paths))
        .collect()
}

fn retain_scoped_diagnostics(diagnostics: &mut Vec<Diagnostic>, scope_paths: &[PathBuf]) {
    if scope_paths.is_empty() {
        return;
    }
    diagnostics.retain(|diagnostic| scope_matches(&diagnostic.path, scope_paths));
}

fn scope_matches(path: &Path, scope_paths: &[PathBuf]) -> bool {
    scope_paths.iter().any(|scope| {
        if scope.is_dir() {
            path.starts_with(scope)
        } else {
            path == scope || path.starts_with(scope)
        }
    })
}

fn collect_source_document_refs(
    root: &Path,
    ignore_patterns: &[String],
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
    for block in collect_comment_blocks(root, ignore_patterns)? {
        for entry in &block.lines {
            parse_documents_annotation_line(
                entry.text.trim(),
                &block.path,
                entry.line,
                refs,
                diagnostics,
            );
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
    for (index, line) in content.lines().enumerate() {
        let line_number = index + 1;
        if starts_markdown_fence(line) {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }
        parse_markdown_documents_line(line, path, line_number, refs, diagnostics);
        collect_special_link_refs(path, line, line_number, refs, diagnostics);
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

    for (index, line) in content.split_inclusive('\n').enumerate() {
        let line_number = index + 1;
        let raw = line.trim_end_matches('\n').trim_end_matches('\r');
        if starts_markdown_fence(raw) {
            in_code_fence = !in_code_fence;
            output.push_str(line);
            continue;
        }
        if in_code_fence {
            output.push_str(line);
            continue;
        }

        if parse_markdown_documents_line(raw, path, line_number, refs, diagnostics) {
            continue;
        }

        output.push_str(&write_special_link_output(
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

fn write_special_link_output(
    path: &Path,
    line: &str,
    line_number: usize,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    let mut output = String::new();
    let mut cursor = 0;

    for link in parse_source_annotations::special_markdown_link_candidates(line) {
        output.push_str(&line[cursor..link.span.start]);
        output.push_str(link.label);
        push_special_link_ref(path, line_number, link, refs, diagnostics);
        cursor = link.span.end;
    }

    output.push_str(&line[cursor..]);
    output
}

fn collect_special_link_refs(
    path: &Path,
    line: &str,
    line_number: usize,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for link in parse_source_annotations::special_markdown_link_candidates(line) {
        push_special_link_ref(path, line_number, link, refs, diagnostics);
    }
}

fn push_special_link_ref(
    path: &Path,
    line_number: usize,
    link: parse_source_annotations::SpecialMarkdownLinkCandidate<'_>,
    refs: &mut Vec<DocumentRef>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(target) = link.target else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: format!("malformed Special docs URI `{}`", link.uri),
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
            message: format!("unknown Special docs target kind `{}`", target.kind),
        }),
    }
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
    specs: BTreeSet<String>,
    groups: BTreeSet<String>,
    modules: BTreeSet<String>,
    areas: BTreeSet<String>,
    patterns: BTreeSet<String>,
}

impl DocumentTargets {
    fn load(root: &Path, ignore_patterns: &[String], version: SpecialVersion) -> Result<Self> {
        let parsed_repo = load_or_parse_repo(root, ignore_patterns, version)?;
        let parsed_architecture = load_or_parse_architecture(root, ignore_patterns)?;
        Ok(Self {
            specs: parsed_repo
                .specs
                .iter()
                .filter(|decl| decl.kind() == NodeKind::Spec)
                .map(|decl| decl.id.clone())
                .collect(),
            groups: parsed_repo
                .specs
                .iter()
                .filter(|decl| decl.kind() == NodeKind::Group)
                .map(|decl| decl.id.clone())
                .collect(),
            modules: parsed_architecture
                .modules
                .iter()
                .filter(|decl| decl.kind() == ArchitectureKind::Module)
                .map(|decl| decl.id.clone())
                .collect(),
            areas: parsed_architecture
                .modules
                .iter()
                .filter(|decl| decl.kind() == ArchitectureKind::Area)
                .map(|decl| decl.id.clone())
                .collect(),
            patterns: parsed_architecture
                .patterns
                .iter()
                .map(|definition| definition.pattern_id.clone())
                .collect(),
        })
    }

    fn contains(&self, kind: DocumentTargetKind, id: &str) -> bool {
        match kind {
            DocumentTargetKind::Spec => self.specs.contains(id),
            DocumentTargetKind::Group => self.groups.contains(id),
            DocumentTargetKind::Module => self.modules.contains(id),
            DocumentTargetKind::Area => self.areas.contains(id),
            DocumentTargetKind::Pattern => self.patterns.contains(id),
        }
    }
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
    Ok(content.lines().any(|line| {
        line.contains("special://")
            || line.contains("@documents")
            || line.contains("@filedocuments")
    }))
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
