#![allow(dead_code, unused_imports)]
/**
@module SPECIAL.TESTS.SUPPORT.CLI
Shared CLI integration-test helper facade with command, render, skills, specs, and generic architecture-fixture helpers delegated to child modules.
*/
// @fileimplements SPECIAL.TESTS.SUPPORT.CLI
#[path = "cli/architecture.rs"]
mod architecture;
#[path = "cli/command.rs"]
mod command;
#[path = "cli/render.rs"]
mod render;
#[path = "cli/skills.rs"]
mod skills;
#[path = "cli/specs.rs"]
mod specs;

pub use architecture::*;
pub use command::{
    pyright_langserver_available, run_special, run_special_raw, run_special_with_env_removed,
    run_special_with_input, run_special_with_input_and_env, rust_analyzer_available, spawn_special,
    temp_repo_dir,
};
pub use render::{
    find_node_by_id, html_node_has_badge, installed_skill_ids, listed_skill_ids,
    rendered_spec_node_ids, rendered_spec_node_line, rendered_spec_node_lines,
    top_level_help_command_names, top_level_help_command_summaries, top_level_help_commands,
};
pub use skills::{
    bundled_skill_ids, bundled_skill_markdown, install_skills, skills_command_shape_lines,
    skills_install_destination_lines, skills_install_destinations,
    write_invalid_skills_root_fixture, write_skills_fixture,
};
pub use specs::{
    write_current_and_planned_fixture, write_deprecated_release_fixture, write_file_attest_fixture,
    write_file_verify_fixture, write_lint_error_fixture, write_missing_version_fixture,
    write_non_adjacent_planned_v1_fixture, write_orphan_verify_fixture,
    write_planned_release_fixture, write_special_toml_dot_root_fixture,
    write_special_toml_root_fixture, write_supported_fixture_without_config,
    write_unverified_current_fixture,
};
