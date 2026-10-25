//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Every tab starts on `about:blank`. All urls are compared as exact, case-sensitive strings.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserError {
    /// the tab id was never issued or the tab was already closed
    UnknownTab,
    /// blank url / blank search text, negative steps, limit or k, `from_ms > to_ms`
    InvalidArgument,
}

pub trait BrowserHistory: Send + Sync {
    /// Opens a tab and returns its id: 1, 2, 3, ... in order of opening, never reused, even after a close.
    /// The new tab shows `about:blank` as its only entry; that opening is NOT recorded in the global history.
    fn open_tab(&self) -> u32;

    /// Closes the tab. Its global history entries stay. The id is unknown from now on.
    fn close_tab(&self, tab_id: u32) -> Result<(), BrowserError>;

    /// Navigates the tab to `url`: the tab's forward entries are discarded, the url becomes the new current
    /// entry, and one visit is recorded in the global history stamped with the clock's current millis and the
    /// tab id. If `url` equals the tab's current url the tab is left exactly as it was (no new entry, forward
    /// entries kept), but the visit is still recorded globally.
    /// Unknown tab: `UnknownTab`; blank url: `InvalidArgument`. A failed call changes nothing.
    fn visit(&self, tab_id: u32, url: &str) -> Result<(), BrowserError>;

    /// Moves the tab back by up to `steps` entries (fewer if there are not that many) and returns the url now
    /// shown. Never fails because of a lack of entries; `steps == 0` just returns the current url.
    /// `steps < 0` is `InvalidArgument`. The global history is not touched.
    fn back(&self, tab_id: u32, steps: i32) -> Result<String, BrowserError>;

    /// Mirror of `back`: moves forward by up to `steps` entries and returns the url now shown.
    fn forward(&self, tab_id: u32, steps: i32) -> Result<String, BrowserError>;

    /// The url the tab shows now.
    fn current_url(&self, tab_id: u32) -> Result<String, BrowserError>;

    /// True if the tab has at least one earlier entry.
    fn can_go_back(&self, tab_id: u32) -> Result<bool, BrowserError>;

    /// True if the tab has at least one later entry.
    fn can_go_forward(&self, tab_id: u32) -> Result<bool, BrowserError>;

    /// The most recent `limit` global entries, most recent first, each formatted `"time|url|tabId"`
    /// (for example `"1000|https://a.com|2"`). Fewer if fewer exist; `limit == 0` gives an empty list.
    /// `limit < 0` is `InvalidArgument`.
    fn history(&self, limit: i32) -> Result<Vec<String>, BrowserError>;

    /// Global entries whose time lies in `[from_ms, to_ms]` (both inclusive), most recent first, same format as
    /// `history`. `from_ms > to_ms` is `InvalidArgument`.
    fn history_between(&self, from_ms: u64, to_ms: u64) -> Result<Vec<String>, BrowserError>;

    /// Distinct urls of the global history that contain `substring`, ignoring case, ordered by their latest
    /// visit, most recent first. A blank `substring` is `InvalidArgument`.
    fn search(&self, substring: &str) -> Result<Vec<String>, BrowserError>;

    /// At most `k` distinct urls of the global history ordered by number of global entries (most first), then by
    /// the time of the latest entry (later first), then by url (ascending, natural `str` order).
    /// `k < 0` is `InvalidArgument`.
    fn most_visited(&self, k: i32) -> Result<Vec<String>, BrowserError>;

    /// Removes every global entry for `url` and returns how many were removed (0 if none). Open tabs keep their
    /// back/forward entries. A blank url is `InvalidArgument`.
    fn delete_from_history(&self, url: &str) -> Result<usize, BrowserError>;

    /// Removes every global entry with time `>= since_ms` and returns how many were removed.
    fn clear_history(&self, since_ms: u64) -> usize;
}
