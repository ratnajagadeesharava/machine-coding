//! Chain of Responsibility — expense approval.
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

/// A chain of approval handlers.
pub struct ApprovalChain;

impl ApprovalChain {
    /// An empty chain.
    pub fn new() -> ApprovalChain {
        todo!()
    }

    /// Appends a handler to the chain.
    pub fn then(self, handler: Box<dyn Decider>) -> ApprovalChain {
        todo!()
    }

    /// Returns the decision text.
    pub fn handle(&self, request: &ExpenseRequest) -> String {
        todo!()
    }
}
