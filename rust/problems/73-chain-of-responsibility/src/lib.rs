//! Chain of Responsibility — handlers are tried in order until one decides.
//!
//! Rust adaptation: a pointer-linked list of handlers (each owning `Option<Box<next>>`) fights ownership, so the
//! chain is an ordered `Vec<Box<dyn Decider>>` owned by an `ApprovalChain`. The behaviour is the same: the request
//! walks the handlers in order and stops at the first decision.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Rejects (`"REJECTED:policy violation"`) any request with amount <= 0 or category "gambling" (any case). Passes
/// everything else on.
pub struct PolicyGuard;

impl PolicyGuard {
    pub fn new() -> PolicyGuard {
        todo!()
    }
}

impl Decider for PolicyGuard {
    fn decide(&self, request: &ExpenseRequest) -> Option<String> {
        todo!()
    }
}

/// Approves (`"APPROVED:team lead"`) amounts up to 50000 cents inclusive - except category "travel" (any case),
/// which always goes up. Passes the rest on.
pub struct TeamLead;

impl TeamLead {
    pub fn new() -> TeamLead {
        todo!()
    }
}

impl Decider for TeamLead {
    fn decide(&self, request: &ExpenseRequest) -> Option<String> {
        todo!()
    }
}

/// Approves (`"APPROVED:manager"`) amounts up to 500000 cents inclusive. Passes the rest on.
pub struct Manager;

impl Manager {
    pub fn new() -> Manager {
        todo!()
    }
}

impl Decider for Manager {
    fn decide(&self, request: &ExpenseRequest) -> Option<String> {
        todo!()
    }
}

/// Approves (`"APPROVED:director"`) amounts up to 5000000 cents inclusive. Passes the rest on.
pub struct Director;

impl Director {
    pub fn new() -> Director {
        todo!()
    }
}

impl Decider for Director {
    fn decide(&self, request: &ExpenseRequest) -> Option<String> {
        todo!()
    }
}

/// An ordered chain of handlers.
pub struct ApprovalChain;

impl ApprovalChain {
    /// An empty chain: every request is rejected with the "no approver" message.
    pub fn new() -> ApprovalChain {
        todo!()
    }

    /// Appends a handler to the END of the chain.
    pub fn then(self, handler: Box<dyn Decider>) -> ApprovalChain {
        todo!()
    }

    /// Asks the handlers in order; the first decision is the answer and the rest of the chain is NOT asked. With no
    /// decision the answer is `"REJECTED:no approver for <amount_cents>"`.
    pub fn handle(&self, request: &ExpenseRequest) -> String {
        todo!()
    }
}
