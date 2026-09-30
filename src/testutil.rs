//! Test-only process-env isolation.
//!
//! `std::env::set_var` is process-global. Parallel nextest will flake unless
//! mutations are serialized and restored, including on panic.

use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Holds the process env lock and restores captured variables on drop.
/// The lock is taken once for all vars — a second `EnvGuard` in the same
/// scope would deadlock on the non-reentrant mutex, so multi-var cases must
/// use [`EnvGuard::apply`].
pub struct EnvGuard {
    vars: Vec<(&'static str, Option<String>)>,
    _lock: MutexGuard<'static, ()>,
}

impl EnvGuard {
    /// Set `key` to `Some(value)` or remove it (`None`), under the global
    /// env lock. Restores the previous values on drop.
    pub fn apply(vars: &[(&'static str, Option<&'static str>)]) -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev: Vec<_> = vars
            .iter()
            .map(|(k, _)| (*k, std::env::var(k).ok()))
            .collect();
        for (key, value) in vars {
            match value {
                // SAFETY: ENV_LOCK serializes set/remove for this process's tests.
                Some(v) => unsafe { std::env::set_var(key, v) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
        Self {
            vars: prev,
            _lock: lock,
        }
    }

    pub fn set(key: &'static str, value: &'static str) -> Self {
        Self::apply(&[(key, Some(value))])
    }

    pub fn remove(key: &'static str) -> Self {
        Self::apply(&[(key, None)])
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, prev) in &self.vars {
            match prev {
                Some(v) => unsafe { std::env::set_var(key, v) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }
}
