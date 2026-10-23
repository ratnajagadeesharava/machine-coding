//! Appointment reminders.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

/// Sends reminders for upcoming appointments.
pub struct ReminderService;

impl ReminderService {
    pub fn new(
        sender: Box<dyn MessageSender>,
        repository: Box<dyn AppointmentRepository>,
        clock: Arc<dyn Clock>,
    ) -> ReminderService {
        todo!()
    }

    pub fn send_due_reminders(&self) -> usize {
        todo!()
    }
}
