/**
@module SPECIAL.TESTS.CACHE.LOCKING
Cache single-flight, stale-lock recovery, and heartbeat tests in `src/cache/tests/locking.rs`.
*/
// @fileimplements SPECIAL.TESTS.CACHE.LOCKING
use std::sync::{Arc, Barrier, Mutex};
use std::time::SystemTime;

use crate::config::SpecialVersion;

use super::super::lock::{
    CacheFillGuard, acquire_cache_fill_lock, acquire_cache_fill_lock_with_hooks, cache_lock_owner,
    cache_lock_path, recover_stale_cache_lock_at, start_cache_lock_heartbeat,
};
use super::super::storage::cache_file_path;
use super::super::{
    CACHE_LOCK_STALE_AFTER, CACHE_SCHEMA_VERSION, load_or_parse_repo, reset_cache_stats,
    snapshot_cache_stats, with_cache_status_notifier,
};
use super::support::{cache_test_lock, temp_root, write_repo_fixture};

#[test]
fn parsed_repo_cache_single_flights_under_contention() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-repo-contention");
    write_repo_fixture(&root);
    let cache_path = cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    let blocker = acquire_cache_fill_lock(&cache_path).expect("lock should be acquired");
    let barrier = Arc::new(Barrier::new(3));

    reset_cache_stats();
    let worker = |barrier: Arc<Barrier>, root: std::path::PathBuf| {
        std::thread::spawn(move || {
            barrier.wait();
            load_or_parse_repo(&root, &[], SpecialVersion::V1)
                .expect("concurrent parse should succeed");
        })
    };

    let first = worker(Arc::clone(&barrier), root.to_path_buf());
    let second = worker(Arc::clone(&barrier), root.to_path_buf());
    barrier.wait();
    std::thread::sleep(std::time::Duration::from_millis(25));
    drop(blocker);

    first.join().expect("first worker should join");
    second.join().expect("second worker should join");

    let stats = snapshot_cache_stats();
    assert_eq!(stats.repo_misses, 1);
    assert_eq!(stats.repo_hits, 1);
    assert_eq!(stats.lock_waits, 2);

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn cache_wait_emits_real_time_status_note() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-wait-note");
    write_repo_fixture(&root);
    let cache_path = cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    let blocker = acquire_cache_fill_lock(&cache_path).expect("lock should be acquired");

    let messages = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&messages);
    let waiter = std::thread::spawn(move || {
        let guard = with_cache_status_notifier(
            move |message| {
                captured
                    .lock()
                    .expect("message mutex should not be poisoned")
                    .push(message.to_string());
            },
            || acquire_cache_fill_lock(&cache_path),
        )
        .expect("waiter should acquire lock after blocker drops");
        drop(guard);
    });
    std::thread::sleep(std::time::Duration::from_millis(25));
    drop(blocker);
    waiter.join().expect("waiter should join");

    let messages = messages
        .lock()
        .expect("message mutex should not be poisoned");
    assert!(messages.iter().any(|message| {
        message.contains("another special run is already building repo annotations")
            && message.contains("waiting to reuse its cached result")
    }));

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn stale_lock_is_recovered_before_filling_cache() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-stale-lock");
    let cache_path = cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).expect("cache dir should exist");
    }
    let lock_path = cache_lock_path(&cache_path);
    std::fs::write(&lock_path, b"stale").expect("lock file should be written");

    reset_cache_stats();
    let future_now = SystemTime::now() + CACHE_LOCK_STALE_AFTER + std::time::Duration::from_secs(1);
    let guard = acquire_cache_fill_lock_with_hooks(&cache_path, || future_now, |_| {})
        .expect("stale lock should be recovered");
    let stats = snapshot_cache_stats();
    assert_eq!(stats.lock_waits, 1);
    assert_eq!(stats.stale_lock_recovers, 1);
    assert!(
        lock_path.exists(),
        "fresh lock should exist while guard is held"
    );
    drop(guard);
    assert!(
        !lock_path.exists(),
        "lock should be removed after guard drops"
    );

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn stale_lock_recovery_counts_only_successful_recovery() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-stale-recovery-count");
    let cache_path = cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).expect("cache dir should exist");
    }
    let lock_path = cache_lock_path(&cache_path);
    std::fs::write(&lock_path, b"stale").expect("lock file should be written");

    reset_cache_stats();
    let future_now = SystemTime::now() + CACHE_LOCK_STALE_AFTER + std::time::Duration::from_secs(1);
    assert!(
        recover_stale_cache_lock_at(&lock_path, &cache_path, future_now)
            .expect("stale lock recovery should succeed"),
        "first recovery attempt should move the stale lock aside"
    );
    assert!(
        !recover_stale_cache_lock_at(&lock_path, &cache_path, future_now)
            .expect("missing lock should be treated as a lost recovery race"),
        "second recovery attempt should not count a lock another worker already moved"
    );

    let stats = snapshot_cache_stats();
    assert_eq!(stats.stale_lock_recovers, 1);
    assert!(
        !lock_path.exists(),
        "recovering a stale lock should leave the active lock path free"
    );

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn stale_lock_recovery_emits_real_time_status_note() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-stale-note");
    let cache_path = cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).expect("cache dir should exist");
    }
    let lock_path = cache_lock_path(&cache_path);
    std::fs::write(&lock_path, b"stale").expect("lock file should be written");

    let messages = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&messages);
    let future_now = SystemTime::now() + CACHE_LOCK_STALE_AFTER + std::time::Duration::from_secs(1);
    let guard = with_cache_status_notifier(
        move |message| {
            captured
                .lock()
                .expect("message mutex should not be poisoned")
                .push(message.to_string());
        },
        || acquire_cache_fill_lock_with_hooks(&cache_path, || future_now, |_| {}),
    )
    .expect("stale lock should be recovered");
    drop(guard);

    let messages = messages
        .lock()
        .expect("message mutex should not be poisoned");
    assert!(messages.iter().any(|message| {
        message.contains("recovered an abandoned cache lock for repo annotations")
    }));

    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn active_cache_lock_heartbeat_refreshes_lock_contents() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-lock-heartbeat");
    let cache_path = cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).expect("cache dir should exist");
    }
    let lock_path = cache_lock_path(&cache_path);
    std::fs::write(&lock_path, b"owner=test-owner\nupdated_nanos=0\n")
        .expect("initial lock should be written");

    let heartbeat = start_cache_lock_heartbeat(
        lock_path.clone(),
        "test-owner".to_string(),
        std::time::Duration::from_millis(10),
    );
    std::thread::sleep(std::time::Duration::from_millis(30));
    heartbeat.stop();

    let refreshed = std::fs::read_to_string(&lock_path).expect("lock should remain readable");
    assert!(refreshed.contains("owner=test-owner"));
    assert!(!refreshed.contains("updated_nanos=0"));

    std::fs::remove_file(&lock_path).expect("lock should be removable");
    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn stale_lock_recovery_does_not_let_old_owner_remove_new_lock() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-stale-owner");
    let cache_path = cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).expect("cache dir should exist");
    }
    let lock_path = cache_lock_path(&cache_path);
    std::fs::write(&lock_path, b"owner=stale\nupdated_nanos=0\n")
        .expect("stale lock should be written");

    let future_now = SystemTime::now() + CACHE_LOCK_STALE_AFTER + std::time::Duration::from_secs(1);
    let stale_guard = acquire_cache_fill_lock_with_hooks(&cache_path, || future_now, |_| {})
        .expect("stale lock should be recovered");
    let replacement_owner =
        cache_lock_owner(&lock_path).expect("replacement lock should record an owner");

    let old_guard = CacheFillGuard {
        path: lock_path.clone(),
        owner_token: "stale".to_string(),
        heartbeat: None,
    };
    drop(old_guard);

    assert_eq!(
        cache_lock_owner(&lock_path).as_deref(),
        Some(replacement_owner.as_str()),
        "old owner must not remove a newer lock"
    );

    drop(stale_guard);
    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}

#[test]
fn stale_lock_recovery_stops_old_heartbeat_from_reclaiming_owner_metadata() {
    let _guard = cache_test_lock();
    let root = temp_root("special-cache-stale-heartbeat");
    let cache_path = cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).expect("cache dir should exist");
    }
    let lock_path = cache_lock_path(&cache_path);
    std::fs::write(&lock_path, b"owner=stale\nupdated_nanos=0\n")
        .expect("stale lock should be written");

    let stale_heartbeat = start_cache_lock_heartbeat(
        lock_path.clone(),
        "stale".to_string(),
        std::time::Duration::from_millis(10),
    );
    let future_now = SystemTime::now() + CACHE_LOCK_STALE_AFTER + std::time::Duration::from_secs(1);
    let fresh_guard = acquire_cache_fill_lock_with_hooks(&cache_path, || future_now, |_| {})
        .expect("stale lock should be recovered");
    let fresh_owner =
        cache_lock_owner(&lock_path).expect("replacement lock should record an owner");

    std::thread::sleep(std::time::Duration::from_millis(30));
    stale_heartbeat.stop();

    assert_eq!(
        cache_lock_owner(&lock_path).as_deref(),
        Some(fresh_owner.as_str()),
        "old heartbeat must not overwrite replacement ownership"
    );

    drop(fresh_guard);
    std::fs::remove_dir_all(&root).expect("temp root should be removed");
}
