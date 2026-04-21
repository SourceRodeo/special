/**
@module SPECIAL.TESTS.EXTRACTOR.MIXED_PURPOSE
Extractor mixed-purpose and foreign-tag tests in `src/extractor/tests/mixed_purpose.rs`.
*/
// @fileimplements SPECIAL.TESTS.EXTRACTOR.MIXED_PURPOSE
use super::support::extract;

#[test]
// @verifies SPECIAL.PARSE.LINE_START_RESERVED_TAGS
fn ignores_mid_line_tag_text_when_collecting_blocks() {
    let blocks = extract(
        "src/example.rs",
        "/// Human prose mentioning @spec EXPORT.CSV inline.\nfn export_csv() {}\n",
    );

    assert!(blocks.is_empty());
}

#[test]
// @verifies SPECIAL.PARSE.MIXED_PURPOSE_COMMENTS
fn extracts_special_tags_from_ordinary_doc_comment_blocks() {
    let blocks = extract(
        "src/example.rs",
        "/// Human overview for maintainers.\n/// @spec EXPORT.CSV\n/// CSV exports include a header row.\nfn export_csv() {}\n",
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].lines[0].text, "Human overview for maintainers.");
    assert_eq!(blocks[0].lines[1].text, "@spec EXPORT.CSV");
    assert_eq!(blocks[0].lines[2].text, "CSV exports include a header row.");
    assert_eq!(
        blocks[0]
            .owned_item
            .as_ref()
            .expect("owned item should be present")
            .body,
        "fn export_csv() {}"
    );
}

#[test]
// @verifies SPECIAL.PARSE.FOREIGN_TAGS_NOT_ERRORS
fn ignores_comment_blocks_with_only_foreign_tags() {
    let blocks = extract(
        "src/example.ts",
        "/**\n * @param file output path\n * @returns CSV text\n */\nexport function render() {}\n",
    );

    assert!(blocks.is_empty());
}
