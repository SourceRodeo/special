/**
@module SPECIAL.MODULES.ANALYZE.PROSE
Finds long natural-language prose blocks outside configured docs sources so health can surface uncaptured explanatory text as a broad repo signal.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.PROSE
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::annotation_syntax::{
    is_reserved_special_annotation, normalize_markdown_declaration_line,
};
use crate::config::DocsOutputConfig;
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{ArchitectureLongProseBlock, ArchitectureRepoSignalsSummary};
use crate::source_paths::normalize_existing_or_joined_path;

const MIN_PROSE_WORDS: usize = 35;
const MIN_PROSE_SENTENCES: usize = 2;
const MIN_PROSE_SCORE: f64 = 0.62;

pub(crate) fn apply_long_prose_outside_docs_summary(
    root: &Path,
    ignore_patterns: &[String],
    docs_outputs: &[DocsOutputConfig],
    scope_paths: Option<&[PathBuf]>,
    summary: &mut ArchitectureRepoSignalsSummary,
) -> Result<()> {
    let discovered = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?;
    let docs_paths = docs_scope_paths(root, docs_outputs);
    let mut findings = Vec::new();

    for path in discovered
        .source_files
        .iter()
        .chain(discovered.markdown_files.iter())
    {
        if scope_paths.is_some_and(|scopes| !path_matches_any(path, scopes)) {
            continue;
        }
        if path_matches_any(path, &docs_paths) {
            continue;
        }
        let text = std::fs::read_to_string(path)?;
        if markdown_path(path) {
            findings.extend(markdown_prose_blocks(root, path, &text));
        } else {
            findings.extend(source_comment_prose_blocks(root, path, &text));
        }
    }

    findings.sort_by(|left, right| {
        right
            .prose_score
            .partial_cmp(&left.prose_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.word_count.cmp(&left.word_count))
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.line.cmp(&right.line))
    });
    summary.long_prose_outside_docs = findings.len();
    summary.long_prose_outside_docs_details = findings;
    Ok(())
}

fn docs_scope_paths(root: &Path, docs_outputs: &[DocsOutputConfig]) -> Vec<PathBuf> {
    docs_outputs
        .iter()
        .flat_map(|output| [&output.source, &output.output])
        .map(|path| normalize_existing_or_joined_path(root, path))
        .collect()
}

fn path_matches_any(path: &Path, scopes: &[PathBuf]) -> bool {
    scopes
        .iter()
        .any(|scope| path == scope || path.starts_with(scope))
}

fn markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
}

fn markdown_prose_blocks(root: &Path, path: &Path, text: &str) -> Vec<ArchitectureLongProseBlock> {
    let mut findings = Vec::new();
    let mut in_fence = false;
    let mut in_annotation_body = false;
    let mut block = Vec::<(usize, &str)>::new();

    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            flush_block(root, path, &mut block, &mut findings);
            in_annotation_body = false;
            in_fence = !in_fence;
            continue;
        }
        if trimmed.is_empty() {
            flush_block(root, path, &mut block, &mut findings);
            in_annotation_body = false;
            continue;
        }
        if markdown_special_annotation_line(trimmed) {
            flush_block(root, path, &mut block, &mut findings);
            in_annotation_body = true;
            continue;
        }
        if in_fence || markdown_boundary(trimmed) {
            flush_block(root, path, &mut block, &mut findings);
            in_annotation_body = false;
            continue;
        }
        if in_annotation_body {
            continue;
        } else {
            block.push((line_number, trimmed));
        }
    }
    flush_block(root, path, &mut block, &mut findings);
    findings
}

fn markdown_boundary(trimmed: &str) -> bool {
    trimmed.starts_with('#')
        || trimmed.starts_with('@')
        || trimmed.starts_with('|')
        || trimmed.starts_with('-')
        || trimmed.starts_with('*')
        || trimmed.starts_with('>')
        || ordered_list_marker(trimmed)
}

fn ordered_list_marker(trimmed: &str) -> bool {
    let Some((digits, rest)) = trimmed.split_once('.') else {
        return false;
    };
    !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit()) && rest.starts_with(' ')
}

fn source_comment_prose_blocks(
    root: &Path,
    path: &Path,
    text: &str,
) -> Vec<ArchitectureLongProseBlock> {
    let mut findings = Vec::new();
    let mut block = Vec::<(usize, String)>::new();
    let mut in_block_comment = false;

    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();
        if in_block_comment {
            let (comment, closed) = strip_block_comment_line(trimmed);
            if !comment.trim().is_empty() {
                block.push((line_number, comment));
            }
            if closed {
                in_block_comment = false;
                flush_owned_block(root, path, &mut block, &mut findings);
            }
            continue;
        }
        if let Some(comment) = line_comment_text(trimmed) {
            block.push((line_number, comment.to_string()));
            continue;
        }
        if let Some(after_start) = trimmed.strip_prefix("/*") {
            flush_owned_block(root, path, &mut block, &mut findings);
            let (comment, closed) = strip_block_comment_line(after_start);
            if !comment.trim().is_empty() {
                block.push((line_number, comment));
            }
            if closed {
                flush_owned_block(root, path, &mut block, &mut findings);
            } else {
                in_block_comment = true;
            }
            continue;
        }
        flush_owned_block(root, path, &mut block, &mut findings);
    }
    flush_owned_block(root, path, &mut block, &mut findings);
    findings
}

fn line_comment_text(trimmed: &str) -> Option<&str> {
    trimmed
        .strip_prefix("///")
        .or_else(|| trimmed.strip_prefix("//!"))
        .or_else(|| trimmed.strip_prefix("//"))
        .or_else(|| trimmed.strip_prefix('#'))
        .map(str::trim)
}

fn strip_block_comment_line(line: &str) -> (String, bool) {
    if let Some((comment, _rest)) = line.split_once("*/") {
        (comment.trim_start_matches('*').trim().to_string(), true)
    } else {
        (line.trim_start_matches('*').trim().to_string(), false)
    }
}

fn flush_owned_block(
    root: &Path,
    path: &Path,
    block: &mut Vec<(usize, String)>,
    findings: &mut Vec<ArchitectureLongProseBlock>,
) {
    let borrowed = block
        .iter()
        .map(|(line, text)| (*line, text.as_str()))
        .collect::<Vec<_>>();
    flush_borrowed_block(root, path, &borrowed, findings);
    block.clear();
}

fn flush_block(
    root: &Path,
    path: &Path,
    block: &mut Vec<(usize, &str)>,
    findings: &mut Vec<ArchitectureLongProseBlock>,
) {
    flush_borrowed_block(root, path, block, findings);
    block.clear();
}

fn flush_borrowed_block(
    root: &Path,
    path: &Path,
    block: &[(usize, &str)],
    findings: &mut Vec<ArchitectureLongProseBlock>,
) {
    if block.is_empty() {
        return;
    }
    if carries_docs_or_special_evidence(block) {
        return;
    }
    let text = block
        .iter()
        .map(|(_, line)| line.trim())
        .collect::<Vec<_>>()
        .join(" ");
    if let Some(finding) = score_prose_block(root, path, block[0].0, &text) {
        findings.push(finding);
    }
}

fn carries_docs_or_special_evidence(block: &[(usize, &str)]) -> bool {
    block
        .iter()
        .any(|(_, line)| carries_docs_evidence(line) || is_reserved_special_annotation(line.trim()))
        || carries_docs_evidence(
            &block
                .iter()
                .map(|(_, line)| line.trim())
                .collect::<Vec<_>>()
                .join(" "),
        )
}

fn carries_docs_evidence(text: &str) -> bool {
    text.contains("documents://") || text.contains("@documents") || text.contains("@filedocuments")
}

fn markdown_special_annotation_line(trimmed: &str) -> bool {
    normalize_markdown_declaration_line(trimmed).is_some_and(is_reserved_special_annotation)
}

fn score_prose_block(
    root: &Path,
    path: &Path,
    line: usize,
    text: &str,
) -> Option<ArchitectureLongProseBlock> {
    let words = prose_words(text);
    let word_count = words.len();
    if word_count < MIN_PROSE_WORDS {
        return None;
    }
    let sentence_count = text
        .chars()
        .filter(|ch| matches!(ch, '.' | '!' | '?'))
        .count()
        .max(1);
    if sentence_count < MIN_PROSE_SENTENCES {
        return None;
    }
    let alphabetic_chars = text.chars().filter(|ch| ch.is_alphabetic()).count();
    let visible_chars = text.chars().filter(|ch| !ch.is_whitespace()).count().max(1);
    let alphabetic_ratio = alphabetic_chars as f64 / visible_chars as f64;
    let average_word_length =
        words.iter().map(|word| word.len()).sum::<usize>() as f64 / word_count as f64;
    let sentence_density = (sentence_count as f64 / (word_count as f64 / 25.0)).min(1.0);
    let length_score = (word_count as f64 / 120.0).min(1.0);
    let word_shape_score = if (3.5..=8.5).contains(&average_word_length) {
        1.0
    } else {
        0.45
    };
    let prose_score = round_score(
        0.40 * length_score
            + 0.25 * sentence_density
            + 0.25 * alphabetic_ratio
            + 0.10 * word_shape_score,
    );
    (prose_score >= MIN_PROSE_SCORE).then(|| ArchitectureLongProseBlock {
        path: super::display_path(root, path),
        line,
        word_count,
        sentence_count,
        prose_score,
        preview: preview(text),
    })
}

fn prose_words(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphabetic() && ch != '\'')
        .filter(|word| word.chars().filter(|ch| ch.is_alphabetic()).count() >= 2)
        .map(str::to_ascii_lowercase)
        .collect()
}

fn preview(text: &str) -> String {
    const MAX_PREVIEW: usize = 160;
    if text.len() <= MAX_PREVIEW {
        return text.to_string();
    }
    let mut end = MAX_PREVIEW;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", text[..end].trim_end())
}

fn round_score(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}
