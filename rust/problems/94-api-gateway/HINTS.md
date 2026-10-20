# Hints — 94 API Gateway (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Which pieces of state belong to a client, which to a route and which to a backend - and which of them change on every request?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

A token bucket needs no timer thread: store the token count and the time of the last update, and top up lazily when the next request arrives. A circuit breaker is a small state machine (closed, open until a time, half-open with one trial in flight) per backend. Keep the three kinds of state separate so each can have its own small lock.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Pipeline the request through the checks in the stated order. Take the client's lock only to charge a token. Pick the start index from the route's counter, then walk the route's backends in order, asking each backend's breaker whether it admits a call. Call the backend with no lock held, then report success or failure back to its breaker. Decide carefully what the half-open state means when two requests arrive at once.

**Rust:** Hold per-client and per-backend state in `Mutex`es inside `Arc`s (or atomics for counters) and drop every guard before calling `backend.handle`. Avoid floating-point drift in the bucket by keeping integer milli-tokens (rpm tokens per 60 000 ms).

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
