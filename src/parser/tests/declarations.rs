/**
@module SPECIAL.TESTS.PARSER.DECLARATIONS
Parser declaration and lifecycle tests in `src/parser/tests/declarations.rs`.
*/
// @fileimplements SPECIAL.TESTS.PARSER.DECLARATIONS
use std::path::PathBuf;

use crate::config::SpecialVersion;
use crate::model::NodeKind;

use super::support::{parse_current, parse_with_version, source_block};

#[test]
// @verifies SPECIAL.PARSE.PLANNED
fn records_planned_on_the_owning_spec_for_each_supported_version() {
    let legacy = source_block(&[
        "@spec EXPORT.LEGACY",
        "Legacy planned behavior.",
        "@planned",
    ]);
    let current = source_block(&[
        "@spec EXPORT.CURRENT",
        "@planned",
        "Current planned behavior.",
    ]);

    let legacy_parsed = parse_with_version(&legacy, SpecialVersion::V0);
    let current_parsed = parse_current(&current);

    assert!(legacy_parsed.specs[0].is_planned());
    assert!(current_parsed.specs[0].is_planned());
    assert_eq!(legacy_parsed.specs[0].planned_release(), None);
    assert_eq!(current_parsed.specs[0].planned_release(), None);
}

#[test]
// @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1
fn version_1_requires_local_planned_ownership() {
    let accepted = source_block(&[
        "@spec EXPORT.CURRENT",
        "@planned",
        "Current planned behavior.",
    ]);
    let rejected = source_block(&["@spec EXPORT.OLD", "Old planned behavior.", "@planned"]);

    let accepted_parsed = parse_current(&accepted);
    let rejected_parsed = parse_current(&rejected);

    assert!(accepted_parsed.specs[0].is_planned());
    assert!(accepted_parsed.diagnostics.is_empty());
    assert!(!rejected_parsed.specs[0].is_planned());
    assert_eq!(rejected_parsed.diagnostics.len(), 1);
}

#[test]
// @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.INLINE
fn parses_inline_planned_in_version_1() {
    let block = source_block(&[
        "@spec EXPORT.METADATA @planned",
        "Exports include provenance metadata.",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert_eq!(parsed.specs[0].kind(), NodeKind::Spec);
    assert!(parsed.specs[0].is_planned());
    assert_eq!(parsed.specs[0].planned_release(), None);
    assert_eq!(parsed.specs[0].text, "Exports include provenance metadata.");
    assert!(parsed.diagnostics.is_empty());
}

#[test]
// @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.EXACT_INLINE_MARKER
fn rejects_fuzzy_inline_planned_markers_in_version_1() {
    let block = source_block(&["@spec EXPORT.METADATA @plannedness"]);

    let parsed = parse_current(&block);

    assert!(parsed.specs.is_empty());
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].path, PathBuf::from("src/example.rs"));
    assert_eq!(parsed.diagnostics[0].line, 1);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("unexpected trailing content after spec id")
    );
}

#[test]
// @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.EXACT_STANDALONE_MARKER
fn rejects_fuzzy_standalone_planned_markers_in_version_1() {
    let block = source_block(&[
        "@spec EXPORT.METADATA",
        "@plannedness",
        "Exports include provenance metadata.",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert!(!parsed.specs[0].is_planned());
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("exact standalone `@planned` marker")
    );
}

#[test]
// @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.NEXT_LINE
fn parses_adjacent_next_line_planned_in_version_1() {
    let block = source_block(&[
        "@spec EXPORT.METADATA",
        "@planned",
        "Exports include provenance metadata.",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert!(parsed.specs[0].is_planned());
    assert_eq!(parsed.specs[0].planned_release(), None);
    assert_eq!(parsed.specs[0].text, "Exports include provenance metadata.");
    assert!(parsed.diagnostics.is_empty());
}

#[test]
// @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.REJECTS_DUPLICATE_MARKERS
fn rejects_duplicate_inline_and_adjacent_planned_markers_in_version_1() {
    let block = source_block(&[
        "@spec EXPORT.METADATA @planned 0.4.0",
        "@planned 0.5.0",
        "Exports include provenance metadata.",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert!(parsed.specs[0].is_planned());
    assert_eq!(parsed.specs[0].planned_release(), Some("0.4.0"));
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("@planned must appear only once")
    );
}

#[test]
// @verifies SPECIAL.PARSE.PLANNED.LEGACY_V0
fn preserves_legacy_planned_association_without_version() {
    let block = source_block(&[
        "@spec EXPORT.METADATA",
        "Exports include provenance metadata.",
        "@planned",
    ]);

    let parsed = parse_with_version(&block, SpecialVersion::V0);

    assert_eq!(parsed.specs.len(), 1);
    assert!(parsed.specs[0].is_planned());
    assert_eq!(parsed.specs[0].planned_release(), None);
    assert!(parsed.diagnostics.is_empty());
}

#[test]
fn rejects_inline_planned_syntax_in_compatibility_mode() {
    let block = source_block(&["@spec EXPORT.METADATA @planned"]);

    let parsed = parse_with_version(&block, SpecialVersion::V0);

    assert!(parsed.specs.is_empty());
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("compatibility parsing does not allow inline lifecycle markers")
    );
}

#[test]
// @verifies SPECIAL.PARSE.PLANNED.RELEASE_TARGET
fn parses_planned_release_metadata_in_supported_forms() {
    let legacy = source_block(&[
        "@spec EXPORT.LEGACY",
        "Legacy planned behavior.",
        "@planned 0.3.0",
    ]);
    let inline_v1 = source_block(&[
        "@spec EXPORT.INLINE @planned 0.4.0",
        "Inline planned behavior.",
    ]);
    let next_line_v1 = source_block(&[
        "@spec EXPORT.NEXT",
        "@planned 0.5.0",
        "Next-line planned behavior.",
    ]);

    let legacy_parsed = parse_with_version(&legacy, SpecialVersion::V0);
    let inline_parsed = parse_current(&inline_v1);
    let next_line_parsed = parse_current(&next_line_v1);

    assert_eq!(legacy_parsed.specs[0].planned_release(), Some("0.3.0"));
    assert_eq!(inline_parsed.specs[0].planned_release(), Some("0.4.0"));
    assert_eq!(next_line_parsed.specs[0].planned_release(), Some("0.5.0"));
}

#[test]
// @verifies SPECIAL.PARSE.DEPRECATED
fn records_deprecated_on_the_owning_spec_for_each_supported_version() {
    let legacy = source_block(&[
        "@spec EXPORT.LEGACY",
        "Legacy deprecated behavior.",
        "@deprecated",
    ]);
    let current = source_block(&[
        "@spec EXPORT.CURRENT",
        "@deprecated",
        "Current deprecated behavior.",
    ]);

    let legacy_parsed = parse_with_version(&legacy, SpecialVersion::V0);
    let current_parsed = parse_current(&current);

    assert!(legacy_parsed.specs[0].is_deprecated());
    assert!(current_parsed.specs[0].is_deprecated());
    assert_eq!(legacy_parsed.specs[0].deprecated_release(), None);
    assert_eq!(current_parsed.specs[0].deprecated_release(), None);
}

#[test]
// @verifies SPECIAL.PARSE.DEPRECATED.RELEASE_TARGET
fn parses_deprecated_release_metadata_in_supported_forms() {
    let legacy = source_block(&[
        "@spec EXPORT.LEGACY",
        "Legacy deprecated behavior.",
        "@deprecated 0.6.0",
    ]);
    let inline_v1 = source_block(&[
        "@spec EXPORT.INLINE @deprecated 0.7.0",
        "Inline deprecated behavior.",
    ]);
    let next_line_v1 = source_block(&[
        "@spec EXPORT.NEXT",
        "@deprecated 0.8.0",
        "Next-line deprecated behavior.",
    ]);

    let legacy_parsed = parse_with_version(&legacy, SpecialVersion::V0);
    let inline_parsed = parse_current(&inline_v1);
    let next_line_parsed = parse_current(&next_line_v1);

    assert_eq!(legacy_parsed.specs[0].deprecated_release(), Some("0.6.0"));
    assert_eq!(inline_parsed.specs[0].deprecated_release(), Some("0.7.0"));
    assert_eq!(
        next_line_parsed.specs[0].deprecated_release(),
        Some("0.8.0")
    );
}

#[test]
fn keeps_the_spec_when_inline_planned_conflicts_with_adjacent_deprecated() {
    let block = source_block(&[
        "@spec EXPORT.CONFLICT @planned 0.4.0",
        "@deprecated 0.6.0",
        "Conflicting lifecycle behavior.",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert!(parsed.specs[0].is_planned());
    assert_eq!(parsed.specs[0].planned_release(), Some("0.4.0"));
    assert!(!parsed.specs[0].is_deprecated());
    assert_eq!(parsed.specs[0].deprecated_release(), None);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("@spec may not be both planned and deprecated")
    );
}

#[test]
// @verifies SPECIAL.GROUPS
fn parses_group_declarations() {
    let block = source_block(&["@group EXPORT", "Export-related claims."]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert_eq!(parsed.specs[0].kind(), NodeKind::Group);
    assert!(!parsed.specs[0].is_planned());
    assert!(parsed.diagnostics.is_empty());
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.PLANNED_SCOPE
fn rejects_inline_planned_on_group_nodes() {
    let block = source_block(&["@group AUTH @planned"]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert!(!parsed.specs[0].is_planned());
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("@planned may only apply to @spec, not @group")
    );
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.PLANNED_SCOPE
fn rejects_planned_outside_spec_declaration() {
    let block = source_block(&["@planned"]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 0);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("@planned must be adjacent to exactly one owning @spec")
    );
}

#[test]
// @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.REJECTS_BACKWARD_FORM
fn rejects_non_adjacent_planned_in_version_1() {
    let block = source_block(&[
        "@spec EXPORT.METADATA",
        "Exports include provenance metadata.",
        "@planned",
    ]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert!(!parsed.specs[0].is_planned());
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("backward-looking form is not allowed in version 1")
    );
}

#[test]
// @verifies SPECIAL.GROUPS.STRUCTURAL_ONLY
fn rejects_planned_on_group_nodes() {
    let block = source_block(&["@group EXPORT", "@planned"]);

    let parsed = parse_current(&block);

    assert_eq!(parsed.specs.len(), 1);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(
        parsed.diagnostics[0]
            .message
            .contains("@planned may only apply to @spec")
    );
}
