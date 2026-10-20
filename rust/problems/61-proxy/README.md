# 61 · Proxy (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — the `DocumentStore` trait, `AccessPolicy` and `StoreError` are provided; the rest is yours to write. |

## Problem

Documents live in a slow remote store that is expensive to connect to and that must enforce permissions. Callers
use the plain `DocumentStore` trait (`fetch(id) -> Result<Option<String>, StoreError>`: `Ok(Some(text))`, or `Ok(None)` if
there is no such document); you write three stand-ins for the real store that add caching, lazy start-up and access
control without the callers noticing, and that can be combined.

## Terms

- **Real store**: the actual `DocumentStore` doing the work (in tests, a fake that records every request it receives).
- **Blank**: empty, or only whitespace.
- **Present / missing document**: `fetch` returns `Ok(Some(..))` / `Ok(None)`.

## Requirements

1. `CachingStoreProxy::new(real)`: `fetch(id)` asks the real store the first time, and serves repeats of a **present** document from memory (the real store is not asked again). Missing documents are **not** cached: each `fetch` of a missing id asks the real store again, so a document that appears later is found. `invalidate(id)` forgets one cached document (no effect if it is not cached); the next `fetch` asks the real store again. `cached_count()` is how many documents are currently cached. Blank id → `Err(StoreError::InvalidId)` (real store not asked).
2. `LazyStoreProxy::new(factory)` (`Box<dyn Fn() -> Box<dyn DocumentStore>>`): the factory is **not** called by `new`; it is called exactly once, on the first `fetch`, and the store it returns is reused for all later fetches (including ones for missing documents).
3. `AccessControlStoreProxy::new(real, policy, user)`: before every fetch it asks `policy.can_read(user, id)`. If `false` → `Err(StoreError::AccessDenied)` and the real store is **never** asked. Blank id → `Err(InvalidId)` and the policy is not asked either. If allowed, the result is whatever the real store returns.
4. Each proxy implements `DocumentStore`, so they can be stacked: `Caching(AccessControl(Lazy(real)))`. In that stack a denied document is never cached and never causes the real store to be built.

## Examples

Real store holds `a → "alpha"`, `b → "beta"`.

| Calls on `CachingStoreProxy` | Real store is asked | Result |
|---|---|---|
| `fetch("a")`, `fetch("a")`, `fetch("b")`, `fetch("a")` | `a`, `b` (2 requests) | `alpha`, `alpha`, `beta`, `alpha`; `cached_count()` = 2 |
| `fetch("late")` twice (missing), then `late` is added, `fetch("late")` | `late` three times | `Ok(None)`, `Ok(None)`, `Ok(Some("now here"))`; `cached_count()` = 1 |
| `fetch("a")`, store changes `a` to `v2`, `fetch("a")`, `invalidate("a")`, `fetch("a")` | 2 requests | `v1`, `v1` (stale), then `v2`; `invalidate("never-cached")` does nothing |

Policy allows only ids starting with `public`, user `ann`: `fetch("public")` → `Ok(Some("hello"))`, `fetch("secret")` → `Err(AccessDenied)`; the policy was asked `ann:public` then `ann:secret`, the real store only `public`.

Stack `Caching(AccessControl(Lazy(real)))` with a policy allowing only `public`: `fetch("secret")` → `Err(AccessDenied)` and the factory has still not been called; `fetch("public")` twice → `Ok(Some("hello"))` both times, the factory was called once, the real store was asked once, `cached_count()` = 1.

## Running

```bash
cargo test -p p61_proxy     # from rust/ — tests FAIL until you implement the proxies
cargo play run 61
```
