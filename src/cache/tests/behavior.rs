/**
@module SPECIAL.TESTS.CACHE.BEHAVIOR
Cache hit, invalidation, and malformed-entry tests in `src/cache/tests/behavior.rs`.
*/
// @fileimplements SPECIAL.TESTS.CACHE.BEHAVIOR
use crate::config::SpecialVersion;
use crate::model::ModuleAnalysisOptions;

use super::super::storage::cache_file_path;
use super::super::{
    CACHE_SCHEMA_VERSION, load_or_build_architecture_analysis, load_or_build_repo_analysis_summary,
    load_or_parse_architecture, load_or_parse_repo, reset_cache_stats, snapshot_cache_stats,
};
use super::support::{cache_test_lock, temp_root, write_repo_fixture};

#[test]
fn parsed_repo_cache_hits_on_second_load() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-repo-hit");
    write_repo_fixture(&root);

    reset_cache_stats();
    let _ = load_or_parse_repo(&root, &[], SpecialVersion::V1).expect("parse should succeed");
    let first = snapshot_cache_stats();
    assert_eq!(first.repo_hits, 0);
    assert_eq!(first.repo_misses, 1);

    let _ =
        load_or_parse_repo(&root, &[], SpecialVersion::V1).expect("cached parse should succeed");
    let second = snapshot_cache_stats();
    assert_eq!(second.repo_hits, 1);
    assert_eq!(second.repo_misses, 1);

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn parsed_architecture_cache_invalidates_after_file_change() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-arch-invalidate");
    write_repo_fixture(&root);

    reset_cache_stats();
    let _ = load_or_parse_architecture(&root, &[]).expect("parse should succeed");
    let first = snapshot_cache_stats();
    assert_eq!(first.architecture_hits, 0);
    assert_eq!(first.architecture_misses, 1);

    let _ = load_or_parse_architecture(&root, &[]).expect("cached parse should succeed");
    let second = snapshot_cache_stats();
    assert_eq!(second.architecture_hits, 1);
    assert_eq!(second.architecture_misses, 1);

    std::thread::sleep(std::time::Duration::from_millis(5));
    std::fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module APP.CORE`\nChanged module text.\n",
    )
    .expect("architecture should be rewritten");

    let _ = load_or_parse_architecture(&root, &[]).expect("reparse should succeed");
    let third = snapshot_cache_stats();
    assert_eq!(third.architecture_hits, 1);
    assert_eq!(third.architecture_misses, 2);

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn repo_analysis_cache_hits_on_second_load() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-repo-analysis-hit");
    write_repo_fixture(&root);
    let parsed_repo =
        load_or_parse_repo(&root, &[], SpecialVersion::V1).expect("parse should succeed");
    let parsed_arch = load_or_parse_architecture(&root, &[]).expect("parse should succeed");

    reset_cache_stats();
    let _ = load_or_build_repo_analysis_summary(
        &root,
        &[],
        SpecialVersion::V1,
        &parsed_arch,
        &parsed_repo,
    )
    .expect("analysis should succeed");
    let first = snapshot_cache_stats();
    assert_eq!(first.repo_analysis_hits, 0);
    assert_eq!(first.repo_analysis_misses, 1);

    let _ = load_or_build_repo_analysis_summary(
        &root,
        &[],
        SpecialVersion::V1,
        &parsed_arch,
        &parsed_repo,
    )
    .expect("cached analysis should succeed");
    let second = snapshot_cache_stats();
    assert_eq!(second.repo_analysis_hits, 1);
    assert_eq!(second.repo_analysis_misses, 1);

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn architecture_analysis_cache_invalidates_after_file_change() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-arch-analysis-invalidate");
    write_repo_fixture(&root);
    let parsed_repo =
        load_or_parse_repo(&root, &[], SpecialVersion::V1).expect("parse should succeed");
    let parsed_arch = load_or_parse_architecture(&root, &[]).expect("parse should succeed");

    reset_cache_stats();
    let _ = load_or_build_architecture_analysis(
        &root,
        &[],
        SpecialVersion::V1,
        &parsed_arch,
        Some(&parsed_repo),
        ModuleAnalysisOptions {
            coverage: true,
            metrics: true,
            traceability: false,
        },
    )
    .expect("analysis should succeed");
    let first = snapshot_cache_stats();
    assert_eq!(first.architecture_analysis_hits, 0);
    assert_eq!(first.architecture_analysis_misses, 1);

    let _ = load_or_build_architecture_analysis(
        &root,
        &[],
        SpecialVersion::V1,
        &parsed_arch,
        Some(&parsed_repo),
        ModuleAnalysisOptions {
            coverage: true,
            metrics: true,
            traceability: false,
        },
    )
    .expect("cached analysis should succeed");
    let second = snapshot_cache_stats();
    assert_eq!(second.architecture_analysis_hits, 1);
    assert_eq!(second.architecture_analysis_misses, 1);

    std::thread::sleep(std::time::Duration::from_millis(5));
    std::fs::write(
        root.join("app.rs"),
        "/**\n@spec APP.LIVE\nLive behavior.\n*/\n\n// @fileimplements APP.CORE\npub fn changed_impl() {}\n",
    )
    .expect("source should be rewritten");
    let parsed_repo =
        load_or_parse_repo(&root, &[], SpecialVersion::V1).expect("reparse should succeed");
    let parsed_arch = load_or_parse_architecture(&root, &[]).expect("reparse should succeed");

    let _ = load_or_build_architecture_analysis(
        &root,
        &[],
        SpecialVersion::V1,
        &parsed_arch,
        Some(&parsed_repo),
        ModuleAnalysisOptions {
            coverage: true,
            metrics: true,
            traceability: false,
        },
    )
    .expect("recomputed analysis should succeed");
    let third = snapshot_cache_stats();
    assert_eq!(third.architecture_analysis_hits, 1);
    assert_eq!(third.architecture_analysis_misses, 2);

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn malformed_repo_cache_is_ignored_and_rebuilt_cleanly() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-repo-malformed");
    write_repo_fixture(&root);
    let cache_path = cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).expect("cache dir should exist");
    }
    std::fs::write(&cache_path, b"{not valid json").expect("malformed cache should be written");

    reset_cache_stats();
    let parsed = load_or_parse_repo(&root, &[], SpecialVersion::V1)
        .expect("repo should rebuild from malformed cache");
    assert_eq!(parsed.specs.len(), 3);
    let first = snapshot_cache_stats();
    assert_eq!(first.repo_misses, 1);
    assert_eq!(first.repo_hits, 0);

    let _ = load_or_parse_repo(&root, &[], SpecialVersion::V1)
        .expect("rebuilt repo cache should be reusable");
    let second = snapshot_cache_stats();
    assert_eq!(second.repo_misses, 1);
    assert_eq!(second.repo_hits, 1);

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn parsed_repo_cache_invalidates_on_same_size_source_edit() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-repo-same-size-edit");
    write_repo_fixture(&root);
    let source_path = root.join("specs/root.md");

    reset_cache_stats();
    let first =
        load_or_parse_repo(&root, &[], SpecialVersion::V1).expect("initial parse should succeed");
    assert_eq!(first.specs.len(), 3);
    let first_stats = snapshot_cache_stats();
    assert_eq!(first_stats.repo_hits, 0);
    assert_eq!(first_stats.repo_misses, 1);

    let original_source =
        std::fs::read_to_string(&source_path).expect("original source should be readable");
    let updated_source = original_source
        .replace("APP.LIVE", "APP.CURR")
        .replace("Live behavior.", "Same behavior.");
    assert_eq!(
        original_source.len(),
        updated_source.len(),
        "same-size edit should keep the source length stable"
    );
    std::fs::write(&source_path, &updated_source).expect("same-size source should be rewritten");

    let reparsed = load_or_parse_repo(&root, &[], SpecialVersion::V1)
        .expect("same-size edit should invalidate cache");
    let ids = reparsed
        .specs
        .iter()
        .map(|spec| spec.id.clone())
        .collect::<Vec<_>>();
    assert!(ids.iter().any(|id| id == "APP.CURR"));
    let second_stats = snapshot_cache_stats();
    assert_eq!(second_stats.repo_hits, 0);
    assert_eq!(second_stats.repo_misses, 2);

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}
