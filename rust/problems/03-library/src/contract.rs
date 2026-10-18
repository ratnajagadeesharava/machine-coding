//! Thin test entry point. Your domain model lives behind it.

pub trait LibraryApi {
    /// Adds a book, or adds copies if the ISBN already exists.
    fn add_book(&mut self, isbn: &str, title: &str, copies: u32);

    fn add_member(&mut self, member_id: &str);

    /// Returns the loan id, or `None` if refused.
    fn checkout(&mut self, member_id: &str, isbn: &str) -> Option<String>;

    /// Closes the loan and returns the fine (0 if on time), or `None` if unknown/already returned.
    fn return_loan(&mut self, loan_id: &str) -> Option<u64>;

    /// Due time in epoch millis.
    fn due_at(&self, loan_id: &str) -> Option<u64>;

    /// Queues the member for the book. Returns true only if queued.
    fn reserve(&mut self, member_id: &str, isbn: &str) -> bool;

    fn available_copies(&self, isbn: &str) -> u32;

    /// ISBNs whose title contains the query (case-insensitive), sorted ascending.
    fn search(&self, query: &str) -> Vec<String>;
}
