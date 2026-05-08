/**
@module SPECIAL.TESTS.CACHE.SUPPORT
Shared cache test helpers and repository fixtures in `src/cache/tests/support.rs`.
*/
// @fileimplements SPECIAL.TESTS.CACHE.SUPPORT
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::test_support::TempProjectDir;

pub(super) fn cache_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn temp_root(prefix: &str) -> TempProjectDir {
    TempProjectDir::new(prefix)
}

pub(super) fn write_repo_fixture(root: &Path) {
    std::fs::create_dir_all(root.join("_project")).expect("architecture dir should exist");
    std::fs::create_dir_all(root.join("specs")).expect("specs dir should exist");
    std::fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("config should be written");
    std::fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module APP.CORE`\nCore module.\n",
    )
    .expect("architecture should be written");
    std::fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
    )
    .expect("specs should be written");
    std::fs::write(
        root.join("app.rs"),
        "/**\n@spec APP.LIVE\nLive behavior.\n*/\n\n// @fileimplements APP.CORE\npub fn live_impl() {}\n",
    )
    .expect("source should be written");
}
