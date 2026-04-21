/**
@module SPECIAL.CACHE.STATS
Tracks cache-hit, rebuild, and lock-contention statistics plus status notifications in `SPECIAL.CACHE`.
*/
// @fileimplements SPECIAL.CACHE.STATS
use std::cell::RefCell;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    pub repo_hits: usize,
    pub repo_misses: usize,
    pub architecture_hits: usize,
    pub architecture_misses: usize,
    pub repo_analysis_hits: usize,
    pub repo_analysis_misses: usize,
    pub architecture_analysis_hits: usize,
    pub architecture_analysis_misses: usize,
    pub lock_waits: usize,
    pub stale_lock_recovers: usize,
}

thread_local! {
    static CACHE_STATUS_NOTIFIER: RefCell<Option<Box<dyn Fn(&str)>>> = RefCell::new(None);
}

pub(super) fn reset_cache_stats() {
    *cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned") = CacheStats::default();
}

pub(super) fn snapshot_cache_stats() -> CacheStats {
    *cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned")
}

pub(super) fn format_cache_stats_summary() -> Option<String> {
    let stats = snapshot_cache_stats();
    if stats.repo_hits
        + stats.repo_misses
        + stats.architecture_hits
        + stats.architecture_misses
        + stats.repo_analysis_hits
        + stats.repo_analysis_misses
        + stats.architecture_analysis_hits
        + stats.architecture_analysis_misses
        + stats.lock_waits
        + stats.stale_lock_recovers
        == 0
    {
        return None;
    }
    let mut summary = format!(
        "cache activity: repo annotations reused {}, rebuilt {}; architecture declarations reused {}, rebuilt {}; health analysis reused {}, rebuilt {}; architecture analysis reused {}, rebuilt {}",
        stats.repo_hits,
        stats.repo_misses,
        stats.architecture_hits,
        stats.architecture_misses,
        stats.repo_analysis_hits,
        stats.repo_analysis_misses,
        stats.architecture_analysis_hits,
        stats.architecture_analysis_misses
    );
    if stats.lock_waits > 0 || stats.stale_lock_recovers > 0 {
        summary.push_str(&format!(
            "; waited for another run {} time(s); recovered {} stale cache lock(s)",
            stats.lock_waits, stats.stale_lock_recovers
        ));
    }
    Some(summary)
}

pub(super) fn with_cache_status_notifier<T>(
    notifier: impl Fn(&str) + 'static,
    f: impl FnOnce() -> T,
) -> T {
    CACHE_STATUS_NOTIFIER.with(|cell| {
        let previous = cell.replace(Some(Box::new(notifier)));
        let result = f();
        let _ = cell.replace(previous);
        result
    })
}

pub(super) fn emit_cache_status(message: &str) {
    CACHE_STATUS_NOTIFIER.with(|cell| {
        if let Some(notifier) = cell.borrow().as_ref() {
            notifier(message);
        }
    });
}

pub(super) fn record_repo_hit() {
    update_stats(|stats| stats.repo_hits += 1);
}

pub(super) fn record_repo_miss() {
    update_stats(|stats| stats.repo_misses += 1);
}

pub(super) fn record_architecture_hit() {
    update_stats(|stats| stats.architecture_hits += 1);
}

pub(super) fn record_architecture_miss() {
    update_stats(|stats| stats.architecture_misses += 1);
}

pub(super) fn record_repo_analysis_hit() {
    update_stats(|stats| stats.repo_analysis_hits += 1);
}

pub(super) fn record_repo_analysis_miss() {
    update_stats(|stats| stats.repo_analysis_misses += 1);
}

pub(super) fn record_architecture_analysis_hit() {
    update_stats(|stats| stats.architecture_analysis_hits += 1);
}

pub(super) fn record_architecture_analysis_miss() {
    update_stats(|stats| stats.architecture_analysis_misses += 1);
}

pub(super) fn record_lock_wait() {
    update_stats(|stats| stats.lock_waits += 1);
}

pub(super) fn record_stale_lock_recover() {
    update_stats(|stats| stats.stale_lock_recovers += 1);
}

fn cache_stats() -> &'static Mutex<CacheStats> {
    static CACHE_STATS: OnceLock<Mutex<CacheStats>> = OnceLock::new();
    CACHE_STATS.get_or_init(|| Mutex::new(CacheStats::default()))
}

fn update_stats(f: impl FnOnce(&mut CacheStats)) {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    f(&mut stats);
}
