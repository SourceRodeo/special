/**
@module SPECIAL.TESTS.INDEX.MATERIALIZE
Index materialization tests in `src/index/tests/materialize.rs`.
*/
// @fileimplements SPECIAL.TESTS.INDEX.MATERIALIZE
use crate::model::NodeKind;

use super::support::{
    group_decl, materialize_all, materialize_current, materialize_unverified_current, parsed_repo,
    spec_decl, verify_ref,
};

#[test]
fn filters_planned_specs_with_current_filter() {
    let parsed = parsed_repo(
        vec![
            spec_decl("EXPORT", NodeKind::Spec, "Export root.", false, 1),
            spec_decl("EXPORT.METADATA", NodeKind::Spec, "Metadata.", true, 3),
        ],
        vec![verify_ref(
            "EXPORT",
            "src/lib.rs",
            8,
            "fn verifies_export() {}",
        )],
        Vec::new(),
    );

    let document = materialize_current(&parsed);

    assert_eq!(document.nodes.len(), 1);
    assert_eq!(document.nodes[0].id, "EXPORT");
}

#[test]
fn includes_planned_specs_by_default() {
    let parsed = parsed_repo(
        vec![
            spec_decl("EXPORT", NodeKind::Spec, "Export root.", false, 1),
            spec_decl("EXPORT.METADATA", NodeKind::Spec, "Metadata.", true, 3),
        ],
        vec![verify_ref(
            "EXPORT",
            "src/lib.rs",
            8,
            "fn verifies_export() {}",
        )],
        Vec::new(),
    );

    let document = materialize_all(&parsed);

    assert_eq!(document.nodes.len(), 1);
    assert_eq!(document.nodes[0].children.len(), 1);
    assert_eq!(document.nodes[0].children[0].id, "EXPORT.METADATA");
}

#[test]
fn filters_to_unverified_current_specs() {
    let parsed = parsed_repo(
        vec![
            spec_decl("EXPORT", NodeKind::Spec, "Export root.", false, 1),
            spec_decl("EXPORT.DOESNTCRASH", NodeKind::Spec, "No crash.", false, 3),
        ],
        vec![verify_ref(
            "EXPORT.DOESNTCRASH",
            "src/lib.rs",
            8,
            "fn verifies_no_crash() {}",
        )],
        Vec::new(),
    );

    let document = materialize_unverified_current(&parsed);

    assert_eq!(document.nodes.len(), 1);
    assert_eq!(document.nodes[0].id, "EXPORT");
    assert!(document.nodes[0].is_unverified());
}

#[test]
// @verifies SPECIAL.GROUPS.SPEC_MAY_HAVE_CHILDREN
fn materializes_nested_tree_structure() {
    let parsed = parsed_repo(
        vec![
            spec_decl("EXPORT", NodeKind::Spec, "Export root.", false, 1),
            spec_decl("EXPORT.DOESNTCRASH", NodeKind::Spec, "No crash.", false, 2),
        ],
        vec![verify_ref(
            "EXPORT.DOESNTCRASH",
            "tests/spec.rs",
            10,
            "fn verifies_export_doesnt_crash() {}",
        )],
        Vec::new(),
    );

    let document = materialize_current(&parsed);

    assert_eq!(document.nodes.len(), 1);
    assert_eq!(document.nodes[0].id, "EXPORT");
    assert!(document.nodes[0].is_unverified());
    assert_eq!(document.nodes[0].children.len(), 1);
    assert_eq!(document.nodes[0].children[0].id, "EXPORT.DOESNTCRASH");
    assert!(!document.nodes[0].children[0].is_unverified());
}

#[test]
// @verifies SPECIAL.GROUPS.STRUCTURAL_ONLY
fn materializes_groups_without_marking_them_unsupported() {
    let parsed = parsed_repo(
        vec![
            group_decl("SPECIAL", "Top-level grouping.", "specs/special.rs", 1),
            spec_decl(
                "SPECIAL.PARSE",
                NodeKind::Spec,
                "Parses annotated blocks.",
                false,
                3,
            ),
        ],
        vec![verify_ref(
            "SPECIAL.PARSE",
            "tests/cli.rs",
            10,
            "fn verifies_special_parse() {}",
        )],
        Vec::new(),
    );

    let document = materialize_current(&parsed);

    assert_eq!(document.nodes.len(), 1);
    assert_eq!(document.nodes[0].kind(), NodeKind::Group);
    assert!(!document.nodes[0].is_unverified());
}
