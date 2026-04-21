/**
@module SPECIAL.LANGUAGE_PACKS.GO.TEST_FIXTURES
Pack-owned Go integration-test fixtures for module analysis and traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.TEST_FIXTURES
use std::fs;
use std::path::Path;

pub fn write_go_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("shared")).expect("shared dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module SHARED`\nShared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport \"fmt\"\nimport \"shared\"\n\nfunc Entry() int {\n    return localHelper() + shared.SharedValue()\n}\n\nfunc localHelper() int {\n    return 1\n}\n\nfunc isolatedExternal() {\n    fmt.Println(\"demo\")\n}\n\nfunc unreachedClusterEntry() {\n    unreachedClusterLeaf()\n}\n\nfunc unreachedClusterLeaf() {}\n",
    )
    .expect("go implementation fixture should be written");
    fs::write(
        root.join("shared/shared.go"),
        "// @fileimplements SHARED\npackage shared\n\nfunc SharedValue() int {\n    return 1\n}\n",
    )
    .expect("go shared fixture should be written");
}

pub fn write_go_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("shared")).expect("shared dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module SHARED`\nShared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport \"shared\"\n\nfunc LiveImpl() int {\n    return helper() + shared.SharedValue()\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc helper() int {\n    return 1\n}\n",
    )
    .expect("go implementation fixture should be written");
    fs::write(
        root.join("shared/shared.go"),
        "// @fileimplements SHARED\npackage shared\n\nfunc SharedValue() int {\n    return 1\n}\n",
    )
    .expect("go shared fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go traceability test fixture should be written");
}

pub fn write_go_tool_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("left")).expect("left dir should be created");
    fs::create_dir_all(root.join("right")).expect("right dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LEFT`\nLeft shared module.\n\n### `@module RIGHT`\nRight shared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport l \"example.com/demo/left\"\n\nfunc LiveImpl() int {\n    return helper() + l.SharedValue()\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc helper() int {\n    return 1\n}\n",
    )
    .expect("go implementation fixture should be written");
    fs::write(
        root.join("left/shared.go"),
        "// @fileimplements LEFT\npackage left\n\nfunc SharedValue() int {\n    return 1\n}\n",
    )
    .expect("go left fixture should be written");
    fs::write(
        root.join("right/shared.go"),
        "// @fileimplements RIGHT\npackage right\n\nfunc SharedValue() int {\n    return 2\n}\n",
    )
    .expect("go right fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go tool traceability test fixture should be written");
}

pub fn write_go_reference_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("live")).expect("live dir should be created");
    fs::create_dir_all(root.join("dead")).expect("dead dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive callback module.\n\n### `@module DEAD`\nDead callback module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive callback behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport live \"example.com/demo/live\"\n\nfunc LiveImpl() int {\n    return invoke(live.LiveCallback)\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc invoke(callback func() int) int {\n    return callback()\n}\n",
    )
    .expect("go reference implementation fixture should be written");
    fs::write(
        root.join("live/live.go"),
        "// @fileimplements LIVE\npackage live\n\nfunc LiveCallback() int {\n    return 1\n}\n",
    )
    .expect("go live callback fixture should be written");
    fs::write(
        root.join("dead/dead.go"),
        "// @fileimplements DEAD\npackage dead\n\nfunc DeadCallback() int {\n    return 2\n}\n",
    )
    .expect("go dead callback fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go reference traceability test fixture should be written");
}
