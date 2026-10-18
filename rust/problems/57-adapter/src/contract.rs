//! Provided: the target trait (`Notifier`) and the legacy adaptee (`LegacySmsGateway`).

/// What the application code wants to talk to. `Err` carries a human-readable reason.
pub trait Notifier {
    fn send(&self, phone: &str, message: &str) -> Result<(), String>;
}

/// The vendor's old SMS gateway (the "adaptee"), NOT to be changed. It speaks XML and integer codes.
/// Return codes: 0 = accepted, 1 = invalid number, 2 = message too long, 3 = unauthorized, anything else = unknown.
pub trait LegacySmsGateway {
    fn submit_xml(&self, xml: &str) -> i32;
}
