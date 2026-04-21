/**
@module SPECIAL.TESTS.INDEX.LINT
Index lint tests in `src/index/tests/lint.rs`.
*/
// @fileimplements SPECIAL.TESTS.INDEX.LINT
use crate::model::NodeKind;

use super::support::{
    block_attest_ref, file_attest_ref, group_decl, lint, parsed_repo, spec_decl, verify_ref,
};

#[test]
// @verifies SPECIAL.LINT_COMMAND.INTERMEDIATE_SPECS
fn reports_missing_intermediate_specs() {
    let parsed = parsed_repo(
        vec![spec_decl(
            "EXPORT.DOESNTCRASH",
            NodeKind::Spec,
            "No crash.",
            false,
            1,
        )],
        Vec::new(),
        Vec::new(),
    );

    let lint = lint(&parsed);
    assert_eq!(lint.diagnostics.len(), 1);
    assert!(
        lint.diagnostics[0]
            .message
            .contains("missing intermediate spec `EXPORT`")
    );
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.UNKNOWN_VERIFY_REFS
fn reports_unknown_verify_refs() {
    let parsed = parsed_repo(
        vec![spec_decl(
            "EXPORT",
            NodeKind::Spec,
            "Export root.",
            false,
            1,
        )],
        vec![verify_ref(
            "UNKNOWN",
            "tests/spec.rs",
            10,
            "fn verifies_unknown() {}",
        )],
        Vec::new(),
    );

    let lint = lint(&parsed);
    assert_eq!(lint.diagnostics.len(), 1);
    assert!(
        lint.diagnostics[0]
            .message
            .contains("unknown spec id `UNKNOWN` referenced by @verifies")
    );
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.UNKNOWN_ATTEST_REFS
fn reports_unknown_attest_refs() {
    let parsed = parsed_repo(
        vec![spec_decl(
            "EXPORT",
            NodeKind::Spec,
            "Export root.",
            false,
            1,
        )],
        Vec::new(),
        vec![
            block_attest_ref("UNKNOWN", "docs/report.txt", 10, "@attests UNKNOWN"),
            file_attest_ref("ALSO_UNKNOWN", "docs/review.md", 1, "# Review"),
        ],
    );

    let lint = lint(&parsed);
    assert_eq!(lint.diagnostics.len(), 2);
    assert!(
        lint.diagnostics[0]
            .message
            .contains("unknown spec id `UNKNOWN` referenced by @attests")
    );
    assert!(
        lint.diagnostics[1]
            .message
            .contains("unknown spec id `ALSO_UNKNOWN` referenced by @fileattests")
    );
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.DUPLICATE_IDS
fn reports_duplicate_spec_ids() {
    let parsed = parsed_repo(
        vec![
            spec_decl("EXPORT", NodeKind::Spec, "Export root.", false, 1),
            spec_decl(
                "EXPORT",
                NodeKind::Spec,
                "Duplicate export root.",
                false,
                20,
            ),
        ],
        Vec::new(),
        Vec::new(),
    );

    let lint = lint(&parsed);
    assert_eq!(lint.diagnostics.len(), 1);
    assert!(
        lint.diagnostics[0]
            .message
            .contains("duplicate node id `EXPORT`")
    );
    assert!(lint.diagnostics[0].message.contains("src/lib.rs:1"));
}

#[test]
// @verifies SPECIAL.GROUPS.MUTUALLY_EXCLUSIVE
fn rejects_duplicate_ids_across_spec_and_group_kinds() {
    let parsed = parsed_repo(
        vec![
            group_decl("EXPORT", "Export grouping.", "src/lib.rs", 1),
            spec_decl("EXPORT", NodeKind::Spec, "Export claim.", false, 5),
        ],
        Vec::new(),
        Vec::new(),
    );

    let lint = lint(&parsed);
    assert_eq!(lint.diagnostics.len(), 1);
    assert!(
        lint.diagnostics[0]
            .message
            .contains("duplicate node id `EXPORT`")
    );
    assert!(lint.diagnostics[0].message.contains("@group"));
    assert!(lint.diagnostics[0].message.contains("src/lib.rs:1"));
}
