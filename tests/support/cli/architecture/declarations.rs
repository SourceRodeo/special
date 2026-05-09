/**
@module SPECIAL.TESTS.SUPPORT.CLI.ARCHITECTURE.DECLARATIONS
Architecture declaration and lint fixture writers for CLI integration tests.
*/
// @fileimplements SPECIAL.TESTS.SUPPORT.CLI.ARCHITECTURE.DECLARATIONS
use std::fs;
use std::path::Path;

pub fn write_modules_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @module DEMO\nDemo root module.\n\n### @module DEMO.LIVE\nLive child module.\n\n### @module DEMO.PLANNED @planned 0.4.0\nPlanned child module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\n\n// @implements DEMO.LIVE\nfn implements_demo_live() {}\n",
    )
    .expect("module implementation fixture should be written");
}

pub fn write_markdown_declarations_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::create_dir_all(root.join("docs")).expect("docs dir should be created");
    fs::write(
        root.join("docs/specs.md"),
        "### @group DEMO\nDemo root group.\n\n### @spec DEMO.MARKDOWN\nDemo root claim.\n",
    )
    .expect("markdown specs fixture should be written");
    fs::write(
        root.join("docs/architecture.md"),
        "### @area DEMO\nDemo architecture root.\n\n### @area DEMO.AREA\nDemo architecture area.\n\n### @module DEMO.MODULE\nDemo architecture module.\n",
    )
    .expect("markdown modules fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO.MODULE\nfn implements_demo_module() {}\n",
    )
    .expect("markdown module implementation fixture should be written");
}

pub fn write_unimplemented_child_module_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @module DEMO\nDemo root module.\n\n### @module DEMO.LIVE\nLive child module.\n\n### @module DEMO.UNIMPLEMENTED\nUnimplemented child module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\n\n// @implements DEMO.LIVE\nfn implements_demo_live() {}\n",
    )
    .expect("module implementation fixture should be written");
}

pub fn write_area_modules_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @area DEMO\nDemo area.\n\n### @module DEMO.API\nAPI module.\n\n### @module DEMO.WEB\nWeb module.\n",
    )
    .expect("area architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO.API\n\n// @implements DEMO.WEB\nfn implements_demo_web() {}\n",
    )
    .expect("area module implementation fixture should be written");
}

pub fn write_area_implements_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @area DEMO\nDemo area.\n",
    )
    .expect("area architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @implements DEMO\nfn invalid_area_implements() {}\n",
    )
    .expect("area implements fixture should be written");
}

pub fn write_planned_area_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @area DEMO @planned 0.4.0\nDemo area.\n",
    )
    .expect("planned area architecture fixture should be written");
}

pub fn write_planned_area_invalid_suffix_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @module DEMO.PLANNED\n@plannedness\nPlanned module with invalid marker suffix.\n",
    )
    .expect("planned area invalid suffix fixture should be written");
}

pub fn write_unimplemented_module_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @module DEMO\nDemo root module.\n",
    )
    .expect("architecture fixture should be written");
}

pub fn write_unknown_implements_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @module DEMO\nDemo root module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @implements DEMO.MISSING\nfn invalid_unknown_implements() {}\n",
    )
    .expect("unknown implements fixture should be written");
}

pub fn write_implements_with_trailing_content_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @module DEMO\nDemo root module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO trailing text\n// @implements DEMO trailing text\nfn invalid_trailing_content() {}\n",
    )
    .expect("trailing content fixture should be written");
}

pub fn write_missing_intermediate_modules_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @module DEMO.CHILD.LEAF\nLeaf module without DEMO or DEMO.CHILD declarations.\n",
    )
    .expect("missing intermediate modules fixture should be written");
}

pub fn write_duplicate_file_scoped_implements_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @module DEMO\nDemo root module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\n// @fileimplements DEMO\nfn duplicate_file_scoped() {}\n",
    )
    .expect("duplicate file scoped fixture should be written");
}

pub fn write_duplicate_item_scoped_implements_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### @module DEMO\nDemo root module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @implements DEMO\n// @implements DEMO\nfn duplicate_item_scoped() {}\n",
    )
    .expect("duplicate item scoped fixture should be written");
}

pub fn write_source_local_modules_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("main.rs"),
        "/**\n@module DEMO\nDemo root module.\n\n@module DEMO.LOCAL\nLocal child module.\n*/\n// @fileimplements DEMO\n\n// @implements DEMO.LOCAL\nfn serves_local() {}\n",
    )
    .expect("source-local modules fixture should be written");
}

pub fn write_mixed_purpose_source_local_module_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("main.rs"),
        "/**\n@module DEMO\nRenders the demo export surface.\n\n@spec APP.CORE\nThis explanatory line should be ignored after the parser sees the next comment tag.\n*/\n// @fileimplements DEMO\nfn serves_demo() {}\n",
    )
    .expect("mixed-purpose source-local module fixture should be written");
}
