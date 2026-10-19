//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum FlagError {
    UnknownFlag,
    AlreadyDefined,
    /// blank name / blank user id / rollout outside 0..=100
    InvalidArgument,
}

pub trait FlagService: Send + Sync {
    /// Creates a boolean flag.
    fn define(&self, flag: &str, default_value: bool) -> Result<(), FlagError>;

    /// Appends a targeting rule: users whose attribute `attribute` equals `value` get `result`.
    /// Rules are evaluated in the order they were added; the first matching rule wins.
    fn add_rule(&self, flag: &str, attribute: &str, value: &str, result: bool) -> Result<(), FlagError>;

    /// Percentage rollout 0..=100 for users that match no rule. Once a rollout is set, users inside the
    /// percentage get true and everybody else false (the default value is ignored). Each user must land in the
    /// same bucket every time for the same flag, buckets of different flags are independent, and raising the
    /// percentage never turns anybody off.
    fn set_rollout(&self, flag: &str, percent: u32) -> Result<(), FlagError>;

    /// A killed flag is false for everybody, whatever its rules, rollout or default say. Reversible.
    fn set_killed(&self, flag: &str, killed: bool) -> Result<(), FlagError>;

    /// Evaluation order: killed -> first matching rule -> rollout (if set) -> default.
    fn is_enabled(&self, flag: &str, user_id: &str, attributes: &[(&str, &str)]) -> Result<bool, FlagError>;
}
