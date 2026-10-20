//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Chat ids: a direct chat between two users has the id `"dm:" + smaller_name + ":" + larger_name` (names compared
//! as strings); a group chat has the id `"group:" + group_id`.

#[derive(Debug, PartialEq, Eq)]
pub enum ChatError {
    /// unknown user, group or message
    Unknown,
    /// the actor is not allowed to do this (non-admin managing members, non-member sending, admin leaving)
    NotAllowed,
    /// group id already used
    AlreadyExists,
}

pub type Chat<T> = Result<T, ChatError>;

pub trait ChatService: Send + Sync {
    fn register(&self, user: &str);

    /// Goes online. Everything waiting for this user is delivered now.
    fn connect(&self, user: &str) -> Chat<()>;

    /// Goes offline; messages sent from now on wait until the next connect.
    fn disconnect(&self, user: &str) -> Chat<()>;

    /// Sends a direct message; returns the message id. It is delivered immediately if the recipient is online.
    fn send_direct(&self, from: &str, to: &str, text: &str) -> Chat<String>;

    /// The creator becomes the group's admin and is a member automatically; the other members are added.
    fn create_group(&self, group_id: &str, admin: &str, members: &[&str]) -> Chat<()>;

    /// Only the admin.
    fn add_member(&self, actor: &str, group_id: &str, user: &str) -> Chat<()>;

    /// The admin may remove anybody; any member may remove themselves (leave). The admin cannot leave.
    fn remove_member(&self, actor: &str, group_id: &str, user: &str) -> Chat<()>;

    /// Sender must currently be a member. Returns the message id.
    fn send_group(&self, from: &str, group_id: &str, text: &str) -> Chat<String>;

    /// The messages the user took part in (sent, or was a recipient of when they were sent) in this chat, in the
    /// order they were sent, formatted "messageId|sender|text".
    fn history(&self, user: &str, chat_id: &str) -> Chat<Vec<String>>;

    /// Marks every message addressed to the user in the chat as read (and delivered).
    fn read(&self, user: &str, chat_id: &str) -> Chat<()>;

    /// Messages addressed to the user in the chat that they have not read yet.
    fn unread_count(&self, user: &str, chat_id: &str) -> Chat<usize>;

    /// Status from the sender's point of view: "SENT" (somebody it is addressed to has not got it yet), "DELIVERED"
    /// (all recipients have it) or "READ" (all recipients have read it). A message nobody receives (a group of one)
    /// is "READ". Recipients who have left the group since are ignored.
    fn status_of(&self, message_id: &str) -> Chat<String>;
}
