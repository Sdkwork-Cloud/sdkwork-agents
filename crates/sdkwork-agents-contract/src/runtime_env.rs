use std::sync::Mutex;

static ENV_TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Serialize integration tests that mutate process environment variables.
pub fn env_test_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
