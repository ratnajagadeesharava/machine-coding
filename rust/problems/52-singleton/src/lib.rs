//! Singleton — exactly one `AppConfig`, created lazily and thread-safely.
//!
//! Do NOT add a `pub fn new`, `Default` or `Clone`: the only way to get an `AppConfig` is `AppConfig::instance()`.
#![allow(unused_variables)]

use std::sync::Arc;

pub struct AppConfig;

impl AppConfig {
    /// The single instance. Many threads may call this at the same moment; still only ONE is ever created.
    pub fn instance() -> Arc<AppConfig> {
        todo!()
    }

    /// How many instances have been constructed since the last `reset_for_tests()`
    /// (0 before the first `instance()` call).
    pub fn instances_created() -> usize {
        todo!()
    }

    /// Test hook: forget the current instance and zero the counter. The next `instance()` builds a fresh one.
    pub fn reset_for_tests() {
        todo!()
    }

    /// Safe to call from many threads at once.
    pub fn set(&self, key: &str, value: &str) {
        todo!()
    }

    pub fn get(&self, key: &str) -> Option<String> {
        todo!()
    }
}
