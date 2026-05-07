/**
@module SPECIAL.MODULES.ANALYZE.STATUS
Thread-local analysis status notifier used to surface long-running repo-analysis progress through the CLI without coupling the shared analysis core to a specific renderer.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.STATUS
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

type StatusNotifier = Arc<dyn Fn(&str) + Send + Sync>;

thread_local! {
    static ANALYSIS_STATUS_NOTIFIER: RefCell<Option<StatusNotifier>> = RefCell::new(None);
}

pub(crate) fn with_analysis_status_notifier<T>(
    notifier: impl Fn(&str) + Send + Sync + 'static,
    f: impl FnOnce() -> T,
) -> T {
    ANALYSIS_STATUS_NOTIFIER.with(|cell| {
        let previous = cell.replace(Some(Arc::new(notifier)));
        let result = f();
        let _ = cell.replace(previous);
        result
    })
}

pub(crate) fn emit_analysis_status(message: &str) {
    ANALYSIS_STATUS_NOTIFIER.with(|cell| {
        if let Some(notifier) = cell.borrow().as_ref() {
            notifier(message);
        }
    });
}

pub(crate) struct ProgressHeartbeat {
    label: String,
    total: usize,
    next_at: Instant,
    interval: Duration,
}

impl ProgressHeartbeat {
    pub(crate) fn new(label: impl Into<String>, total: usize) -> Self {
        Self {
            label: label.into(),
            total,
            next_at: Instant::now() + Duration::from_secs(10),
            interval: Duration::from_secs(10),
        }
    }

    pub(crate) fn maybe_emit(&mut self, completed: usize) {
        let now = Instant::now();
        if now < self.next_at {
            return;
        }
        emit_analysis_status(&format!(
            "{}: processed {}/{} item(s)",
            self.label, completed, self.total
        ));
        self.next_at = now + self.interval;
    }

    pub(crate) fn finish(&self) {
        if self.total > 0 {
            emit_analysis_status(&format!(
                "{}: processed {}/{} item(s)",
                self.label, self.total, self.total
            ));
        }
    }

    pub(crate) fn maybe_emit_dynamic(&mut self, visited: usize, pending: usize) {
        let now = Instant::now();
        if now < self.next_at {
            return;
        }
        emit_analysis_status(&format!(
            "{}: visited {} item(s), {} pending",
            self.label, visited, pending
        ));
        self.next_at = now + self.interval;
    }

    pub(crate) fn finish_dynamic(&self, visited: usize) {
        emit_analysis_status(&format!("{}: visited {} item(s)", self.label, visited));
    }
}

pub(crate) fn with_periodic_analysis_status<T>(
    label: impl Into<String>,
    f: impl FnOnce() -> T,
) -> T {
    let Some(notifier) = current_analysis_notifier() else {
        return f();
    };
    let label = label.into();
    let started_at = Instant::now();
    let (stop_sender, stop_receiver) = mpsc::channel();
    let handle = thread::spawn(move || {
        while stop_receiver.recv_timeout(Duration::from_secs(10)).is_err() {
            notifier(&format!(
                "{label}: still running after {:.0}s",
                started_at.elapsed().as_secs_f32()
            ));
        }
    });
    let _guard = PeriodicStatusGuard {
        stop_sender,
        handle: Some(handle),
    };

    f()
}

fn current_analysis_notifier() -> Option<StatusNotifier> {
    ANALYSIS_STATUS_NOTIFIER.with(|cell| cell.borrow().as_ref().cloned())
}

struct PeriodicStatusGuard {
    stop_sender: mpsc::Sender<()>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Drop for PeriodicStatusGuard {
    fn drop(&mut self) {
        let _ = self.stop_sender.send(());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
