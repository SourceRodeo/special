/**
@module SPECIAL.TESTS.EXTRACTOR.OWNERSHIP
Extractor ownership-attachment tests in `src/extractor/tests/ownership.rs`.
*/
// @fileimplements SPECIAL.TESTS.EXTRACTOR.OWNERSHIP
use super::support::extract;

#[test]
// @verifies SPECIAL.PARSE.VERIFIES.ATTACHES_TO_NEXT_ITEM
fn attaches_verify_block_to_next_item() {
    let blocks = extract(
        "src/example.rs",
        "// @verifies AUTH.LOGIN\n#[test]\nfn verifies_auth_login() {\n    assert!(true);\n}\n",
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(
        blocks[0]
            .owned_item
            .as_ref()
            .expect("owned item should be present")
            .body,
        "#[test]\nfn verifies_auth_login() {\n    assert!(true);\n}"
    );
}
