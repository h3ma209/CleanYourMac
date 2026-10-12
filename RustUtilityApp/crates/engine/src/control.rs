use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::Notify;

use crate::error::ScanError;

/// Pause and cancel gate checked before each file, matching a scan execution control.
#[derive(Debug)]
pub struct ExecutionControl {
    paused: AtomicBool,
    canceled: AtomicBool,
    notify: Notify,
}

impl ExecutionControl {
    pub fn new() -> Self {
        Self {
            paused: AtomicBool::new(false),
            canceled: AtomicBool::new(false),
            notify: Notify::new(),
        }
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn cancel(&self) {
        self.canceled.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn is_canceled(&self) -> bool {
        self.canceled.load(Ordering::SeqCst)
    }

    /// Wait while paused. Returns [`ScanError::Canceled`] once cancel is set.
    pub async fn checkpoint(&self) -> Result<(), ScanError> {
        loop {
            if self.is_canceled() {
                return Err(ScanError::Canceled);
            }
            if !self.is_paused() {
                return Ok(());
            }
            self.notify.notified().await;
        }
    }
}

impl Default for ExecutionControl {
    fn default() -> Self {
        Self::new()
    }
}
