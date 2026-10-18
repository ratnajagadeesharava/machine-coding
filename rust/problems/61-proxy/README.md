# 61 · Proxy (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — the `DocumentStore` trait, `AccessPolicy` and `StoreError` are provided; three proxies are stubs. |

> **Pattern:** *Proxy* — a stand-in that has the **same interface** as the real object and controls access to it.
> Three classic kinds in one problem: a **caching** proxy (avoid repeated expensive calls), a **virtual** (lazy)
> proxy (delay creating something expensive) and a **protection** proxy (permission checks).

## Problem

Documents live in a slow remote store that is expensive to connect to and that must enforce permissions. Callers
use the plain `DocumentStore` trait; proxies add caching, lazy start-up and access control around the real thing
without the callers noticing.

## Requirements

1. `CachingStoreProxy::new(real)`: `fetch(id)` asks the real store once per **present** document and serves repeats from memory. Missing documents are **not** cached. `invalidate(id)` forgets one cached document; `cached_count()` is how many are cached. Blank id → `Err(StoreError::InvalidId)` (real store not asked). `fetch` takes `&self`, so you need interior mutability (`RefCell<HashMap<..>>`).
2. `LazyStoreProxy::new(factory)`: the factory is **not** called by `new`; it is called exactly once, on the first `fetch`, and the store it returns is reused (`OnceCell` is a natural fit).
3. `AccessControlStoreProxy::new(real, policy, user)`: before every fetch it asks `policy.can_read(user, id)`. If `false` → `Err(StoreError::AccessDenied)` and the real store is **never** asked. Blank id → `Err(InvalidId)` and the policy is not asked either.
4. Each proxy implements `DocumentStore`, so they can be stacked: `Caching(AccessControl(Lazy(real)))`. In that stack a denied document is never cached and never causes the real store to be built.

## Constraints (what the tests catch)

- The real store is a counting fake; the policy is a recording struct; the factory is a counting closure.

## Running

```bash
cargo test -p p61_proxy     # from rust/ — tests FAIL until you implement the proxies
cargo play run 61
```
