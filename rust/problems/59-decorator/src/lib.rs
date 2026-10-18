//! Decorator — each decorator wraps a `Box<dyn MessageSink>` and is itself a `MessageSink`.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

/// Sends `"[t=<clock.now_millis()>] " + message`.
pub struct TimestampingSink;

impl TimestampingSink {
    pub fn new(next: Box<dyn MessageSink>, clock: Arc<dyn Clock>) -> TimestampingSink {
        todo!()
    }
}

impl MessageSink for TimestampingSink {
    fn send(&self, message: &str) -> Result<(), SinkError> {
        todo!()
    }
}

/// Every run of 4 or more digits becomes `****`; runs of 1 to 3 digits stay.
pub struct RedactingSink;

impl RedactingSink {
    pub fn new(next: Box<dyn MessageSink>) -> RedactingSink {
        todo!()
    }
}

impl MessageSink for RedactingSink {
    fn send(&self, message: &str) -> Result<(), SinkError> {
        todo!()
    }
}

/// Tries the next sink up to `max_attempts` times. After the last failed attempt the LAST error is returned.
/// `max_attempts` is at least 1 (the constructor makes 0 behave like 1).
pub struct RetryingSink;

impl RetryingSink {
    pub fn new(next: Box<dyn MessageSink>, max_attempts: u32) -> RetryingSink {
        todo!()
    }
}

impl MessageSink for RetryingSink {
    fn send(&self, message: &str) -> Result<(), SinkError> {
        todo!()
    }
}

/// Passes messages on and counts those that were delivered (the next sink returned `Ok`).
pub struct CountingSink;

impl CountingSink {
    pub fn new(next: Box<dyn MessageSink>) -> CountingSink {
        todo!()
    }

    /// Number of successfully delivered messages.
    pub fn count(&self) -> usize {
        todo!()
    }
}

impl MessageSink for CountingSink {
    fn send(&self, message: &str) -> Result<(), SinkError> {
        todo!()
    }
}
