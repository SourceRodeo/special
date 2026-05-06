/**
@module SPECIAL.MODULES.PARSE.MARKDOWN
Parses architecture declarations and markdown-local implementation attachments from ordinary markdown files under the project root.
*/
// @fileimplements SPECIAL.MODULES.PARSE.MARKDOWN
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::annotation_syntax::{ReservedSpecialAnnotation, reserved_special_annotation_rest};
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{
    ArchitectureKind, Diagnostic, DiagnosticSeverity, ImplementRef, ModuleDecl, ParsedArchitecture,
    PatternApplication, PatternDefinition, PlanState, SourceLocation,
};

use super::parse::declarations::{
    collect_doc_description_lines, maybe_consume_doc_planned, maybe_consume_pattern_strictness,
    normalized_annotation_line, normalized_architecture_heading, normalized_pattern_heading,
    parse_implements_module_id, parse_module_header, parse_pattern_id, skip_blank_doc_lines,
};
use crate::parser::starts_markdown_fence;

pub(super) fn parse_markdown_architecture_decls(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &mut ParsedArchitecture,
) -> Result<()> {
    for path in discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .markdown_files
    {
        let content = fs::read_to_string(&path)?;
        let lines: Vec<&str> = content.lines().collect();
        let mut index = 0;
        let mut in_code_fence = false;

        while index < lines.len() {
            let raw = lines[index];
            if starts_markdown_fence(raw) {
                in_code_fence = !in_code_fence;
                index += 1;
                continue;
            }
            if in_code_fence {
                index += 1;
                continue;
            }

            if let Some((rest, file_scoped, annotation)) =
                normalized_markdown_implements_annotation(raw)
            {
                let line_number = index + 1;
                let Some(module_id) =
                    parse_implements_module_id(rest, annotation, parsed, &path, line_number)
                else {
                    index += 1;
                    continue;
                };

                let (body_location, body) = if file_scoped {
                    (None, None)
                } else if let Some(section) = markdown_implementation_section(&lines, index) {
                    (
                        Some(SourceLocation {
                            path: path.to_path_buf(),
                            line: section.start + 1,
                        }),
                        Some(markdown_section_body(&lines, &section, index)),
                    )
                } else {
                    parsed.diagnostics.push(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        path: path.to_path_buf(),
                        line: line_number,
                        message: "@implements must attach to a markdown heading section"
                            .to_string(),
                    });
                    index += 1;
                    continue;
                };

                parsed.implements.push(ImplementRef {
                    module_id,
                    location: SourceLocation {
                        path: path.to_path_buf(),
                        line: line_number,
                    },
                    body_location,
                    body,
                });
                index += 1;
                continue;
            }

            if let Some((rest, file_scoped, annotation)) =
                normalized_markdown_pattern_application(raw)
            {
                let line_number = index + 1;
                let Some(pattern_id) =
                    parse_pattern_id(rest, annotation, parsed, &path, line_number)
                else {
                    index += 1;
                    continue;
                };

                let (body_location, body) = if file_scoped {
                    (None, Some(markdown_file_body(&lines)))
                } else if let Some(section) = markdown_implementation_section(&lines, index) {
                    (
                        Some(SourceLocation {
                            path: path.to_path_buf(),
                            line: section.start + 1,
                        }),
                        Some(markdown_section_body(&lines, &section, index)),
                    )
                } else {
                    parsed.diagnostics.push(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        path: path.to_path_buf(),
                        line: line_number,
                        message: "@applies must attach to a markdown heading section".to_string(),
                    });
                    index += 1;
                    continue;
                };

                parsed.pattern_applications.push(PatternApplication {
                    pattern_id,
                    location: SourceLocation {
                        path: path.to_path_buf(),
                        line: line_number,
                    },
                    body_location,
                    body,
                });
                index += 1;
                continue;
            }

            if let Some(raw_pattern) = normalized_pattern_heading(raw) {
                let line_number = index + 1;
                let Some(pattern_id) =
                    parse_pattern_id(raw_pattern, "@pattern", parsed, &path, line_number)
                else {
                    index += 1;
                    continue;
                };
                let mut cursor = skip_blank_doc_lines(&lines, index + 1);
                let strictness = if let Some(strictness) = maybe_consume_pattern_strictness(
                    normalized_annotation_line(lines.get(cursor).copied()),
                    parsed,
                    &path,
                    cursor + 1,
                ) {
                    cursor = skip_blank_doc_lines(&lines, cursor + 1);
                    strictness
                } else {
                    Default::default()
                };
                let (description_lines, cursor) = collect_doc_description_lines(&lines, cursor);
                parsed.patterns.push(PatternDefinition {
                    pattern_id,
                    strictness,
                    text: description_lines.join(" "),
                    location: SourceLocation {
                        path: path.to_path_buf(),
                        line: line_number,
                    },
                });
                index = cursor;
                continue;
            }

            let Some((kind, raw_decl)) = normalized_architecture_heading(raw) else {
                index += 1;
                continue;
            };

            let line_number = index + 1;
            let Some((id, inline_release, inline_planned)) =
                parse_module_header(kind, raw_decl, parsed, &path, line_number)
            else {
                index += 1;
                continue;
            };

            let mut cursor = skip_blank_doc_lines(&lines, index + 1);
            let (planned, planned_release, next_cursor) = maybe_consume_doc_planned(
                kind,
                &lines,
                cursor,
                parsed,
                &path,
                inline_planned,
                inline_release,
            );
            cursor = next_cursor;
            let (description_lines, cursor) = collect_doc_description_lines(&lines, cursor);
            let location = SourceLocation {
                path: path.to_path_buf(),
                line: line_number,
            };
            let plan = if planned {
                PlanState::planned(planned_release)
            } else {
                PlanState::current()
            };
            push_module_decl(
                parsed,
                kind,
                id,
                description_lines.join(" "),
                plan,
                location,
            );
            index = cursor;
        }
    }

    Ok(())
}

fn push_module_decl(
    parsed: &mut ParsedArchitecture,
    kind: ArchitectureKind,
    id: String,
    text: String,
    plan: PlanState,
    location: SourceLocation,
) {
    let module = match ModuleDecl::new(id, kind, text, plan, location.clone()) {
        Ok(module) => module,
        Err(err) => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: location.path,
                line: location.line,
                message: err.to_string(),
            });
            return;
        }
    };
    parsed.modules.push(module);
}

fn normalized_markdown_implements_annotation(line: &str) -> Option<(&str, bool, &'static str)> {
    let annotation = normalized_annotation_line(Some(line))?;
    if let Some(rest) =
        reserved_special_annotation_rest(annotation, ReservedSpecialAnnotation::Implements)
    {
        Some((rest, false, "@implements"))
    } else {
        reserved_special_annotation_rest(annotation, ReservedSpecialAnnotation::FileImplements)
            .map(|rest| (rest, true, "@fileimplements"))
    }
}

fn normalized_markdown_pattern_application(line: &str) -> Option<(&str, bool, &'static str)> {
    let annotation = normalized_annotation_line(Some(line))?;
    if let Some(rest) =
        reserved_special_annotation_rest(annotation, ReservedSpecialAnnotation::Applies)
    {
        Some((rest, false, "@applies"))
    } else {
        reserved_special_annotation_rest(annotation, ReservedSpecialAnnotation::FileApplies)
            .map(|rest| (rest, true, "@fileapplies"))
    }
}

#[derive(Debug, Clone, Copy)]
struct MarkdownSection {
    start: usize,
    end: usize,
}

fn markdown_implementation_section(
    lines: &[&str],
    annotation_index: usize,
) -> Option<MarkdownSection> {
    if let Some(next_heading) = next_heading_after_attachment_lines(lines, annotation_index + 1) {
        return markdown_section_at(lines, next_heading);
    }

    let heading_index = containing_markdown_heading(lines, annotation_index)?;
    markdown_section_at(lines, heading_index)
}

fn next_heading_after_attachment_lines(lines: &[&str], cursor: usize) -> Option<usize> {
    let mut cursor = skip_blank_doc_lines(lines, cursor);
    while cursor < lines.len() && is_markdown_architecture_attachment_line(lines[cursor]) {
        cursor = skip_blank_doc_lines(lines, cursor + 1);
    }
    (cursor < lines.len() && markdown_heading_level(lines[cursor]).is_some()).then_some(cursor)
}

fn containing_markdown_heading(lines: &[&str], annotation_index: usize) -> Option<usize> {
    let mut in_code_fence = false;
    let mut heading_index = None;
    for (index, line) in lines.iter().enumerate().take(annotation_index + 1) {
        if starts_markdown_fence(line) {
            in_code_fence = !in_code_fence;
            continue;
        }
        if !in_code_fence && markdown_heading_level(line).is_some() {
            heading_index = Some(index);
        }
    }
    heading_index
}

fn markdown_section_at(lines: &[&str], heading_index: usize) -> Option<MarkdownSection> {
    let level = markdown_heading_level(lines[heading_index])?;
    let mut in_code_fence = false;
    let mut end = lines.len();
    for (index, line) in lines.iter().enumerate().skip(heading_index + 1) {
        if starts_markdown_fence(line) {
            in_code_fence = !in_code_fence;
            continue;
        }
        if !in_code_fence && markdown_heading_level(line).is_some_and(|next| next <= level) {
            end = index;
            break;
        }
    }
    Some(MarkdownSection {
        start: heading_index,
        end,
    })
}

fn markdown_section_body(
    lines: &[&str],
    section: &MarkdownSection,
    annotation_index: usize,
) -> String {
    let mut in_code_fence = false;
    lines[section.start..section.end]
        .iter()
        .enumerate()
        .filter_map(|(offset, line)| {
            let index = section.start + offset;
            if starts_markdown_fence(line) {
                in_code_fence = !in_code_fence;
                return Some(*line);
            }
            (in_code_fence
                || (index != annotation_index && !is_markdown_architecture_attachment_line(line)))
            .then_some(*line)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn markdown_file_body(lines: &[&str]) -> String {
    let mut in_code_fence = false;
    lines
        .iter()
        .filter_map(|line| {
            if starts_markdown_fence(line) {
                in_code_fence = !in_code_fence;
                return Some(*line);
            }
            (in_code_fence || !is_markdown_architecture_attachment_line(line)).then_some(*line)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_markdown_architecture_attachment_line(line: &str) -> bool {
    normalized_markdown_implements_annotation(line).is_some()
        || normalized_markdown_pattern_application(line).is_some()
}

fn markdown_heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|char| *char == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    trimmed
        .chars()
        .nth(level)
        .is_some_and(char::is_whitespace)
        .then_some(level)
}
