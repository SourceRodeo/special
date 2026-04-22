/**
@module SPECIAL.CACHE.LOCK
Coordinates single-flight cache fills with stale-lock recovery and heartbeat refresh in `SPECIAL.CACHE`.
*/
// @fileimplements SPECIAL.CACHE.LOCK
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;

use super::stats;

pub(super) fn cache_lock_path(cache_path: &Path) -> PathBuf {
    let mut path = cache_path.as_os_str().to_os_string();
    path.push(".lock");
    PathBuf::from(path)
}

pub(super) fn acquire_cache_fill_lock(cache_path: &Path) -> Result<CacheFillGuard> {
    acquire_cache_fill_lock_with_hooks(cache_path, SystemTime::now, std::thread::sleep)
}

pub(super) fn acquire_cache_fill_lock_with_hooks(
    cache_path: &Path,
    now: impl Fn() -> SystemTime,
    sleep: impl Fn(Duration),
) -> Result<CacheFillGuard> {
    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let lock_path = cache_lock_path(cache_path);
    let mut waited = false;
    loop {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => {
                let owner_token = next_cache_lock_owner();
                refresh_cache_lock(&lock_path, &owner_token)?;
                return Ok(CacheFillGuard {
                    path: lock_path.clone(),
                    owner_token: owner_token.clone(),
                    heartbeat: Some(start_cache_lock_heartbeat(
                        lock_path,
                        owner_token,
                        super::CACHE_LOCK_REFRESH_INTERVAL,
                    )),
                });
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                if !waited {
                    stats::record_lock_wait();
                    stats::emit_cache_status(&format!(
                        "another special run is already building {}; waiting to reuse its cached result",
                        cache_entry_label(cache_path)
                    ));
                    waited = true;
                }
                if cache_lock_is_stale_at(&lock_path, now()) {
                    stats::record_stale_lock_recover();
                    stats::emit_cache_status(&format!(
                        "recovered an abandoned cache lock for {}; rebuilding it now",
                        cache_entry_label(cache_path)
                    ));
                    let _ = fs::remove_file(&lock_path);
                    continue;
                }
                sleep(super::CACHE_LOCK_POLL_INTERVAL);
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn cache_lock_is_stale_at(lock_path: &Path, now: SystemTime) -> bool {
    let Ok(metadata) = fs::metadata(lock_path) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    let Ok(elapsed) = now.duration_since(modified) else {
        return false;
    };
    elapsed > super::CACHE_LOCK_STALE_AFTER
}

pub(super) struct CacheFillGuard {
    pub(super) path: PathBuf,
    pub(super) owner_token: String,
    pub(super) heartbeat: Option<CacheLockHeartbeat>,
}

impl Drop for CacheFillGuard {
    fn drop(&mut self) {
        if let Some(heartbeat) = self.heartbeat.take() {
            heartbeat.stop();
        }
        if cache_lock_owner(&self.path).as_deref() == Some(self.owner_token.as_str()) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(super) struct CacheLockHeartbeat {
    stop: Sender<()>,
    handle: JoinHandle<()>,
}

impl CacheLockHeartbeat {
    pub(super) fn stop(self) {
        let _ = self.stop.send(());
        let _ = self.handle.join();
    }
}

pub(super) fn start_cache_lock_heartbeat(
    lock_path: PathBuf,
    owner_token: String,
    interval: Duration,
) -> CacheLockHeartbeat {
    let (stop, rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        while let Err(mpsc::RecvTimeoutError::Timeout) = rx.recv_timeout(interval) {
            if cache_lock_owner(&lock_path).as_deref() != Some(owner_token.as_str()) {
                break;
            }
            if let Err(error) = refresh_cache_lock(&lock_path, &owner_token) {
                stats::emit_cache_status(&format!(
                    "stopped refreshing cache lock {} after write failure: {error}",
                    lock_path.display()
                ));
                break;
            }
        }
    });
    CacheLockHeartbeat { stop, handle }
}

fn refresh_cache_lock(lock_path: &Path, owner_token: &str) -> Result<()> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_string());
    fs::write(
        lock_path,
        format!("owner={owner_token}\nupdated_nanos={stamp}\n"),
    )?;
    Ok(())
}

pub(super) fn cache_lock_owner(lock_path: &Path) -> Option<String> {
    let contents = fs::read_to_string(lock_path).ok()?;
    contents
        .lines()
        .find_map(|line| line.strip_prefix("owner=").map(ToString::to_string))
}

fn next_cache_lock_owner() -> String {
    static LOCK_OWNER_COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = LOCK_OWNER_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}-{counter:x}")
}

fn cache_entry_label(cache_path: &Path) -> &'static str {
    let file_name = cache_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if file_name.starts_with("parsed-repo-") {
        "repo annotations"
    } else if file_name.starts_with("parsed-architecture-") {
        "architecture declarations"
    } else if file_name.starts_with("repo-analysis-") {
        "health analysis"
    } else if file_name.starts_with("architecture-analysis-") {
        "architecture analysis"
    } else {
        "shared analysis"
    }
}
