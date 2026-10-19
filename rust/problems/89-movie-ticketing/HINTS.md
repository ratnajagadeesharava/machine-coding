# Hints — 89 Movie Ticket Booking (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

A seat can be free, held, or sold. Which of the three states needs a timestamp, and who is responsible for noticing that the timestamp has passed?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

You do not need a timer thread: when any operation touches a show, first treat every hold whose expiry is `<= now` as released, then proceed. Seat inventory is per show, so the unit that has to be atomic is one show, not the whole system.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Keep per-show state: seat label -> status, plus a hold table (hold id -> user, seats, expiry, used flag) and a booking table. Protect one show with one lock (or one lock per show) and do 'sweep expired, validate all seats, then mark all seats held' in one critical section. `confirm` re-validates that the hold is still live and unused, then flips the seats to sold; `cancel` computes the refund from `showStart - now` using integer minutes.

**Rust:** A `Mutex<ShowState>` per show inside a `HashMap<String, Arc<Mutex<..>>>` (or one `Mutex` around everything to start); read time from the injected clock inside the guard, and use an `enum SeatState { Free, Held(HoldId), Sold(BookingId) }`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
