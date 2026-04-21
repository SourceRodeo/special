/**
@module SPECIAL.RENDER.TEXT.OVERVIEW
Renders the repo overview summary into human-readable text output.
*/
// @fileimplements SPECIAL.RENDER.TEXT.OVERVIEW
use crate::model::OverviewDocument;

pub(in crate::render) fn render_overview_text(document: &OverviewDocument) -> String {
    let mut output = String::from("special\n");
    output.push_str("  lint\n");
    output.push_str(&format!("    errors: {}\n", document.lint.errors));
    output.push_str(&format!("    warnings: {}\n", document.lint.warnings));
    output.push_str("  specs\n");
    output.push_str(&format!(
        "    total specs: {}\n",
        document.specs.total_specs
    ));
    output.push_str(&format!(
        "    unverified specs: {}\n",
        document.specs.unverified_specs
    ));
    output.push_str(&format!(
        "    planned specs: {}\n",
        document.specs.planned_specs
    ));
    output.push_str(&format!(
        "    deprecated specs: {}\n",
        document.specs.deprecated_specs
    ));
    output.push_str("  arch\n");
    output.push_str(&format!("    modules: {}\n", document.arch.total_modules));
    output.push_str(&format!("    areas: {}\n", document.arch.total_areas));
    output.push_str(&format!(
        "    unimplemented modules: {}\n",
        document.arch.unimplemented_modules
    ));
    output.push_str("  health\n");
    output.push_str(&format!(
        "    duplicate items: {}\n",
        document.health.duplicate_items
    ));
    output.push_str(&format!(
        "    unowned items: {}\n",
        document.health.unowned_items
    ));

    output.push_str("  look next\n");
    output.push_str("    special lint\n");
    output.push_str("    special specs\n");
    output.push_str("    special specs --metrics\n");
    output.push_str("    special specs --unverified\n");
    output.push_str("    special arch\n");
    output.push_str("    special arch --metrics\n");
    output.push_str("    special arch --unimplemented\n");
    output.push_str("    special health\n");
    output.push_str("    special health --metrics\n");

    output
}
