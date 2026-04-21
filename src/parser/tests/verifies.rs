/**
@module SPECIAL.TESTS.PARSER.VERIFIES
Parser verify-reference tests in `src/parser/tests/verifies.rs`.
*/
// @fileimplements SPECIAL.TESTS.PARSER.VERIFIES
use super::support::{TempFile, block_with_path, parse_current, source_block_with_owned_item};

#[test]
// @verifies SPECIAL.PARSE.VERIFIES
fn parses_single_verify_reference() {
    let block = source_block_with_owned_item(
        &["@verifies EXPORT.DOESNTCRASH"],
        2,
        "fn verifies_export_doesnt_crash() {}",
    );

    let parsed = parse_current(&block);

    assert_eq!(parsed.verifies.len(), 1);
    assert_eq!(parsed.verifies[0].spec_id, "EXPORT.DOESNTCRASH");
    assert_eq!(
        parsed.verifies[0].body.as_deref(),
        Some("fn verifies_export_doesnt_crash() {}")
    );
    assert!(parsed.diagnostics.is_empty());
}

#[test]
// @verifies SPECIAL.PARSE.VERIFIES.FILE_SCOPE
fn parses_file_verify_reference() {
    let content = "// @fileverifies EXPORT.DOESNTCRASH\nfn verifies_export_doesnt_crash() {}\n";
    let fixture = TempFile::new("special-fileverify", "rs", content);
    let block = block_with_path(
        fixture.path().to_path_buf(),
        &["@fileverifies EXPORT.DOESNTCRASH"],
    );

    let parsed = parse_current(&block);

    assert_eq!(parsed.verifies.len(), 1);
    assert_eq!(parsed.verifies[0].spec_id, "EXPORT.DOESNTCRASH");
    assert!(parsed.verifies[0].body_location.is_none());
    assert_eq!(parsed.verifies[0].body.as_deref(), Some(content.trim_end()));
    assert!(parsed.diagnostics.is_empty());
}

#[test]
// @verifies SPECIAL.PARSE.VERIFIES.ONE_PER_BLOCK
fn rejects_multiple_verifies_in_one_block() {
    let block = source_block_with_owned_item(
        &["@verifies AUTH.ONE", "@verifies AUTH.TWO"],
        3,
        "fn verifies_auth() {}",
    );

    let parsed = parse_current(&block);

    assert_eq!(parsed.verifies.len(), 1);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(parsed.diagnostics[0].message.contains("only one @verifies"));
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.ORPHAN_VERIFIES
fn rejects_orphan_verifies_blocks() {
    let block = block_with_path("src/example.rs", &["@verifies EXPORT.ORPHAN"]);

    let parsed = parse_current(&block);

    assert!(parsed.verifies.is_empty());
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("@verifies must attach to the next supported item")
    );
}
