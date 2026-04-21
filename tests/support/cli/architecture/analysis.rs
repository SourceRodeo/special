/**
@module SPECIAL.TESTS.SUPPORT.CLI.ARCHITECTURE.ANALYSIS
Module-analysis-oriented architecture fixture writers for CLI integration tests.
*/
// @fileimplements SPECIAL.TESTS.SUPPORT.CLI.ARCHITECTURE.ANALYSIS
use std::fs;
use std::path::Path;

pub fn write_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn demo_public() {}\n\nfn demo_private() {}\n",
    )
    .expect("main module implementation fixture should be written");
    fs::write(root.join("hidden.rs"), "fn hidden_subsystem() {}\n")
        .expect("hidden subsystem fixture should be written");
}

pub fn write_item_scoped_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "const BEFORE: usize = 1;\n\n// @implements DEMO\npub fn demo_public() {}\n\nfn hidden_helper() {}\n",
    )
    .expect("item-scoped module analysis fixture should be written");
}

pub fn write_source_local_module_analysis_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("main.rs"),
        "/**\n@module DEMO\nDemo module.\n*/\n// @fileimplements DEMO\npub fn demo_public() {}\n\nfn demo_private() {}\n",
    )
    .expect("source-local module analysis fixture should be written");
}

pub fn write_dependency_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\nuse crate::shared::util::helper;\nuse serde_json::Value;\n\npub fn demo_public() -> Value {\n    helper()\n}\n",
    )
    .expect("dependency analysis fixture should be written");
}

pub fn write_complexity_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn simple() {}\n\nfn branchy(a: bool, b: bool) {\n    if a && b {\n        for _i in 0..1 {}\n    } else if a || b {\n        while a {\n            break;\n        }\n    }\n}\n",
    )
    .expect("complexity analysis fixture should be written");
}

pub fn write_cognitive_complexity_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn simple() {}\n\nfn nested(flag: bool) {\n    if flag {\n        for _i in 0..1 {}\n    }\n}\n",
    )
    .expect("cognitive complexity analysis fixture should be written");
}

pub fn write_quality_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn open_widget(id: &str, name: String, force: bool) {\n    if force {\n        panic!(\"forced\");\n    }\n}\n",
    )
    .expect("quality analysis fixture should be written");
}

pub fn write_coupling_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@area DEMO`\nDemo root.\n\n### `@module DEMO.API`\nAPI module.\n\n### `@module DEMO.SHARED`\nShared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("api.rs"),
        "// @fileimplements DEMO.API\nuse crate::shared::helper;\n\npub fn run() {\n    helper();\n}\n",
    )
    .expect("api analysis fixture should be written");
    fs::write(
        root.join("shared.rs"),
        "// @fileimplements DEMO.SHARED\npub fn helper() {}\n",
    )
    .expect("shared analysis fixture should be written");
}

pub fn write_ambiguous_coupling_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@area DEMO`\nDemo root.\n\n### `@module DEMO.API`\nAPI module.\n\n### `@module DEMO.LEFT`\nLeft shared module.\n\n### `@module DEMO.RIGHT`\nRight shared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("api.rs"),
        "// @fileimplements DEMO.API\nuse crate::shared::helper;\n\npub fn run() {\n    helper();\n}\n",
    )
    .expect("api analysis fixture should be written");
    fs::write(
        root.join("shared.rs"),
        "// @implements DEMO.LEFT\npub fn helper() {}\n\n// @implements DEMO.RIGHT\npub fn helper_alt() {}\n",
    )
    .expect("ambiguous shared analysis fixture should be written");
}

pub fn write_item_signals_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn entry() {\n    core_helper();\n}\n\nfn core_helper() {\n    helper_leaf();\n    helper_leaf();\n}\n\nfn helper_leaf() {}\n\npub fn outbound_heavy(id: &str, path: String, force: bool) {\n    helper_leaf();\n    if force && id.is_empty() {\n        panic!(\"forced\");\n    }\n    std::env::var(\"X\").ok();\n    std::fs::read_to_string(path).ok();\n}\n\nfn complex_hotspot(flag: bool, extra: bool) {\n    if flag {\n        for _i in 0..1 {\n            if extra || flag {\n                helper_leaf();\n            }\n        }\n    } else if extra {\n        while flag {\n            break;\n        }\n    }\n}\n\nfn isolated_external() {\n    std::process::id();\n}\n",
    )
    .expect("item signals analysis fixture should be written");
}

pub fn write_item_scoped_item_signals_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @implements DEMO\nfn connected() {\n    shared();\n}\n\n// @implements DEMO\nfn shared() {}\n\n// @implements DEMO\nfn isolated_external() {\n    std::process::id();\n}\n",
    )
    .expect("item-scoped item signals fixture should be written");
}

pub fn write_unreached_code_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn entry() {\n    live_helper();\n}\n\nfn live_helper() {}\n\nfn unreached_cluster_entry() {\n    unreached_cluster_leaf();\n}\n\nfn unreached_cluster_leaf() {}\n",
    )
    .expect("unreached-code implementation fixture should be written");
    fs::write(root.join("hidden.rs"), "fn hidden_unreached() {}\n")
        .expect("unowned unreached-code fixture should be written");
}

pub fn write_duplicate_item_signals_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("alpha.rs"),
        "// @fileimplements DEMO\npub fn first_duplicate(value: i32) -> i32 {\n    let doubled = normalize(value + value);\n    if doubled > 10 {\n        doubled - offset()\n    } else {\n        doubled + offset()\n    }\n}\n\nfn normalize(value: i32) -> i32 {\n    value\n}\n\nfn offset() -> i32 {\n    1\n}\n\npub fn distinct_alpha(input: i32) -> i32 {\n    input * 3\n}\n",
    )
    .expect("first duplicate fixture should be written");
    fs::write(
        root.join("beta.rs"),
        "// @fileimplements DEMO\npub fn second_duplicate(input: i32) -> i32 {\n    let total = normalize(input + input);\n    if total > 10 {\n        total - offset()\n    } else {\n        total + offset()\n    }\n}\n\nfn normalize(value: i32) -> i32 {\n    value\n}\n\nfn offset() -> i32 {\n    1\n}\n",
    )
    .expect("second duplicate fixture should be written");
}

pub fn write_many_duplicate_item_signals_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");

    for name in ["alpha", "beta", "gamma", "delta", "epsilon", "zeta"] {
        fs::write(
            root.join(format!("{name}.rs")),
            format!(
                "// @fileimplements DEMO\npub fn {name}_duplicate(value: i32) -> i32 {{\n    let doubled = normalize(value + value);\n    if doubled > 10 {{\n        doubled - offset()\n    }} else {{\n        doubled + offset()\n    }}\n}}\n\nfn normalize(value: i32) -> i32 {{\n    value\n}}\n\nfn offset() -> i32 {{\n    1\n}}\n"
            ),
        )
        .expect("many duplicate fixture should be written");
    }
}

pub fn write_restricted_visibility_root_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("lib.rs"),
        "// @fileimplements DEMO\nmod nested {\n    pub(super) fn entry() {\n        helper();\n    }\n\n    fn helper() {}\n}\n",
    )
    .expect("restricted visibility fixture should be written");
}

pub fn write_binary_entrypoint_root_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::create_dir_all(root.join("src/bin")).expect("binary fixture dir should be created");
    fs::write(
        root.join("src/bin/demo.rs"),
        "// @fileimplements DEMO\nfn main() {\n    helper();\n}\n\nfn helper() {}\n",
    )
    .expect("binary entrypoint fixture should be written");
}
