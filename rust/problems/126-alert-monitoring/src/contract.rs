//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub const GT: &str = "GT";
pub const GTE: &str = "GTE";
pub const LT: &str = "LT";
pub const LTE: &str = "LTE";

pub const AVG: &str = "AVG";
pub const MAX: &str = "MAX";
pub const MIN: &str = "MIN";
pub const COUNT: &str = "COUNT";
pub const SUM: &str = "SUM";

/// Severities, lowest to highest: INFO < WARN < CRITICAL.
pub const INFO: &str = "INFO";
pub const WARN: &str = "WARN";
pub const CRITICAL: &str = "CRITICAL";

/// Rule states.
pub const OK: &str = "OK";
pub const PENDING: &str = "PENDING";
pub const FIRING: &str = "FIRING";
pub const RESOLVED: &str = "RESOLVED";

#[derive(Debug, PartialEq, Eq)]
pub enum MonitorError {
    /// blank metric / rule id / channel name, NaN or infinite value or threshold, unknown comparator /
    /// aggregation / severity name (names are case-sensitive), window <= 0, for < 0, cooldown < 0
    InvalidArgument,
    /// the rule id is already defined
    DuplicateRule,
    /// the rule id is not defined
    UnknownRule,
    /// the channel name is already registered
    DuplicateChannel,
}

/// Delivery sink for alert notifications. PROVIDED: the tests implement it and record what they receive.
/// Not part of your design. A delivery may fail (`Err`).
pub trait Notifier: Send + Sync {
    /// One delivery to one channel. `state` is `FIRING` or `RESOLVED`; `severity` is the rule's severity; `value` is
    /// the rule's aggregate at the evaluation that caused the notification and `time_ms` the clock time of that
    /// evaluation.
    fn notify(
        &self,
        channel_name: &str,
        rule_id: &str,
        state: &str,
        severity: &str,
        value: f64,
        time_ms: u64,
    ) -> Result<(), String>;
}

pub trait AlertMonitor: Send + Sync {
    /// Stores a sample of the metric stamped with the clock's current time in milliseconds. Callable from many
    /// threads, also while others evaluate. `InvalidArgument` for a blank metric or a NaN/infinite value (nothing
    /// is stored).
    fn record(&self, metric: &str, value: f64) -> Result<(), MonitorError>;

    /// Registers a rule in state `OK` with an empty history. Its condition is
    /// `aggregation(samples of metric in (now - window_ms, now]) comparator threshold`.
    /// `InvalidArgument` for a blank rule id or metric, an unknown comparator / aggregation / severity name, a
    /// NaN/infinite threshold, `window_ms <= 0`, `for_ms < 0` or `cooldown_ms < 0`; `DuplicateRule` for an
    /// existing id. Nothing changes on error.
    #[allow(clippy::too_many_arguments)]
    fn define_rule(
        &self,
        rule_id: &str,
        metric: &str,
        comparator: &str,
        threshold: f64,
        window_ms: i64,
        aggregation: &str,
        severity: &str,
        for_ms: i64,
        cooldown_ms: i64,
    ) -> Result<(), MonitorError>;

    /// Removes the rule with its state, history, silence and counters. `UnknownRule` if not defined.
    fn delete_rule(&self, rule_id: &str) -> Result<(), MonitorError>;

    /// Registers a channel that receives every notification whose severity is >= `min_severity`.
    /// `InvalidArgument` for a blank name or an unknown severity name, `DuplicateChannel` for a registered name.
    fn register_channel(&self, name: &str, min_severity: &str) -> Result<(), MonitorError>;

    /// Evaluates every rule once at the clock's current time (see the README for the state machine).
    fn evaluate(&self);

    /// Current state of the rule. `UnknownRule` if not defined.
    fn state(&self, rule_id: &str) -> Result<String, MonitorError>;

    /// Ids of the rules currently in state `FIRING` (silenced or not), highest severity first, then rule id
    /// ascending (plain string order).
    fn active_alerts(&self) -> Vec<String>;

    /// Every state transition of the rule, oldest first, as `"<time_ms>:<STATE>"` where STATE is the state entered
    /// (for example `"61000:FIRING"`). `UnknownRule` if not defined.
    fn history(&self, rule_id: &str) -> Result<Vec<String>, MonitorError>;

    /// Suppresses notifications for the rule while `clock time < until_ms`. State tracking continues. A later call
    /// replaces the earlier one. `UnknownRule` if not defined.
    fn silence(&self, rule_id: &str, until_ms: u64) -> Result<(), MonitorError>;

    /// Number of samples currently retained for the metric (0 for a metric never recorded).
    fn sample_count(&self, metric: &str) -> usize;

    /// How many FIRING notifications of the rule were withheld because of its cooldown. `UnknownRule` if not defined.
    fn suppressed_count(&self, rule_id: &str) -> Result<usize, MonitorError>;
}
