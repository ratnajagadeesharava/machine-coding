//! Alerts and channels.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Writes one line to the wire per delivery: `"EMAIL to:<recipient> subject:<title> | <body>"`.
pub struct EmailChannel;

impl EmailChannel {
    pub fn new(wire: Box<dyn Fn(&str)>) -> EmailChannel {
        todo!()
    }
}

impl Channel for EmailChannel {
    fn deliver(&self, recipient: &str, title: &str, body: &str) {
        todo!()
    }
}

/// Writes `"SMS <recipient>: <text>"` where text = `title + ": " + body`, cut to 40 characters: a longer text
/// becomes its first 37 characters followed by "...".
pub struct SmsChannel;

impl SmsChannel {
    pub fn new(wire: Box<dyn Fn(&str)>) -> SmsChannel {
        todo!()
    }
}

impl Channel for SmsChannel {
    fn deliver(&self, recipient: &str, title: &str, body: &str) {
        todo!()
    }
}

/// Writes `"SLACK @<recipient> *<title>* <body>"`.
pub struct SlackChannel;

impl SlackChannel {
    pub fn new(wire: Box<dyn Fn(&str)>) -> SlackChannel {
        todo!()
    }
}

impl Channel for SlackChannel {
    fn deliver(&self, recipient: &str, title: &str, body: &str) {
        todo!()
    }
}

/// A message to be sent through a channel.
pub trait Alert {
    /// Sends (or, for digests, queues) a message for the recipient.
    fn send(&mut self, recipient: &str, message: &str);
    /// Switches the delivery channel at runtime.
    fn set_channel(&mut self, channel: Box<dyn Channel>);
}

/// Delivers immediately with the title "Info" and the message unchanged.
pub struct InfoAlert;

impl InfoAlert {
    pub fn new(channel: Box<dyn Channel>) -> InfoAlert {
        todo!()
    }
}

impl Alert for InfoAlert {
    fn send(&mut self, recipient: &str, message: &str) {
        todo!()
    }
    fn set_channel(&mut self, channel: Box<dyn Channel>) {
        todo!()
    }
}

/// Delivers immediately with the title "URGENT" and the message in UPPER CASE.
pub struct UrgentAlert;

impl UrgentAlert {
    pub fn new(channel: Box<dyn Channel>) -> UrgentAlert {
        todo!()
    }
}

impl Alert for UrgentAlert {
    fn send(&mut self, recipient: &str, message: &str) {
        todo!()
    }
    fn set_channel(&mut self, channel: Box<dyn Channel>) {
        todo!()
    }
}

/// Queues messages per recipient and delivers them together as one message on `flush`.
pub struct DigestAlert;

impl DigestAlert {
    pub fn new(channel: Box<dyn Channel>) -> DigestAlert {
        todo!()
    }

    /// Number of messages waiting for that recipient.
    pub fn pending(&self, recipient: &str) -> usize {
        todo!()
    }

    /// Delivers one message to the recipient: title "Digest (<n>)", body = the queued messages joined with "; " in
    /// order; then empties that recipient's queue and returns true. Nothing queued -> false, nothing delivered.
    pub fn flush(&mut self, recipient: &str) -> bool {
        todo!()
    }
}

impl Alert for DigestAlert {
    /// Queues the message for the recipient (nothing is delivered yet).
    fn send(&mut self, recipient: &str, message: &str) {
        todo!()
    }
    fn set_channel(&mut self, channel: Box<dyn Channel>) {
        todo!()
    }
}
