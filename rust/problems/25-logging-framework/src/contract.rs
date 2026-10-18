//! Thin test entry point. Your domain model lives behind it.

use std::sync::Arc;

/// An output destination supplied by the caller. Receives complete lines (no trailing newline).
pub trait LogSink: Send + Sync {
    fn write(&self, line: &str);
}

pub trait LoggingApi {
    /// Levels: TRACE, DEBUG, INFO, WARN, ERROR. The root logger is named "root". False if invalid.
    fn set_level(&mut self, logger_name: &str, level: &str) -> bool;

    /// Template placeholders are "{}". Ignored for an unknown level or blank logger name.
    fn log(&mut self, logger_name: &str, level: &str, template: &str, args: &[&str]);

    /// False if the name is blank or taken, or the level is unknown.
    fn add_appender(&mut self, name: &str, sink: Arc<dyn LogSink>, min_level: &str) -> bool;

    fn remove_appender(&mut self, name: &str) -> bool;
}
