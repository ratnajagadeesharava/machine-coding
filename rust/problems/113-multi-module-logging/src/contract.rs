//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Fixed values (case-sensitive): modules `AUTH`, `PAYMENTS`, `SEARCH`; destinations `FILE`, `TOOL`;
//! levels `DEBUG < INFO < WARN < ERROR`.
//!
//! Line format, used for what a sink receives and for what `query` returns:
//! `<seq>|<MODULE>|<DESTINATION>|<LEVEL>|<activity>` (for example `7|AUTH|FILE|WARN|bad password`).
//! The activity text is kept verbatim and may itself contain `|`.

#[derive(Debug, PartialEq, Eq)]
pub enum LogError {
    /// Unknown module / destination / level, blank activity, or `from_seq > to_seq`.
    InvalidArgument,
    /// The destination sink did not write the line (it returned `false`). No sequence number was consumed
    /// and the log was not stored.
    WriteFailed,
    /// The service has been closed.
    Closed,
}

/// A simulated destination (a log file or a monitoring tool). Implemented by the tests, not by you.
pub trait Sink: Send + Sync {
    /// Writes one line. Returns `true` when the line was durably written, `false` when the write failed.
    /// The line has the format documented above.
    fn write(&self, line: &str) -> bool;
}

pub trait LogService: Send + Sync {
    /// Accepts a log and returns its sequence number once the destination sink has written it successfully.
    /// Sequence numbers start at 1 and are gap-free: a log whose sink write fails is NOT given a number, it is not
    /// stored, and the next successful log gets the next number. Sinks receive lines in increasing sequence order.
    /// Returns `Err(WriteFailed)` when the sink write failed.
    /// Blocks the caller while the shared bounded buffer is full or until the write has been attempted.
    /// A log that was acknowledged is immediately visible to `query`.
    ///
    /// Errors: `InvalidArgument` if module, destination or level is not one of the fixed values or the activity
    /// is blank (nothing is consumed or written); `Closed` if the service has been closed.
    fn log(
        &self,
        module: &str,
        destination: &str,
        level: &str,
        activity: &str,
    ) -> Result<u64, LogError>;

    /// Returns the acknowledged logs matching every `Some` filter, as lines, in increasing sequence order.
    /// `level` is a threshold: logs at that level or above match. `from_seq` and `to_seq` are inclusive bounds.
    /// The result is a snapshot the caller owns.
    ///
    /// Errors: `InvalidArgument` for an unknown destination, module or level, or when both bounds are given
    /// and `from_seq > to_seq`.
    fn query(
        &self,
        destination: Option<&str>,
        module: Option<&str>,
        level: Option<&str>,
        from_seq: Option<u64>,
        to_seq: Option<u64>,
    ) -> Result<Vec<String>, LogError>;

    /// Stops accepting logs, lets everything already accepted finish, and stops any background thread before
    /// returning. Safe to call more than once. After it returns, `log` fails with `Closed` while `query`
    /// keeps working.
    fn close(&self);
}
