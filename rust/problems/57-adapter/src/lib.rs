//! Legacy SMS gateway as a `Notifier`.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// A `Notifier` backed by the legacy gateway.
pub struct LegacySmsAdapter;

impl LegacySmsAdapter {
    pub fn new(legacy: Box<dyn LegacySmsGateway>) -> LegacySmsAdapter {
        todo!()
    }
}

impl Notifier for LegacySmsAdapter {
    /// See README for the spec.
    fn send(&self, phone: &str, message: &str) -> Result<(), String> {
        todo!()
    }
}

/// Alerts every phone; returns the failures. See README.
pub fn alert_all(notifier: &dyn Notifier, phones: &[&str], message: &str) -> Vec<String> {
    todo!()
}
