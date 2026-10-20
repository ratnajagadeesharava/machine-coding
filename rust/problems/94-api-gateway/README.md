# 94 · API Gateway (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the edge gateway of a small platform. Clients present an API key and a path; the gateway decides who the
client is, where the request goes, whether the client is allowed to make it right now, and which of several replicas of the
target service should serve it - and it must keep working when replicas misbehave. Every answer is an HTTP-like status and a
body. The gateway is called from many threads at once, and backends can be slow, so one slow call must not hold up everybody else.

## Requirements

1. `registerBackend(id, backend)` makes a backend callable by id. A backend takes a path and returns a body, or fails by throwing.
2. `addRoute(prefix, backendIds)` sends every path starting with `prefix` to that ordered list of backends. When several prefixes match, the **longest** wins. Unknown backend id or an empty list is rejected (`IllegalArgumentException` / `Err(InvalidRoute)`).
3. `addClient(apiKey, name, requestsPerMinute)` registers a client with a token bucket of capacity `requestsPerMinute`, full at registration, refilled continuously at `requestsPerMinute / 60` tokens per second (fractions accumulate, never above capacity).
4. `handle(apiKey, path)` checks in order: unknown key -> `401 "unauthorized"`; no matching route -> `404 "no route"` (costs no token); no token -> `429 "rate limited"`; otherwise one token is spent and the request is dispatched.
5. Dispatch: candidates are the route's backends whose circuit is not open, starting at the next one in round-robin order (one counter per route) and failing over to the following ones when a backend throws. First success -> `200` with the backend's body.
6. If every candidate threw -> `502 "bad gateway"`. If every backend of the route has an open circuit -> `503 "service unavailable"` and nothing is called.
7. Circuit breaker per backend: 3 consecutive failures open it for 30 s; afterwards exactly one trial request is let through (half-open): success closes it, failure re-opens it for another 30 s. Any success resets the failure count.
8. `successCount(client)` counts requests answered 200 for that client; `callCount(backend)` counts calls made to the backend, successful or not.

## Constraints

- `handle` is called from many threads at once; token accounting must be exact: a bucket of N tokens admits exactly N requests, never N+1.
- Backend calls may be slow. Do not hold a gateway-wide lock (or a lock shared by unrelated clients/backends) while calling a backend.
- Time comes only from the injected clock; the tests move it with `FakeClock.advance`. Never read the system clock and never sleep.
- Round-robin must stay fair under concurrency: 1,600 concurrent requests over 4 backends reach each backend exactly 400 times.

## Examples

```
backends a, b, c on route /api, client key-1 (600 rpm)
handle(key-1, /api/p) x6 -> 200 a/api/p, b/api/p, c/api/p, a/api/p, b/api/p, c/api/p

a throws, route /api -> [a, b]:   handle -> 200 "b/api/1"   (a called once, b called once)
after 3 consecutive failures of a: a is skipped for 30 000 ms (callCount stays 3)
at +30 000 ms one request reaches a again; if it fails, a is open for another 30 s
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Gateway`, `Backend` (a lambda `path -> body`, failure = any exception), `Response(int status, String body)`, factory `Solution.create(Clock)`. Rust: `Gateway` trait with `register_backend(id, Arc<dyn Backend>)`, `Backend::handle(&str) -> Result<String, String>` (failure = `Err`), `Response { status: u16, body }`, `create(Arc<dyn Clock>)`.
- Exact bodies: 401 `unauthorized`, 404 `no route`, 429 `rate limited`, 502 `bad gateway`, 503 `service unavailable`. A 200 carries the backend's body unchanged (the tests' backends return `name + path`, the full request path).
- A `null` API key is simply unknown: 401 (Java). Unknown-key and no-route requests cost no token.
- Rust: `add_route` returns `Err(GatewayError::InvalidRoute)`; `success_count` / `call_count` return `Result<usize, GatewayError>` (`UnknownName` for an unregistered name; the tests only query registered ones); Java returns plain `int`.
- Prefix match is plain `startsWith` on the path: `/api` matches `/api/x`; the longest registered prefix wins.
- Token bucket: capacity = `requestsPerMinute`; one token = 60 000 / rpm ms. After exactly 1 000 ms at 60 rpm one request passes, half a token (500 ms) is not enough, and a long idle period never gives more than `capacity` tokens.
- Round-robin: one shared counter per route, so with all circuits closed consecutive requests go to a, b, c, a, b, c... Whether a failover attempt also advances the counter is yours to decide; the tests only fix the order for healthy backends and an even spread.
- Circuit: opens on the 3rd consecutive failure (a failure is a thrown exception); stays open while `now < openedAt + 30 000`; at `+30 000` a single trial is allowed. A trial failure re-opens for a full 30 s from that moment. A request that failed over after a backend's failure counts as that backend's failure.
- With a single-backend route and an open circuit the answer is 503 (not 502) and `callCount` does not change; the token is still spent.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Time comes from the clock handed to `create`.

use std::sync::Arc;

/// A service behind the gateway (provided by the tests). Thread-safe; fails by returning `Err`.
/// Any `Fn(&str) -> Result<String, String>` closure is a backend.
pub trait Backend: Send + Sync {
    fn handle(&self, path: &str) -> Result<String, String>;
}

impl<F> Backend for F
where
    F: Fn(&str) -> Result<String, String> + Send + Sync,
{
    fn handle(&self, path: &str) -> Result<String, String> {
        self(path)
    }
}

/// What the gateway answers. Provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

impl Response {
    pub fn new(status: u16, body: &str) -> Response {
        Response {
            status,
            body: body.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GatewayError {
    /// unknown backend id, empty backend list
    InvalidRoute,
    UnknownName,
}

pub trait Gateway: Send + Sync {
    fn register_backend(&self, id: &str, backend: Arc<dyn Backend>);

    /// Routes every path that starts with `path_prefix` to the backends (in the given order). When several prefixes
    /// match, the longest one wins.
    fn add_route(&self, path_prefix: &str, backend_ids: &[&str]) -> Result<(), GatewayError>;

    /// Registers an API key. The client may make `requests_per_minute` requests per minute: a token bucket with that
    /// capacity, full at registration, refilled continuously at requests_per_minute / 60 tokens per second
    /// (fractions accumulate). Each request that passes authentication and routing costs one token.
    fn add_client(&self, api_key: &str, client_name: &str, requests_per_minute: u32);

    /// Handles a request, checking in this order:
    /// 1. unknown API key: 401 "unauthorized";
    /// 2. no route: 404 "no route" (costs no token);
    /// 3. no token left: 429 "rate limited";
    /// 4. pick backends: the route's backends whose circuit is not open, starting with the next one in round-robin
    ///    order (one shared counter per route) and failing over to the following ones when a backend fails. First
    ///    success: 200 with the backend's body. If every candidate failed: 502 "bad gateway". If every backend's
    ///    circuit is open: 503 "service unavailable" (nothing is called).
    ///
    /// Circuit breaker per backend: 3 consecutive failures open it for 30 seconds; after that one trial request is
    /// let through (half-open): success closes it, failure opens it for another 30 seconds. Any success resets the
    /// consecutive-failure count.
    fn handle(&self, api_key: &str, path: &str) -> Response;

    /// Number of requests answered 200 for the client.
    fn success_count(&self, client_name: &str) -> Result<usize, GatewayError>;

    /// Number of times the backend was called (successful or not).
    fn call_count(&self, backend_id: &str) -> Result<usize, GatewayError>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_clock: Arc<dyn Clock>) -> Option<Arc<dyn Gateway>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- 401 for unknown or null key; 404 without consuming the only token of a 1 rpm client.
- Longest-prefix routing (`/api` vs `/api/users`); bad routes rejected.
- Round-robin order over three backends; failover to the next backend when one throws; 502 when all throw.
- Circuit opens after 3 failures (backend called exactly 3 times over 20 requests), stays open until 29 999 ms, gets a trial at 30 000 ms and closes on success; a failing trial re-opens it; 503 with nothing called when every circuit is open; two failures then success never opens it.
- Token bucket: 60 requests pass, the 61st gets 429, a second client has its own bucket, refill after 1 s / 500 ms steps, capacity is never exceeded after an hour idle.
- Concurrency with 16 threads: exactly 1,000 of 1,600 requests admitted at 1,000 rpm; four backends receive exactly 400 calls each.
- **Repeat 100x**: 8 threads against a 50-token bucket admit exactly 50 requests and exactly 50 backend calls.
- More edge cases in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

Hold per-client and per-backend state in `Mutex`es inside `Arc`s (or atomics for counters) and drop every guard before calling `backend.handle`. Avoid floating-point drift in the bucket by keeping integer milli-tokens (rpm tokens per 60 000 ms).

## Running

```bash
cargo test -p p94_api_gateway
STRICT=1 cargo test -p p94_api_gateway   # fail instead of skip while unimplemented
cargo test -p p94_api_gateway --features hidden   # interviewer scenarios
cargo play run 94              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.
