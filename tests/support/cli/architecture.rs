/**
@module SPECIAL.TESTS.SUPPORT.CLI.ARCHITECTURE
Architecture and module-analysis fixture facade for CLI integration tests.
*/
// @fileimplements SPECIAL.TESTS.SUPPORT.CLI.ARCHITECTURE
#[path = "architecture/analysis.rs"]
mod analysis;
#[path = "architecture/declarations.rs"]
mod declarations;

pub use analysis::{
    write_ambiguous_coupling_module_analysis_fixture, write_binary_entrypoint_root_fixture,
    write_cognitive_complexity_module_analysis_fixture, write_complexity_module_analysis_fixture,
    write_coupling_module_analysis_fixture, write_dependency_module_analysis_fixture,
    write_duplicate_item_signals_module_analysis_fixture,
    write_item_scoped_item_signals_module_analysis_fixture,
    write_item_scoped_module_analysis_fixture, write_item_signals_module_analysis_fixture,
    write_many_duplicate_item_signals_module_analysis_fixture, write_module_analysis_fixture,
    write_quality_module_analysis_fixture, write_restricted_visibility_root_fixture,
    write_source_local_module_analysis_fixture, write_unreached_code_module_analysis_fixture,
};
pub use declarations::{
    write_area_implements_fixture, write_area_modules_fixture,
    write_duplicate_file_scoped_implements_fixture, write_duplicate_item_scoped_implements_fixture,
    write_implements_with_trailing_content_fixture, write_markdown_declarations_fixture,
    write_missing_intermediate_modules_fixture, write_mixed_purpose_source_local_module_fixture,
    write_modules_fixture, write_planned_area_fixture, write_planned_area_invalid_suffix_fixture,
    write_source_local_modules_fixture, write_unimplemented_child_module_fixture,
    write_unimplemented_module_fixture, write_unknown_implements_fixture,
};
