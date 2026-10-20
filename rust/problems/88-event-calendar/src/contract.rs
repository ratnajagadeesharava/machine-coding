//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Times are minutes (`u64`) since some epoch; an event occupies the half-open range `[start, end)`, so two events
//! that merely touch (one ends when the other starts) do not conflict. Users exist implicitly by being named.

#[derive(Debug, PartialEq, Eq)]
pub enum CalError {
    /// end <= start, blank title, duplicate people (the organizer may not be an attendee), bad recurrence
    InvalidInput,
    /// somebody already has an overlapping event; nothing was created
    Conflict,
    UnknownEvent,
    /// cancel by a non-organizer, or decline by the organizer
    PermissionDenied,
}

pub type Cal<T> = Result<T, CalError>;

pub trait Calendar: Send + Sync {
    /// Schedules an event for the organizer and the attendees and returns its id. Conflicts for ANY person create
    /// nothing.
    fn schedule(
        &self,
        organizer: &str,
        title: &str,
        start: u64,
        end: u64,
        attendees: &[&str],
    ) -> Cal<String>;

    /// Schedules `count` occurrences (count >= 1) starting at `start`, each repeating `every` minutes later
    /// (`every >= end - start`). All-or-nothing. Returns the ids in time order.
    #[allow(clippy::too_many_arguments)]
    fn schedule_recurring(
        &self,
        organizer: &str,
        title: &str,
        start: u64,
        end: u64,
        attendees: &[&str],
        every: u64,
        count: usize,
    ) -> Cal<Vec<String>>;

    /// Only the organizer may cancel; the event disappears for everybody.
    fn cancel(&self, user: &str, event_id: &str) -> Cal<()>;

    /// An attendee leaves the event; it stays for the others. The organizer cannot decline.
    fn decline(&self, user: &str, event_id: &str) -> Cal<()>;

    /// The user's events overlapping `[from, to)`, sorted by start then title, formatted "start-end:title".
    fn agenda(&self, user: &str, from: u64, to: u64) -> Vec<String>;

    /// Windows inside `[from, to)` where ALL the users are free for at least `duration` minutes, merged and
    /// sorted, formatted "start-end".
    fn common_free_slots(&self, users: &[&str], from: u64, to: u64, duration: u64) -> Vec<String>;
}
