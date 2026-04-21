/**
@module SPECIAL.RENDER.PROJECTION.SPEC
Projects spec documents into backend-ready verbose or non-verbose shapes.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.SPEC
use crate::model::{SpecDocument, SpecNode};

pub(in crate::render) fn project_document(document: &SpecDocument, verbose: bool) -> SpecDocument {
    if verbose {
        document.clone()
    } else {
        SpecDocument {
            metrics: document.metrics.clone(),
            nodes: document
                .nodes
                .iter()
                .cloned()
                .map(strip_node_support_bodies)
                .collect(),
        }
    }
}

fn strip_node_support_bodies(mut node: SpecNode) -> SpecNode {
    for verify in &mut node.verifies {
        verify.body_location = None;
        verify.body = None;
    }
    for attest in &mut node.attests {
        attest.body = None;
    }
    node.children = node
        .children
        .into_iter()
        .map(strip_node_support_bodies)
        .collect();
    node
}
