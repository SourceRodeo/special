/**
@module SPECIAL.PARSER.BLOCK.ATTESTS
Handles `@attests` and `@fileattests` parsing within a source comment block.
*/
// @fileimplements SPECIAL.PARSER.BLOCK.ATTESTS
use std::fs;

use crate::annotation_syntax::{ReservedSpecialAnnotation, reserved_special_annotation_rest};
use crate::model::{AttestRef, AttestScope, CommentBlock, ParsedRepo, SourceLocation};

use super::super::attestation::parse_attestation_metadata;
use super::parse_supported_spec_id;

pub(super) fn handle_attest_line(
    block: &CommentBlock,
    parsed: &mut ParsedRepo,
    index: usize,
    line: usize,
    trimmed: &str,
) -> Option<usize> {
    if let Some(rest) =
        reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Attests)
    {
        return Some(handle_block_attest(block, parsed, index, line, rest));
    }

    if let Some(rest) =
        reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::FileAttests)
    {
        return Some(handle_file_attest(block, parsed, index, line, rest));
    }

    None
}

fn handle_block_attest(
    block: &CommentBlock,
    parsed: &mut ParsedRepo,
    index: usize,
    line: usize,
    rest: &str,
) -> usize {
    let Some(id) = parse_supported_spec_id(rest, parsed, block, line, "@attests") else {
        return index + 1;
    };

    let (attestation, cursor) = parse_attestation_metadata(parsed, block, line, index + 1);
    if let Some(attestation) = attestation {
        parsed.attests.push(AttestRef {
            spec_id: id,
            artifact: attestation.artifact,
            owner: attestation.owner,
            last_reviewed: attestation.last_reviewed,
            review_interval_days: attestation.review_interval_days,
            scope: AttestScope::Block,
            location: SourceLocation {
                path: block.path.clone(),
                line,
            },
            body: Some(
                block
                    .lines
                    .iter()
                    .map(|line| line.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_string(),
            ),
        });
    }

    cursor
}

fn handle_file_attest(
    block: &CommentBlock,
    parsed: &mut ParsedRepo,
    index: usize,
    line: usize,
    rest: &str,
) -> usize {
    let Some(id) = parse_supported_spec_id(rest, parsed, block, line, "@fileattests") else {
        return index + 1;
    };

    let (attestation, cursor) = parse_attestation_metadata(parsed, block, line, index + 1);
    if let Some(attestation) = attestation {
        let body = fs::read_to_string(&block.path)
            .ok()
            .map(|body| body.trim_end().to_string());
        parsed.attests.push(AttestRef {
            spec_id: id,
            artifact: attestation.artifact,
            owner: attestation.owner,
            last_reviewed: attestation.last_reviewed,
            review_interval_days: attestation.review_interval_days,
            scope: AttestScope::File,
            location: SourceLocation {
                path: block.path.clone(),
                line,
            },
            body,
        });
    }

    cursor
}
