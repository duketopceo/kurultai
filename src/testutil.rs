//! Test-only process-env isolation.
//!
//! `std::env::set_var` is process-global. Parallel nextest will flake unless
//! mutations are serialized and restored, including on panic.

use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Holds the process env lock and restores one variable on drop.
pub struct EnvGuard {
    key: &'static str,
    prev: Option<String>,
    _lock: MutexGuard<'static, ()>,
}

impl EnvGuard {
    pub fn set(key: &'static str, value: &str) -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var(key).ok();
        // SAFETY: ENV_LOCK serializes set/remove for this process's tests.
        unsafe { std::env::set_var(key, value) };
        Self {
            key,
            prev,
            _lock: lock,
        }
    }

    pub fn remove(key: &'static str) -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var(key).ok();
        unsafe { std::env::remove_var(key) };
        Self {
            key,
            prev,
            _lock: lock,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => unsafe { std::env::set_var(self.key, v) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}
