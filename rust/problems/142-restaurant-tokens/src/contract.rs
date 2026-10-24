//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

/// Customer type names.
pub const DINE_IN: &str = "DINE_IN";
pub const TAKEAWAY: &str = "TAKEAWAY";
pub const VIP: &str = "VIP";

/// Status names returned by `status`.
pub const WAITING: &str = "WAITING";
pub const CALLED: &str = "CALLED";
pub const SERVED: &str = "SERVED";
pub const NO_SHOW: &str = "NO_SHOW";
pub const CANCELLED: &str = "CANCELLED";
pub const EXPIRED: &str = "EXPIRED";

/// Average service time (ms) used for estimates before any token has been served.
pub const DEFAULT_SERVICE_MS: u64 = 300_000;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenError {
    /// blank name / counter id, unknown type name, bad party size, duplicate or unknown counter, bad counter set
    InvalidArgument,
    /// no such token today
    UnknownToken,
    /// the token is not in a status that allows the operation (or a recall is not allowed)
    InvalidState,
}

pub trait TokenSystem: Send + Sync {
    /// Registers a service counter that serves only the given customer types.
    /// `InvalidArgument` if the id is blank or already registered, or the list is empty or holds a name that is
    /// not DINE_IN / TAKEAWAY / VIP.
    fn add_counter(&self, counter_id: &str, supported_types: &[&str]) -> Result<(), TokenError>;

    /// Issues the next token of the day and returns its id, `"T-<n>"`, with n = 1, 2, 3, ... The token is WAITING.
    /// `InvalidArgument` if the name is blank, the type is not one of the three names, a DINE_IN party size is
    /// outside 1..=12, or a TAKEAWAY / VIP party size is not exactly 1. Nothing changes on error.
    fn issue_token(
        &self,
        customer_name: &str,
        customer_type: &str,
        party_size: u32,
    ) -> Result<String, TokenError>;

    /// Serves the first token (in the current serving order) whose type the counter supports and returns it; that
    /// token becomes CALLED at the current clock time. Tokens the counter does not support are skipped, not
    /// consumed. `Ok(None)` when no waiting token matches. `InvalidArgument` for an unknown counter.
    fn call_next(&self, counter_id: &str) -> Result<Option<String>, TokenError>;

    /// CALLED -> SERVED, recording the service duration. `UnknownToken`, or `InvalidState` if not CALLED.
    fn mark_served(&self, token: &str) -> Result<(), TokenError>;

    /// CALLED -> NO_SHOW at the current clock time. `UnknownToken`, or `InvalidState` if not CALLED.
    fn mark_no_show(&self, token: &str) -> Result<(), TokenError>;

    /// Moves every CALLED token that has been called for MORE than grace_time_ms to NO_SHOW (stamped with the
    /// current time), and every NO_SHOW token that has been no-show for MORE than recall_window_ms to EXPIRED.
    /// Returns the tokens that became NO_SHOW in this call, ordered by token number.
    fn expire_overdue(&self) -> Vec<String>;

    /// NO_SHOW -> WAITING, once per token, at most recall_window_ms after it became NO_SHOW, at the front of its
    /// class. `UnknownToken`; `InvalidState` if the token is not NO_SHOW, was already recalled once, or the window
    /// has passed. Nothing changes on error.
    fn recall(&self, token: &str) -> Result<(), TokenError>;

    /// WAITING -> CANCELLED. `UnknownToken`, or `InvalidState` if not WAITING.
    fn cancel(&self, token: &str) -> Result<(), TokenError>;

    /// One of the status names above. `UnknownToken` for a token that does not exist today.
    fn status(&self, token: &str) -> Result<String, TokenError>;

    /// 1-based position among all WAITING tokens in the serving order. `UnknownToken`, or `InvalidState` if the
    /// token is not WAITING.
    fn position_in_queue(&self, token: &str) -> Result<usize, TokenError>;

    /// (position - 1) * average service time, in ms. Same errors as `position_in_queue`.
    fn estimated_wait_ms(&self, token: &str) -> Result<u64, TokenError>;

    /// All WAITING tokens in serving order.
    fn queue_snapshot(&self) -> Vec<String>;

    /// `"issued:served:noShow:cancelled:waiting"`.
    fn stats(&self) -> String;

    /// Starts a new day: forgets every token and statistic, restarts numbering at 1. Counters stay registered.
    fn reset_day(&self);
}
