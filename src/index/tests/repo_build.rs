/**
@module SPECIAL.TESTS.INDEX.REPO_BUILD
Index end-to-end repo-build tests in `src/index/tests/repo_build.rs`.
*/
// @fileimplements SPECIAL.TESTS.INDEX.REPO_BUILD
use crate::model::NodeKind;

use super::support::TempRepo;

#[test]
// @verifies SPECIAL.GROUPS
fn builds_spec_document_from_repo_files() {
    let repo = TempRepo::new("special-self-host");
    repo.write(
        "spec.rs",
        r#"/**
@group DEMO
Demo root.

@spec DEMO.OK
Demo child.
*/
"#,
    );
    repo.write(
        "tests.rs",
        &[
            "/",
            "/ @verifies DEMO.OK\n",
            "fn verifies_demo_child() {}\n",
        ]
        .concat(),
    );

    let (document, lint) = repo.build_current_document();

    assert!(lint.diagnostics.is_empty());
    assert_eq!(document.nodes.len(), 1);
    assert_eq!(document.nodes[0].id, "DEMO");
    assert_eq!(document.nodes[0].kind(), NodeKind::Group);
    assert_eq!(document.nodes[0].verifies.len(), 0);
    assert_eq!(document.nodes[0].children.len(), 1);
}

#[test]
// @verifies SPECIAL.PARSE.MULTI_FILE_TREE
fn builds_one_tree_from_multiple_files() {
    let repo = TempRepo::new("special-multi-file-tree");
    repo.write(
        "specs/root.rs",
        r#"/**
@group DEMO
Demo root.
*/
"#,
    );
    repo.write(
        "specs/child.rs",
        r#"/**
@spec DEMO.CHILD
Child claim.
*/
"#,
    );
    repo.write(
        "checks.rs",
        &[
            "/",
            "/ @verifies DEMO.CHILD\n",
            "fn verifies_demo_child() {}\n",
        ]
        .concat(),
    );

    let (document, lint) = repo.build_current_document();

    assert!(lint.diagnostics.is_empty());
    assert_eq!(document.nodes.len(), 1);
    assert_eq!(document.nodes[0].id, "DEMO");
    assert_eq!(document.nodes[0].children.len(), 1);
    assert_eq!(document.nodes[0].children[0].id, "DEMO.CHILD");
}

#[test]
// @verifies SPECIAL.PARSE.MULTI_FILE_TREE.MIXED_FILE_TYPES
fn builds_one_tree_from_mixed_supported_file_types() {
    let repo = TempRepo::new("special-mixed-file-tree");
    repo.write(
        "specs/root.rs",
        r#"/**
@group DEMO
Demo root.
*/
"#,
    );
    repo.write("specs/child.sh", "# @spec DEMO.CHILD\n# Child claim.\n");
    repo.write(
        "checks.rs",
        &[
            "/",
            "/ @verifies DEMO.CHILD\n",
            "fn verifies_demo_child() {}\n",
        ]
        .concat(),
    );

    let (document, lint) = repo.build_current_document();

    assert!(lint.diagnostics.is_empty());
    assert_eq!(document.nodes.len(), 1);
    assert_eq!(document.nodes[0].id, "DEMO");
    assert_eq!(document.nodes[0].children.len(), 1);
    assert_eq!(document.nodes[0].children[0].id, "DEMO.CHILD");
}
