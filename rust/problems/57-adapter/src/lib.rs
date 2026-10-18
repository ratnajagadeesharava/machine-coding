//! Adapter — make the legacy gateway usable as a `Notifier`.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Wraps (has-a) a legacy gateway and presents it as a `Notifier`.
pub struct LegacySmsAdapter;

impl LegacySmsAdapter {
    pub fn new(legacy: Box<dyn LegacySmsGateway>) -> LegacySmsAdapter {
        todo!()
    }
}

impl Notifier for LegacySmsAdapter {
    /// Blank phone -> `Err("invalid number")`; then blank message -> `Err("empty message")`; in both cases the
    /// legacy gateway is NOT called. Otherwise calls `submit_xml` exactly once with
    /// `<sms><to>PHONE</to><body>MESSAGE</body></sms>` where both texts have & < > " ' escaped as
    /// `&amp;` `&lt;` `&gt;` `&quot;` `&apos;` (the ampersand first). Code 0 -> `Ok(())`; 1 -> "invalid number";
    /// 2 -> "message too long"; 3 -> "unauthorized"; any other code N -> "unknown error (code N)".
    fn send(&self, phone: &str, message: &str) -> Result<(), String> {
        todo!()
    }
}

/// Application code: it only knows the `Notifier` trait. Sends the message to every phone, in order, and returns
/// one entry `"<phone>: <error>"` per failed delivery (empty when everything went out).
pub fn alert_all(notifier: &dyn Notifier, phones: &[&str], message: &str) -> Vec<String> {
    todo!()
}
