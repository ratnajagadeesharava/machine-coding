//! Thin test entry point for two-phase commit. Both node types run inside the deterministic simulation
//! (`playground_support::SimNet`): messages arrive through `on_message`, time through `on_tick` (once per simulated
//! millisecond). Nodes talk to each other only through the `send` function they were created with. Use interior
//! mutability (`Mutex`) for the state: the traits take `&self`.

use std::sync::Arc;

/// Stable storage of one node (provided by the tests; it survives a restart of the node: a restarted node is a NEW
/// instance created with the SAME log). Appending is atomic and durable as soon as the call returns.
pub trait DurableLog: Send + Sync {
    fn append(&self, record: &str);

    /// Everything appended so far, oldest first.
    fn records(&self) -> Vec<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Pending,
    Committed,
    Aborted,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TxError {
    /// empty writes, unknown participant, duplicate transaction id
    InvalidTransaction,
    UnknownTransaction,
}

/// Writes of one participant: `(participant id, [(key, value), ...])`.
pub type ParticipantWrites<'a> = (&'a str, &'a [(&'a str, &'a str)]);

/// The transaction coordinator.
///
/// Protocol:
/// 1. `begin`: write a BEGIN record (with the participants and their writes) to the log, then send PREPARE to every
///    participant involved. PREPAREs are re-sent every 100 ms to participants that have not voted yet.
/// 2. A NO vote decides ABORT at once. Votes of YES from all participants decide COMMIT. If no decision has been
///    reached 500 ms after begin, ABORT is decided. The decision is appended to the log BEFORE any message about it
///    is sent (that is the commit point), and then sent to every participant of the transaction.
/// 3. The decision is re-sent every 200 ms to participants that have not acknowledged it, until all have.
/// 4. A participant that asks about a transaction (INQUIRE) gets the decision if there is one, "ABORT" if the
///    coordinator has never heard of the transaction (presumed abort), and no answer while the vote is still running.
/// 5. Restart: a new instance created with a non-empty log rebuilds its transactions from it. A transaction with a
///    BEGIN but no decision is decided ABORT right away (presumed abort). Every decided transaction is announced to
///    its participants again (acknowledgements are not persisted).
///
/// The payload format is yours.
pub trait CoordinatorNode: Send + Sync {
    fn on_message(&self, from: &str, payload: &str);

    fn on_tick(&self, now_ms: u64);

    /// Starts a transaction.
    fn begin(&self, tx_id: &str, writes: &[ParticipantWrites<'_>]) -> Result<(), TxError>;

    /// `Pending` until decided.
    fn outcome(&self, tx_id: &str) -> Result<Outcome, TxError>;
}

/// A resource manager in two-phase commit: a small key-value store whose writes are applied atomically with the
/// other participants'.
///
/// Protocol:
/// * PREPARE(tx, writes): if the transaction was already prepared, answer YES again; if it was already decided or
///   refused, answer consistently (NO for ABORT/refused, YES for COMMIT). Otherwise the participant votes NO if any
///   key is locked by another prepared transaction, or any value to write equals the text `"REJECT"` (a stand-in for
///   a business rule). On YES it locks the keys and appends a PREPARED record (with the writes) to its log BEFORE
///   answering: after that it may not unilaterally change its mind.
/// * DECISION(tx, COMMIT|ABORT): COMMIT applies the writes; both release the locks; the decision is logged and
///   acknowledged (acknowledge duplicates again). A decision for a transaction that was never prepared here is
///   remembered too, so that a late PREPARE for it is refused instead of locking keys forever.
/// * In-doubt transactions (prepared, no decision) keep their locks and are asked about every 200 ms (INQUIRE), until
///   the coordinator answers.
/// * Restart: a new instance created with a non-empty log rebuilds its store, locks and in-doubt set from it.
pub trait ParticipantNode: Send + Sync {
    fn on_message(&self, from: &str, payload: &str);

    fn on_tick(&self, now_ms: u64);

    /// The last committed value of the key.
    fn get(&self, key: &str) -> Option<String>;

    /// True while a prepared (in-doubt) transaction holds the key.
    fn is_locked(&self, key: &str) -> bool;

    /// Transactions that are prepared and not decided yet, sorted.
    fn in_doubt(&self) -> Vec<String>;
}

pub type Log = Arc<dyn DurableLog>;
