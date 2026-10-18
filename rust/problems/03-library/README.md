# 03 · Library Management (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

A public library wants a system that tracks its books, its members, who has borrowed what, what is overdue,
and who is waiting for a popular title. Design and build the core.

## Requirements

1. The library holds **books** identified by ISBN, each with a title and a number of identical **copies**. Adding a book whose ISBN exists adds copies.
2. **Members** are registered by id.
3. A member can **check out** a book if a copy is available to them. A successful checkout creates a **loan** with a due date **14 days** after checkout.
4. A member may hold at most **3** active loans, and at most one active loan per ISBN.
5. A loan is closed by **returning** it. The fine is **50 per full overdue day** (0 if returned on or before the due time, or less than 24 h late).
6. When no copy is available, a member can **reserve** the book. Returned copies are **held** for waiting members in first-come-first-served order; a held copy cannot be borrowed by anyone else.
7. A member cannot reserve a book that is available, already reserved by them, or that they currently have on loan.
8. Books can be searched by (case-insensitive) title substring.
9. Time comes from the `Clock` given to the factory.

## Constraints

- Up to 100,000 books and 10,000 members; `search` must not be a noticeable linear scan on every keystroke for 10k titles (a scan is acceptable at this size, but be ready to discuss).
- Failed operations must not change any state.

## Examples

```
addBook("111", "Clean Code", 1); addMember("alice"); addMember("bob")
checkout(alice, "111") → loan L1    availableCopies("111") = 0
reserve(bob, "111")    → true
returnLoan(L1) after 16 days → fine 100 (2 full overdue days)
checkout(carol, "111") → refused (copy is held for bob)       checkout(bob, "111") → loan L2
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Unknown member or ISBN, no available copy, limit reached, or a duplicate loan → `checkout` returns empty.
- `availableCopies(isbn)` = copies neither on loan nor held for a reservation; unknown ISBN → 0.
- `dueAt(loan)` = checkout time + 14 days (ms); empty for unknown loans. `returnLoan` returns the fine (0 allowed) or empty if the loan is unknown or already returned.
- Fine = `50 × floor((returnTime − dueTime) / 1 day)` when late, else 0.
- `reserve` returns `true` only when it queues the member (no copy available, and the member has no loan/reservation for that ISBN). A held copy can be checked out only by the reserver at the head of the queue (or the next ones, if several copies are held).
- `search(query)` returns matching ISBNs, sorted ascending; empty query matches every book.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
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
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_clock: Arc<dyn Clock>) -> Option<Box<dyn LibraryApi>> {
    None
}
```

## Running

```bash
cargo test -p p03_library
STRICT=1 cargo test -p p03_library   # fail instead of skip while unimplemented
cargo test -p p03_library --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p03_library --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
