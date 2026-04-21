/**
@module SPECIAL.MODULES.ANALYZE.STATUS
Thread-local analysis status notifier used to surface long-running repo-analysis progress through the CLI without coupling the shared analysis core to a specific renderer.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.STATUS
use std::cell::RefCell;

thread_local! {
    static ANALYSIS_STATUS_NOTIFIER: RefCell<Option<Box<dyn Fn(&str)>>> = RefCell::new(None);
}

pub(crate) fn with_analysis_status_notifier<T>(
    notifier: impl Fn(&str) + 'static,
    f: impl FnOnce() -> T,
) -> T {
    ANALYSIS_STATUS_NOTIFIER.with(|cell| {
        let previous = cell.replace(Some(Box::new(notifier)));
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
