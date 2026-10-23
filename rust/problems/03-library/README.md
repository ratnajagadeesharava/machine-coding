# 03 · Library Management (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

**What you are building.** The back-office system of a public library. Librarians register books (by ISBN, with a number of identical copies) and members. Members borrow books for 14 days, return them (paying a fine if late) and, when every copy of a popular title is out, join a waiting list. When a copy comes back it is set aside for the first person waiting instead of going back on the shelf. Visitors also search the catalogue by part of a title. All times come from an injected `Clock`, so late returns can be tested by moving the clock.

## Requirements

1. The library holds **books** identified by ISBN, each with a title and a number of identical **copies**. Adding a book whose ISBN exists adds copies.
2. **Members** are registered by id.
3. A member can **check out** a book if a copy is available to them (a copy is available to a member if it is free and not held for someone else). A successful checkout creates a **loan** with a due date **14 days** after checkout.
4. A member may hold at most **3** active loans, and at most one active loan per ISBN.
5. A loan is closed by **returning** it (`return_loan(loan_id)`), which returns the fine. The fine is **50 per full overdue day** (0 if returned on or before the due time, or less than 24 h late). A returned loan no longer counts toward the member's limit of 3.
6. When no copy is available, a member can **reserve** the book. Returned copies are **held** for waiting members in first-come-first-served order; a held copy cannot be borrowed by anyone else.
7. A member cannot reserve a book that is available, already reserved by them, or that they currently have on loan.
8. Books can be searched by (case-insensitive) title substring.
9. Time comes from the `Clock` given to the factory.

## Constraints

- Up to 100,000 books and 10,000 members; `search` must not be a noticeable linear scan on every keystroke for 10k titles (a scan is acceptable at this size, but be ready to discuss).
- Failed operations must not change any state.

## Examples

Times: day 0 = checkout; 1 day = 24 h. Members alice, bob, carol are registered.

**1. Reservation hand-over and fine** (one book, 1 copy)

```
add_book("111", "Clean Code", 1)
checkout(alice, "111")  -> loan L1          available_copies("111") = 0
due_at(L1)               -> checkout time + 14 days
reserve(bob, "111")     -> true             (no copy free, bob has none)
reserve(carol, "111")   -> true             (queued behind bob)
[16 days pass]
return_loan(L1)          -> fine 100         (2 full overdue days x 50)
available_copies("111")  = 0                 (the copy is held for bob)
checkout(carol, "111")  -> refused          (carol is second in line)
checkout(bob, "111")    -> loan L2
return_loan(L2) ... then checkout(carol, "111") -> loan L3
```

**2. Fine boundaries**

| Returned at | Fine |
|---|---|
| day 14, exactly at the due time | 0 |
| day 14 + 23 h | 0 (less than one full day late) |
| day 15 | 50 |
| day 16 | 100 |

**3. Limits** (books B1..B4, 1 copy each)

```
checkout(alice, B1), checkout(alice, B2), checkout(alice, B3) -> loans
checkout(alice, B4)  -> refused (already 3 active loans); available_copies(B4) = 1
return_loan(loan of B1); checkout(alice, B4) -> loan   (the slot is free again)

add_book("111", "Clean Code", 3); checkout(alice, "111") -> loan
checkout(alice, "111") -> refused (one loan per ISBN); available_copies("111") = 2
```

**4. Reserve refusals** (book "111", 1 copy)

```
reserve(bob, "111")   -> false  (a copy is available; just borrow it)
checkout(alice, "111"); reserve(alice, "111") -> false  (alice has it on loan)
reserve(bob, "111")   -> true;  reserve(bob, "111") -> false  (already queued)
```

**5. Search** (books 300 "The Pragmatic Programmer", 100 "Clean Code", 200 "Clean Architecture")

```
search("CLEAN")      -> ["100", "200"]
search("pragmatic")  -> ["300"]
search("zzz")        -> []
search("")           -> ["100", "200", "300"]
```

## Edge cases

- A refused `checkout` or `reserve` and a rejected `return_loan` change nothing (copy counts, limits and queues stay as they were).
- `return_loan` on an unknown id, or a second time on the same loan, returns `None` and must not add a phantom copy.
- A member with unknown id, or an unknown ISBN, is refused on `checkout`; `available_copies` of an unknown ISBN is 0.
- A copy held for a reserver cannot be borrowed by anyone else, even if the holder is slow to come.

## Glossary

- **ISBN**: the string that identifies a title; several identical copies can share one ISBN.
- **Loan**: the record created by a successful checkout; it has an id, a member, an ISBN and a due time.
- **Overdue**: returned strictly after the due time; only complete 24-hour periods are fined.
- **Held copy**: a returned copy set aside for the head of the reservation queue; it is neither on the shelf nor on loan.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Unknown member or ISBN, no available copy, limit reached, or a duplicate loan → `checkout` returns `None`.
- `available_copies(isbn)` = copies neither on loan nor held for a reservation; unknown ISBN → 0.
- `due_at(loan)` = checkout time + 14 days (ms); `None` for unknown loans. `return_loan` returns the fine (0 allowed) or `None` if the loan is unknown or already returned.
- Fine = `50 × floor((return_time − due_time) / 1 day)` when late, else 0.
- `reserve` returns `true` only when it queues the member (no copy available, and the member has no loan/reservation for that ISBN). A held copy can be checked out only by the reserver at the head of the queue (or the next ones, if several copies are held).
- `search(query)` returns matching ISBNs, sorted ascending; empty query matches every book.
- Returning the loan of a book with a waiting reservation puts that copy on hold for the head of the queue; returning a book with no queue makes the copy available again.
- A reserver leaves the queue when they check out their held copy, so the next reserver is then first in line.
- `search` matches against titles only, not ISBNs, and returns each matching ISBN once.

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
