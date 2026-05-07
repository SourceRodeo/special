/**
@module SPECIAL.LANGUAGE_PACKS.PYTHON.TEST_FIXTURES
Python fixture scenarios for parser-backed Python language-pack coverage.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.PYTHON.TEST_FIXTURES
use std::path::Path;

#[path = "../shared/test_fixture_support.rs"]
mod support;

use support::{create_dirs, write_architecture, write_file, write_specs};

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_python_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src", "src/package", "tests"]);
    write_file(root, "special.toml", "version = \"1\"\nroot = \".\"\n");
    write_architecture(
        root,
        "# Architecture\n\n### `@module APP`\nApp module.\n\n### `@module SHARED`\nShared module.\n\n### `@module WORKER`\nWorker module.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive Python behavior.\n",
    );
    write_file(
        root,
        "src/app.py",
        "# @fileimplements APP\nfrom package.shared import shared_value as live_shared_value\nfrom package.service import Worker\n\n\ndef live_impl():\n    return helper() + live_shared_value() + Worker().run()\n\n\ndef orphan_impl():\n    return 1\n\n\ndef helper():\n    return 1\n",
    );
    write_file(root, "src/package/__init__.py", "");
    write_file(
        root,
        "src/package/shared.py",
        "# @fileimplements SHARED\n\n\ndef shared_value():\n    return 1\n",
    );
    write_file(
        root,
        "src/package/service.py",
        "# @fileimplements WORKER\nfrom .helpers import nested_value\n\n\nclass Worker:\n    def run(self):\n        return self.offset() + nested_value()\n\n    def offset(self):\n        return 1\n",
    );
    write_file(
        root,
        "src/package/helpers.py",
        "# @fileimplements WORKER\n\n\ndef nested_value():\n    return 1\n",
    );
    write_file(
        root,
        "tests/test_app.py",
        "import pytest\nfrom app import live_impl\n\n\n@pytest.fixture\ndef live_result():\n    return live_impl()\n\n\n# @verifies APP.LIVE\ndef test_live_impl(live_result):\n    return live_result\n",
    );
}
