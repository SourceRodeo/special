/**
@module SPECIAL.TESTS.PARSER.BLOCK
Parser block-routing and reserved-tag tests in `src/parser/tests/block.rs`.
*/
// @fileimplements SPECIAL.TESTS.PARSER.BLOCK
use super::support::{parse_current, source_block};

#[test]
// @verifies SPECIAL.PARSE.FOREIGN_TAGS_NOT_ERRORS
fn ignores_foreign_line_start_tags() {
    let block = source_block(&["@param file output path", "\\returns csv text"]);

    let parsed = parse_current(&block);

    assert!(parsed.specs.is_empty());
    assert!(parsed.verifies.is_empty());
    assert!(parsed.attests.is_empty());
    assert!(parsed.diagnostics.is_empty());
}

#[test]
// @verifies SPECIAL.PARSE.LINE_START_RESERVED_TAGS
fn ignores_reserved_tag_text_when_it_does_not_begin_the_line() {
    let block = source_block(&["Human note mentioning @spec EXPORT.CSV inline."]);

    let parsed = parse_current(&block);

    assert!(parsed.specs.is_empty());
    assert!(parsed.verifies.is_empty());
    assert!(parsed.attests.is_empty());
    assert!(parsed.diagnostics.is_empty());
}

#[test]
// @verifies SPECIAL.PARSE.RESERVED_TAGS.REQUIRE_DIRECTIVE_SHAPE
fn reports_malformed_reserved_tags_instead_of_treating_them_as_foreign() {
    let block = source_block(&[
        "@spec",
        "@verifies",
        "@attests",
        "@fileverifies",
        "@fileattests",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.diagnostics.len(), 5);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("missing spec id after @spec")
    );
    assert!(
        parsed.diagnostics[1]
            .message
            .contains("missing spec id after @verifies")
    );
    assert!(
        parsed.diagnostics[2]
            .message
            .contains("missing spec id after @attests")
    );
    assert!(
        parsed.diagnostics[3]
            .message
            .contains("missing spec id after @fileverifies")
    );
    assert!(
        parsed.diagnostics[4]
            .message
            .contains("missing spec id after @fileattests")
    );
}

#[test]
fn reserved_support_annotations_reject_trailing_tokens() {
    let block = source_block(&[
        "@verifies APP.ROOT trailing",
        "@attests APP.ROOT trailing",
        "@fileverifies APP.ROOT trailing",
        "@fileattests APP.ROOT trailing",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.diagnostics.len(), 4);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("unexpected trailing content after @verifies spec id")
    );
    assert!(
        parsed.diagnostics[1]
            .message
            .contains("unexpected trailing content after @attests spec id")
    );
    assert!(
        parsed.diagnostics[2]
            .message
            .contains("unexpected trailing content after @fileverifies spec id")
    );
    assert!(
        parsed.diagnostics[3]
            .message
            .contains("unexpected trailing content after @fileattests spec id")
    );
}

#[test]
// @verifies SPECIAL.PARSE.FOREIGN_TAG_BOUNDARIES
fn foreign_tag_lines_stop_attached_spec_text() {
    let block = source_block(&[
        "@spec EXPORT.CSV",
        "CSV exports include a header row.",
        "@param file output path",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert_eq!(parsed.specs[0].text, "CSV exports include a header row.");
    assert!(parsed.diagnostics.is_empty());
}

#[test]
// @verifies SPECIAL.PARSE.ARCH_ANNOTATIONS_RESERVED
fn ignores_reserved_architecture_annotations() {
    let block = source_block(&[
        "@implements SPECIAL.PARSER",
        "@module SPECIAL.PARSER.PLANNED",
        "@area SPECIAL.PARSER",
        "@fileimplements SPECIAL.PARSER",
    ]);

    let parsed = parse_current(&block);

    assert!(parsed.diagnostics.is_empty());
    assert!(parsed.specs.is_empty());
    assert!(parsed.verifies.is_empty());
    assert!(parsed.attests.is_empty());
}
