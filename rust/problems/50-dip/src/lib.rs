//! DIP — the service gets its sender, repository and clock handed in as abstractions.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

/// Sends reminders for appointments in the next 24 hours. It never creates a gateway, a database connection or a
/// clock itself.
pub struct ReminderService;

impl ReminderService {
    pub fn new(
        sender: Box<dyn MessageSender>,
        repository: Box<dyn AppointmentRepository>,
        clock: Arc<dyn Clock>,
    ) -> ReminderService {
        todo!()
    }

    /// `now = clock.now_millis()`. Asks the repository for appointments in `[now, now + 24h]`. For each one sends
    /// `"Reminder: appointment at {time_ms}"` to its phone. If the sender returns true the appointment is marked
    /// reminded and counted; otherwise it is NOT marked (retried on a later run) and the loop carries on.
    /// Returns how many reminders were sent.
    pub fn send_due_reminders(&self) -> usize {
        todo!()
    }
}
