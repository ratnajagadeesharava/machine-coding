# 61 · Proxy (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — the `DocumentStore` subject interface and `AccessPolicy` are provided; the rest is yours to write. |

## Problem

Documents live in a slow remote store that is expensive to connect to and that must enforce permissions. Callers
use the plain `DocumentStore` interface; proxies add caching, lazy start-up and access control around the real thing
without the callers noticing.

## Requirements

1. `CachingStoreProxy(real)`: `fetch(id)` asks the real store once per **present** document and serves repeats from memory. Missing documents are **not** cached (looked up again each time). `invalidate(id)` forgets one cached document; `cachedCount()` is how many are cached. Blank/`null` id → `IllegalArgumentException` (real store not asked).
2. `LazyStoreProxy(Supplier<DocumentStore>)`: the supplier is **not** called when the proxy is constructed; it is called exactly once, on the first `fetch`, and the result is reused.
3. `AccessControlStoreProxy(real, policy, user)`: before every fetch it asks `policy.canRead(user, id)`. If `false` → `SecurityException` and the real store is **never** asked. Blank/`null` id → `IllegalArgumentException` and the policy is not asked either.
4. Each proxy implements `DocumentStore`, so they can be stacked: `Caching(AccessControl(Lazy(real)))`. In that stack a denied document is never cached and never causes the real store to be built.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the proxies
```
