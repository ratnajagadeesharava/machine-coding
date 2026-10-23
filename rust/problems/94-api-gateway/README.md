# 94 · API Gateway (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

You are building the edge gateway of a small platform: the one door every API call goes through. A client presents an
API key and a path such as `/api/users/42`. The gateway works out who the client is, picks the backend group the path belongs to,
enforces the client's rate limit, and forwards the call to one of the replicas of that backend, rotating between them. When
a replica keeps failing the gateway stops sending it traffic for a while (a circuit breaker) and retries it carefully
later. Every answer is an HTTP-like status code plus a text body. Many request threads call it at once and backends can be
slow, so one slow backend call must not block unrelated requests.

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
9. Order of checks in `handle`, and what each step costs: unknown key -> 401 (no token); no matching route -> 404 (no token); empty bucket -> 429 (no backend called); otherwise exactly one token is spent, even if the answer is then 502 or 503.
10. Round-robin position is per route, not per client or per backend; a route registered with `[a, b, c]` rotates a, b, c, a, b, c, ... across all clients.
11. A backend's circuit is per backend id, shared by every route that uses it, and its state does not depend on which client made the call.
12. `success_count` counts only 200 answers for that client; 401, 404, 429, 502 and 503 do not count. `call_count` counts every call made into the backend, including failed ones and failover attempts, but never counts a call that was skipped because the circuit was open.

## Constraints

- `handle` is called from many threads at once; token accounting must be exact: a bucket of N tokens admits exactly N requests, never N+1.
- Backend calls may be slow. Requests for unrelated clients and backends must not wait on each other while a backend call is in progress.
- Time comes only from the injected clock; the tests move it with `FakeClock.advance`. Never read the system clock and never sleep.
- Round-robin must stay fair under concurrency: 1,600 concurrent requests over 4 backends reach each backend exactly 400 times.

## Glossary

- **API key / client**: the secret string a caller sends; each key is registered with a client name and a rate limit.
- **Route / prefix**: a path prefix mapped to an ordered list of backend ids. `/api` matches `/api`, `/api/x` and `/apiary`; the longest matching prefix wins.
- **Backend**: a function from a path to a body that may throw. Replicas of one service are several backends listed in one route.
- **Token bucket**: a client's allowance: it holds up to `requests_per_minute` tokens, each request spends one, and tokens trickle back continuously (1 per `60 000 / requests_per_minute` ms).
- **Failover**: if the chosen backend fails, the request is tried on the next candidate of the same route instead of failing at once.
- **Circuit breaker**: per-backend protection. Closed = normal. Open = the backend is skipped for 30 s after 3 consecutive failures. Half-open = the 30 s are over and a single trial request is allowed: success closes the circuit, failure opens it again for 30 s.

## Examples

**Example 1: round-robin** (backends `a`, `b`, `c` echo `name + path`; route `/api` -> [a, b, c]; client `key-1` with 600 rpm)

```
handle(key-1, "/api/p") six times -> 200 "a/api/p", 200 "b/api/p", 200 "c/api/p", 200 "a/api/p", 200 "b/api/p", 200 "c/api/p"
success_count("acme") -> 6
```

**Example 2: status codes and cost in tokens** (client `tiny`, 1 request per minute; route `/api` -> [a])

```
handle("nope", "/api/x")   -> 401 "unauthorized"   (no token spent)
handle("k", "/other")      -> 404 "no route"       (no token spent)
handle("k", "/api/x")      -> 200 "a/api/x"        (the only token)
handle("k", "/api/x")      -> 429 "rate limited"
```

**Example 3: token bucket with a refill** (60 rpm = 1 token per second)

```
60 requests -> 200 each;  the 61st -> 429
+1000 ms: 1 request -> 200, the next -> 429
+500 ms:  429 (half a token)       +500 ms more: 200
+1 hour idle: still only 60 tokens available, never more than the capacity
```

**Example 4: failover and circuit breaker** (route `/api` -> [a, b], `a` always fails, `b` works)

```
handle -> 200 "b/api/1"           // a was tried first and threw, the request failed over to b
                                  // call_count(a) = 1, call_count(b) = 1
20 requests in a row all give 200; call_count(a) stops at 3 once a has failed 3 times in a row
the circuit of a is open: for the next 30 000 ms (up to and including +29 999) a gets no calls
at +30 000 ms exactly one trial call reaches a; if it fails, a is open for another 30 000 ms;
if it succeeds, a is closed again and takes its share of the rotation
```

**Example 5: 502 versus 503** (route `/api` -> [a] only, `a` always fails, plenty of tokens)

```
handle x3 -> 502 "bad gateway", 502, 502        // call_count(a) = 3, the third failure opens the circuit
handle    -> 503 "service unavailable"          // a is not called, call_count(a) stays 3, but a token is spent
```
Two failures followed by a success never open a circuit: the success resets the consecutive-failure count.

## Edge cases

- `add_route` with an unknown backend id or an empty list is rejected (`Err(GatewayError::InvalidRoute)`) and registers nothing.
- A path that matches no prefix is 404 even if the client is out of tokens (the route check comes before the rate limit).
- With two prefixes `/api` and `/api/users`, the path `/api/users/42` goes to the `/api/users` route and `/api/orders` to the `/api` route.
- A `null` key (only possible in Java) is just an unknown key: 401.
- 1,600 concurrent requests on a 1,000-rpm client: exactly 1,000 get through (200) and 600 get 429; 16 threads over four backends give each backend exactly 400 calls.
- When every candidate backend of the route fails, the answer is 502 and each of those failures counts towards its backend's circuit.

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

## Running

```bash
cargo test -p p94_api_gateway
STRICT=1 cargo test -p p94_api_gateway   # fail instead of skip while unimplemented
cargo test -p p94_api_gateway --features hidden   # interviewer scenarios
cargo play run 94              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.
