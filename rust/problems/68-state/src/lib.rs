//! State — each ticket state is its own type implementing a state trait; `Ticket` delegates to the current one.
//!
//! Keep everything in this file: the tests scan it for state-switching code and count the `impl ... for ...` blocks.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

/// A support ticket whose behaviour depends on its current state.
///
/// ```text
///             assign          hold         resume          resolve       reopen            close     comment
/// NEW         -> IN_PROGRESS  illegal      illegal         illegal       illegal           illegal   ok
/// IN_PROGRESS reassign        -> ON_HOLD   illegal         -> RESOLVED   illegal           illegal   ok
/// ON_HOLD     reassign        illegal      -> IN_PROGRESS  illegal       illegal           illegal   ok
/// RESOLVED    illegal         illegal      illegal         illegal       -> IN_PROGRESS *  -> CLOSED ok
/// CLOSED      illegal         illegal      illegal         illegal       illegal           illegal   illegal
/// ```
/// (* a ticket can be reopened at most twice; the third reopen is `Err(TicketError::ReopenLimit)`.)
///
/// Order of checks for every operation: a blank ARGUMENT -> `InvalidArgument` first, then an operation not allowed
/// in the current state -> `Illegal("cannot <operation> a ticket in state <STATE>")`.
pub struct Ticket;

impl Ticket {
    /// Blank id -> `Err(TicketError::InvalidArgument)`.
    pub fn new(id: &str, clock: Arc<dyn Clock>) -> Result<Ticket, TicketError> {
        todo!()
    }

    pub fn id(&self) -> &str {
        todo!()
    }

    /// "NEW", "IN_PROGRESS", "ON_HOLD", "RESOLVED" or "CLOSED".
    pub fn state(&self) -> String {
        todo!()
    }

    pub fn assigned_to(&self) -> Option<&str> {
        todo!()
    }

    /// NEW -> IN_PROGRESS and records the agent; in IN_PROGRESS or ON_HOLD it only changes the agent.
    pub fn assign(&mut self, agent: &str) -> Result<(), TicketError> {
        todo!()
    }

    /// IN_PROGRESS -> ON_HOLD; adds the comment `"HOLD: <reason>"`.
    pub fn hold(&mut self, reason: &str) -> Result<(), TicketError> {
        todo!()
    }

    /// ON_HOLD -> IN_PROGRESS.
    pub fn resume(&mut self) -> Result<(), TicketError> {
        todo!()
    }

    /// IN_PROGRESS -> RESOLVED; adds the comment `"RESOLVED: <note>"` and remembers when.
    pub fn resolve(&mut self, note: &str) -> Result<(), TicketError> {
        todo!()
    }

    /// RESOLVED -> IN_PROGRESS; adds the comment `"REOPENED"`. At most twice per ticket.
    pub fn reopen(&mut self) -> Result<(), TicketError> {
        todo!()
    }

    /// RESOLVED -> CLOSED.
    pub fn close(&mut self) -> Result<(), TicketError> {
        todo!()
    }

    /// Adds the text as a comment in every state except CLOSED.
    pub fn comment(&mut self, text: &str) -> Result<(), TicketError> {
        todo!()
    }

    /// Comments in the order they were added.
    pub fn comments(&self) -> &[String] {
        todo!()
    }

    /// A RESOLVED ticket that has been resolved for 72 hours or more (by the clock given at construction) becomes
    /// CLOSED and this returns true. In any other situation it changes nothing and returns false.
    pub fn auto_close_if_stale(&mut self) -> bool {
        todo!()
    }
}
