/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.SURFACE
Summarizes TypeScript item visibility and review-surface classification for the built-in TypeScript pack analyzer.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.SURFACE
use std::path::Path;

use crate::model::ModuleItemKind;

#[derive(Default)]
pub(super) struct TypeScriptSurfaceSummary {
    pub(super) public_items: usize,
    pub(super) internal_items: usize,
}

impl TypeScriptSurfaceSummary {
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

pub(super) fn is_typescript_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx")
    )
}

pub(super) fn is_test_file_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "tests")
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.ends_with(".test.ts")
                    || name.ends_with(".test.tsx")
                    || name.ends_with(".spec.ts")
                    || name.ends_with(".spec.tsx")
            })
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

#[cfg(test)]
mod tests {
    use super::{is_review_surface, is_test_file_path, is_typescript_path};
    use crate::syntax::SourceItemKind;

    #[test]
    fn provider_surface_recognizes_typescript_files_and_test_files() {
        assert!(is_typescript_path(std::path::Path::new("src/app.ts")));
        assert!(is_typescript_path(std::path::Path::new("src/app.tsx")));
        assert!(!is_typescript_path(std::path::Path::new("src/app.js")));
        assert!(is_test_file_path(std::path::Path::new("src/app.test.ts")));
        assert!(is_test_file_path(std::path::Path::new("src/tests/app.ts")));
        assert!(!is_test_file_path(std::path::Path::new("src/app.ts")));
    }

    #[test]
    fn provider_surface_keeps_tests_out_of_review_surface() {
        assert!(is_review_surface(
            true,
            "render",
            SourceItemKind::Function,
            false
        ));
        assert!(is_review_surface(
            false,
            "main",
            SourceItemKind::Function,
            false
        ));
        assert!(!is_review_surface(
            true,
            "render",
            SourceItemKind::Function,
            true
        ));
    }
}
