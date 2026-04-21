/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES.TRACEABILITY
TypeScript fixture scenarios for direct, tool-backed, and reference-backed traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES.TRACEABILITY
use std::path::Path;

use super::support::{
    create_dirs, write_architecture, write_file, write_special_toml, write_specs,
};

pub fn write_typescript_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_architecture(
        root,
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module SHARED`\nShared module.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
    );
    write_file(
        root,
        "src/app.ts",
        "// @fileimplements DEMO\nimport { sharedValue } from \"./shared\";\n\nexport function liveImpl() {\n    return helper() + sharedValue();\n}\n\nexport function orphanImpl() {\n    return 1;\n}\n\nfunction helper() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/shared.ts",
        "// @fileimplements SHARED\nexport function sharedValue() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/app.test.ts",
        "import { liveImpl } from \"./app\";\n\n// @verifies APP.LIVE\nexport function verifies_live_impl() {\n    return liveImpl();\n}\n",
    );
}

pub fn write_typescript_tool_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_architecture(
        root,
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LEFT`\nLeft shared module.\n\n### `@module RIGHT`\nRight shared module.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
    );
    write_file(
        root,
        "src/app.ts",
        "// @fileimplements DEMO\nimport { sharedValue as liveSharedValue } from \"./left\";\n\nexport function liveImpl() {\n    return helper() + liveSharedValue();\n}\n\nexport function orphanImpl() {\n    return 1;\n}\n\nfunction helper() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/left.ts",
        "// @fileimplements LEFT\nexport function sharedValue() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/right.ts",
        "// @fileimplements RIGHT\nexport function sharedValue() {\n    return 2;\n}\n",
    );
    write_file(
        root,
        "src/app.test.ts",
        "import { liveImpl } from \"./app\";\n\n// @verifies APP.LIVE\nexport function verifies_live_impl() {\n    return liveImpl();\n}\n",
    );
}

pub fn write_typescript_reference_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_architecture(
        root,
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive callback module.\n\n### `@module DEAD`\nDead callback module.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive callback behavior.\n",
    );
    write_file(
        root,
        "src/app.ts",
        "// @fileimplements DEMO\nimport { liveCallback } from \"./live\";\n\nexport function runLive() {\n    return invoke(liveCallback);\n}\n\nexport function orphanImpl() {\n    return 1;\n}\n\nfunction invoke(callback: () => number) {\n    return callback();\n}\n",
    );
    write_file(
        root,
        "src/live.ts",
        "// @fileimplements LIVE\nexport function liveCallback() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/dead.ts",
        "// @fileimplements DEAD\nexport function deadCallback() {\n    return 2;\n}\n",
    );
    write_file(
        root,
        "src/app.test.ts",
        "import { runLive } from \"./app\";\n\n// @verifies APP.LIVE\nexport function verifies_run_live() {\n    return runLive();\n}\n",
    );
}
