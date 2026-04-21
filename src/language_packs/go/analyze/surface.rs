/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.SURFACE
Summarizes Go item visibility and review-surface classification for the built-in Go pack analyzer.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.SURFACE
use std::path::Path;

use crate::model::ModuleItemKind;

#[derive(Default)]
pub(super) struct GoSurfaceSummary {
    pub(super) public_items: usize,
    pub(super) internal_items: usize,
}

impl GoSurfaceSummary {
    pub(super) fn observe(&mut self, items: &[crate::syntax::SourceItem]) {
        for item in items {
            if item.public {
                self.public_items += 1;
            } else {
                self.internal_items += 1;
            }
        }
    }
}

pub(super) fn is_go_path(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("go"))
}

pub(super) fn is_test_file_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with("_test.go"))
}

fn is_process_entrypoint_name(name: &str, kind: crate::syntax::SourceItemKind) -> bool {
    kind == crate::syntax::SourceItemKind::Function && name == "main"
}

pub(super) fn is_review_surface(
    public: bool,
    name: &str,
    kind: crate::syntax::SourceItemKind,
    test_file: bool,
) -> bool {
    !test_file && (public || is_process_entrypoint_name(name, kind))
}

pub(super) fn source_item_kind(kind: crate::syntax::SourceItemKind) -> ModuleItemKind {
    match kind {
        crate::syntax::SourceItemKind::Function => ModuleItemKind::Function,
        crate::syntax::SourceItemKind::Method => ModuleItemKind::Method,
    }
}
