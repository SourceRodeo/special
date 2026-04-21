/**
@module SPECIAL.TESTS.PARSER.ATTESTATION
Parser attestation tests in `src/parser/tests/attestation.rs`.
*/
// @fileimplements SPECIAL.TESTS.PARSER.ATTESTATION
use crate::model::AttestScope;

use super::support::{TempFile, block_with_path, parse_current, source_block};

#[test]
// @verifies SPECIAL.PARSE.ATTESTS
fn parses_attestation_blocks() {
    let block = source_block(&[
        "@attests AUTH",
        "artifact: docs/auth.md",
        "owner: security",
        "last_reviewed: 2026-04-12",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.attests.len(), 1);
    assert_eq!(parsed.attests[0].scope, AttestScope::Block);
    assert!(
        parsed.attests[0]
            .body
            .as_deref()
            .expect("attest body should be present")
            .contains("@attests AUTH")
    );
    assert!(parsed.diagnostics.is_empty());
}

#[test]
// @verifies SPECIAL.PARSE.ATTESTS.REQUIRED_FIELDS
fn requires_attestation_metadata() {
    let block = source_block(&["@attests AUTH", "artifact:"]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.attests.len(), 0);
    assert_eq!(parsed.diagnostics.len(), 3);
    let messages: Vec<&str> = parsed
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    let lines: Vec<usize> = parsed
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.line)
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("missing required attestation metadata `artifact`"))
    );
    assert!(messages.iter().any(|message| {
        message.contains("missing required attestation metadata `last_reviewed`")
    }));
    assert!(
        messages
            .iter()
            .any(|message| message.contains("missing required attestation metadata `owner`"))
    );
    assert_eq!(lines, vec![2, 1, 1]);
}

#[test]
// @verifies SPECIAL.PARSE.ATTESTS.FILE_SCOPE
fn parses_file_scoped_attestation_blocks() {
    let content = "// @fileattests AUTH\n// artifact: docs/auth-review.md\n// owner: security\n// last_reviewed: 2026-04-16\nfn helper() {}\n";
    let fixture = TempFile::new("special-parser-file-attests", "rs", content);
    let block = block_with_path(
        fixture.path().to_path_buf(),
        &[
            "@fileattests AUTH",
            "artifact: docs/auth-review.md",
            "owner: security",
            "last_reviewed: 2026-04-16",
        ],
    );

    let parsed = parse_current(&block);

    assert_eq!(parsed.attests.len(), 1);
    assert_eq!(parsed.attests[0].scope, AttestScope::File);
    assert!(
        parsed.attests[0]
            .body
            .as_deref()
            .expect("attest body should be present")
            .contains("fn helper() {}")
    );
    assert!(parsed.diagnostics.is_empty());
}

#[test]
fn rejects_duplicate_attestation_metadata_keys() {
    let block = source_block(&[
        "@attests EXPORT.CSV_HEADER",
        "artifact: cargo test",
        "artifact: cargo test --release",
        "owner: qa@example.com",
        "last_reviewed: 2026-04-13",
    ]);

    let parsed = parse_current(&block);

    assert!(parsed.attests.is_empty());
    assert!(parsed.diagnostics.iter().any(|diag| {
        diag.message
            .contains("duplicate attestation metadata `artifact`")
    }));
}

#[test]
// @verifies SPECIAL.PARSE.ATTESTS.ALLOWED_FIELDS
fn rejects_unknown_attestation_metadata_keys() {
    let block = source_block(&[
        "@attests AUTH",
        "artifact: docs/auth.md",
        "owner: security",
        "last_reviewed: 2026-04-12",
        "reviewed_by: qa",
    ]);

    let parsed = parse_current(&block);

    assert!(parsed.attests.is_empty());
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].line, 5);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("unknown attestation metadata `reviewed_by`")
    );
}

#[test]
// @verifies SPECIAL.PARSE.ATTESTS.DATE_FORMAT
fn requires_attestation_dates_in_iso_format() {
    let block = source_block(&[
        "@attests AUTH",
        "artifact: docs/auth.md",
        "owner: security",
        "last_reviewed: 04-12-2026",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.attests.len(), 0);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].line, 4);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("last_reviewed must use YYYY-MM-DD format")
    );
}

#[test]
// @verifies SPECIAL.PARSE.ATTESTS.REVIEW_INTERVAL_DAYS
fn requires_numeric_attestation_review_interval() {
    let block = source_block(&[
        "@attests AUTH",
        "artifact: docs/auth.md",
        "owner: security",
        "last_reviewed: 2026-04-12",
        "review_interval_days: thirty",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.attests.len(), 0);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].line, 5);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("review_interval_days must be a positive integer")
    );
}

#[test]
// @verifies SPECIAL.PARSE.ATTESTS.REVIEW_INTERVAL_DAYS
fn requires_positive_attestation_review_interval() {
    let block = source_block(&[
        "@attests AUTH",
        "artifact: docs/auth.md",
        "owner: security",
        "last_reviewed: 2026-04-12",
        "review_interval_days: 0",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.attests.len(), 0);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].line, 5);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("review_interval_days must be a positive integer")
    );
}
