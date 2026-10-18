//! Provided: the abstractions the service depends on. The clock is `playground_support::Clock`.

/// Times are milliseconds since the Unix epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Appointment {
    pub id: String,
    pub phone: String,
    pub time_ms: u64,
}

/// How a text message gets out. Could be an SMS gateway, a test fake, anything.
pub trait MessageSender {
    /// True if the message was accepted for delivery.
    fn send(&self, phone: &str, text: &str) -> bool;
}

/// Where appointments live. Already-reminded appointments are not returned by `find_between`.
pub trait AppointmentRepository {
    /// Appointments whose time is in `[from_ms, to_ms]` (both inclusive) that have not been reminded yet.
    fn find_between(&self, from_ms: u64, to_ms: u64) -> Vec<Appointment>;
    fn mark_reminded(&self, appointment_id: &str);
}
