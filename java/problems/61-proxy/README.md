# 61 · Proxy (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — the `DocumentStore` subject interface and `AccessPolicy` are provided; the rest is yours to write. |

## Problem

Documents live in a slow remote store that is expensive to connect to and that must enforce permissions. Callers
use the plain `DocumentStore` interface (`Optional<String> fetch(String id)`: the document text, or empty if there is no
such document); you write three stand-ins for the real store that add caching, lazy start-up and access control without
the callers noticing, and that can be combined.

## Terms

- **Real store**: the actual `DocumentStore` doing the work (in tests, a fake that records every request it receives).
- **Blank**: `null`, empty, or only whitespace.
- **Present / missing document**: `fetch` returns a value / returns empty.

## Requirements

1. `CachingStoreProxy(real)`: `fetch(id)` asks the real store the first time, and serves repeats of a **present** document from memory (the real store is not asked again). Missing documents are **not** cached: each `fetch` of a missing id asks the real store again, so a document that appears later is found. `invalidate(id)` forgets one cached document (no effect if it is not cached); the next `fetch` asks the real store again. `cachedCount()` is how many documents are currently cached. Blank/`null` id → `IllegalArgumentException` (real store not asked).
2. `LazyStoreProxy(Supplier<DocumentStore>)`: the supplier is **not** called when the proxy is constructed; it is called exactly once, on the first `fetch`, and the result is reused for all later fetches (including ones for missing documents).
3. `AccessControlStoreProxy(real, policy, user)`: before every fetch it asks `policy.canRead(user, id)`. If `false` → `SecurityException` and the real store is **never** asked. Blank/`null` id → `IllegalArgumentException` and the policy is not asked either. If allowed, the result is whatever the real store returns.
4. Each proxy implements `DocumentStore`, so they can be stacked: `Caching(AccessControl(Lazy(real)))`. In that stack a denied document is never cached and never causes the real store to be built.

## Examples

Real store holds `a → "alpha"`, `b → "beta"`.

| Calls on `CachingStoreProxy` | Real store is asked | Result |
|---|---|---|
| `fetch("a")`, `fetch("a")`, `fetch("b")`, `fetch("a")` | `a`, `b` (2 requests) | `alpha`, `alpha`, `beta`, `alpha`; `cachedCount()` = 2 |
| `fetch("late")` twice (missing), then `late` is added, `fetch("late")` | `late` three times | empty, empty, `"now here"`; `cachedCount()` = 1 |
| `fetch("a")`, store changes `a` to `v2`, `fetch("a")`, `invalidate("a")`, `fetch("a")` | 2 requests | `v1`, `v1` (stale), then `v2`; `invalidate("never-cached")` does nothing |

Policy allows only ids starting with `public`, user `ann`: `fetch("public")` → `"hello"`, `fetch("secret")` → `SecurityException`; the policy was asked `ann:public` then `ann:secret`, the real store only `public`.

Stack `Caching(AccessControl(Lazy(real)))` with a policy allowing only `public`: `fetch("secret")` → `SecurityException` and the supplier has still not been called; `fetch("public")` twice → `"hello"` both times, the supplier was called once, the real store was asked once, `cachedCount()` = 1.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the proxies
```
