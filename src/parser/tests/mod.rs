/**
@module SPECIAL.TESTS.PARSER
Top-level parser integration and shared test-tree wiring for parser behavior in `src/parser/tests/`.
*/
// @fileimplements SPECIAL.TESTS.PARSER
mod attestation;
mod block;
mod declarations;
mod support;
mod verifies;

use crate::model::NodeKind;

use self::support::{parse_current, source_block};

#[test]
fn normalize_markdown_annotation_line_preserves_inline_code_at_line_start() {
    assert_eq!(
        super::normalize_markdown_annotation_line(
            "`paypal config` manages `paypal.env.yaml` against linked remote apps."
        ),
        Some("`paypal config` manages `paypal.env.yaml` against linked remote apps.")
    );
}

#[test]
fn normalize_markdown_annotation_line_treats_whole_line_code_span_as_literal() {
    assert_eq!(
        super::normalize_markdown_annotation_line("`@spec DEMO.CMD`"),
        None
    );
}

#[test]
// @verifies SPECIAL.PARSE
fn parses_mixed_annotation_kinds() {
    let block = source_block(&["@spec EXPORT", "@planned", "Exports data."]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert_eq!(parsed.specs[0].id, "EXPORT");
    assert_eq!(parsed.specs[0].kind(), NodeKind::Spec);
    assert!(parsed.specs[0].is_planned());
    assert_eq!(parsed.specs[0].text, "Exports data.");
    assert!(parsed.verifies.is_empty());
    assert!(parsed.attests.is_empty());
    assert!(parsed.diagnostics.is_empty());
}

#[test]
// @verifies SPECIAL.PARSE.MIXED_PURPOSE_COMMENTS
fn parses_reserved_annotations_from_mixed_purpose_comment_blocks() {
    let block = source_block(&[
        "Human overview for maintainers.",
        "@spec EXPORT.CSV",
        "CSV exports include a header row.",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert_eq!(parsed.specs[0].id, "EXPORT.CSV");
    assert_eq!(parsed.specs[0].text, "CSV exports include a header row.");
    assert!(parsed.diagnostics.is_empty());
}
