/**
@module SPECIAL.LANGUAGE_PACKS.PYTHON.TEST_FIXTURES
Pack-owned Python integration-test fixtures for module analysis and traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.PYTHON.TEST_FIXTURES
use std::fs;
use std::path::Path;

pub fn write_python_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("app.py"),
        "# @fileimplements DEMO\ndef live_impl():\n    return _helper()\n\n\ndef _helper():\n    return 1\n\n\ndef _isolated_external():\n    return print('x')\n\n\ndef _unreached_cluster_entry():\n    return _unreached_cluster_leaf()\n\n\ndef _unreached_cluster_leaf():\n    return 1\n",
    )
    .expect("python module analysis fixture should be written");
}

pub fn write_python_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("shared")).expect("shared dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module SHARED`\nShared helper module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app.py"),
        "# @fileimplements DEMO\nfrom shared.helpers import shared_value as shared\n\n\ndef live_impl():\n    return helper() + shared()\n\n\ndef orphan_impl():\n    return 1\n\n\ndef helper():\n    return 1\n",
    )
    .expect("python implementation fixture should be written");
    fs::write(
        root.join("shared/helpers.py"),
        "# @fileimplements SHARED\ndef shared_value():\n    return 1\n",
    )
    .expect("python shared fixture should be written");
    fs::write(
        root.join("test_app.py"),
        "from app import live_impl\n\n# @verifies APP.LIVE\ndef test_live_impl():\n    live_impl()\n",
    )
    .expect("python traceability test fixture should be written");
}

pub fn write_python_reference_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("live")).expect("live dir should be created");
    fs::create_dir_all(root.join("dead")).expect("dead dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
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
        root.join("app.py"),
        "# @fileimplements DEMO\nfrom live.callbacks import live_callback\n\n\ndef run_live():\n    return invoke(live_callback)\n\n\ndef orphan_impl():\n    return 1\n\n\ndef invoke(callback):\n    return callback()\n",
    )
    .expect("python reference implementation fixture should be written");
    fs::write(
        root.join("live/callbacks.py"),
        "# @fileimplements LIVE\ndef live_callback():\n    return 1\n",
    )
    .expect("python live callback fixture should be written");
    fs::write(
        root.join("dead/callbacks.py"),
        "# @fileimplements DEAD\ndef dead_callback():\n    return 2\n",
    )
    .expect("python dead callback fixture should be written");
    fs::write(
        root.join("test_app.py"),
        "from app import run_live\n\n# @verifies APP.LIVE\ndef test_run_live():\n    run_live()\n",
    )
    .expect("python reference traceability test fixture should be written");
}

pub fn write_python_tool_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@area APP`\nApp root.\n\n### `@module APP.CORE`\nCore Python implementation.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive object-flow behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app.py"),
        "# @fileimplements APP.CORE\nclass Signer:\n    def sign(self):\n        return 1\n\n    def validate(self):\n        return True\n\n\nclass Serializer:\n    def dumps(self, value):\n        return value\n\n\ndef orphan_impl():\n    return 0\n",
    )
    .expect("python tool implementation fixture should be written");
    fs::write(
        root.join("test_app.py"),
        "from functools import partial\n\nfrom app import Serializer\nfrom app import Signer\n\n# @verifies APP.LIVE\ndef test_live_impl():\n    signer = Signer()\n    signer.sign()\n    signer.validate()\n    factory = partial(Serializer)\n    factory().dumps([42])\n",
    )
    .expect("python tool traceability test fixture should be written");
}

pub fn write_python_syntax_error_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("src")).expect("src dir should be created");
    fs::create_dir_all(root.join("tests")).expect("tests dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nBroken Python syntax still reports backward-trace unavailability honestly.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("src/live.py"),
        "# @fileimplements DEMO\n\ndef live_impl():\n    return helper() + shared_value()\n\n\ndef helper():\n    return 1\n\n\ndef shared_value():\n    return 41\n",
    )
    .expect("python implementation fixture should be written");
    fs::write(
        root.join("src/broken.py"),
        "# @fileimplements DEMO\n\ndef broken_impl(\n    return 1\n",
    )
    .expect("broken python implementation fixture should be written");
    fs::write(
        root.join("tests/test_live.py"),
        "from src.live import live_impl\n\n# @verifies APP.LIVE\ndef test_live_impl():\n    assert live_impl() == 42\n",
    )
    .expect("python syntax traceability test fixture should be written");
}
