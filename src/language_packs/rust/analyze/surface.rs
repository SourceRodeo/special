/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.SURFACE
Extracts Rust public and internal callable counts from the shared parser-backed source graph used by the rest of Rust health analysis.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.SURFACE
use crate::syntax::ParsedSourceGraph;

#[derive(Debug, Default)]
pub(super) struct RustSurfaceSummary {
    pub public_items: usize,
    pub internal_items: usize,
}

impl RustSurfaceSummary {
    pub(super) fn observe(&mut self, graph: &ParsedSourceGraph) {
        for item in &graph.items {
            if item.public {
                self.public_items += 1;
            } else {
                self.internal_items += 1;
            }
        }
    }
}
