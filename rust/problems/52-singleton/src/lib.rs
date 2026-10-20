//! Application-wide configuration.
#![allow(unused_variables)]

use std::sync::Arc;

pub struct AppConfig;

impl AppConfig {
    pub fn instance() -> Arc<AppConfig> {
        todo!()
    }

    pub fn instances_created() -> usize {
        todo!()
    }

    /// Test hook.
    pub fn reset_for_tests() {
        todo!()
    }

    pub fn set(&self, key: &str, value: &str) {
        todo!()
    }

    pub fn get(&self, key: &str) -> Option<String> {
        todo!()
    }
}
