# 55 · Builder (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — `HttpRequest` and `HttpRequestBuilder` are stubs in `src/lib.rs` (`todo!()`); `BuildError` is provided. |

> **Pattern:** *Builder* — separate the construction of a complex object from its representation, so the same
> step-by-step process can create different results, and the finished object is **immutable** and **always valid**.
> The idiomatic Rust builder is a consuming `self -> Self` chain ending in `build() -> Result<_, _>`.

## Problem

An HTTP client library needs request objects with many optional parts (method, URL, headers, query parameters, body,
timeout). A function with eight parameters is unreadable, and public fields allow half-built or later-modified
requests. Provide a fluent builder.

## Requirements

1. `HttpRequest::builder()` returns a builder. Defaults: method `GET`, timeout `5000` ms, no headers, no body.
2. Setters take and return `self` and never fail. The **first** problem is remembered and returned by `build()`:
   - `method`: GET, POST, PUT, PATCH, DELETE in any case (stored upper-case), otherwise `BuildError::InvalidMethod`.
   - `url`: `http://` or `https://` plus at least one more character, otherwise `InvalidUrl`.
   - `header(name, value)`: blank name → `InvalidName`. Names stored lower-case; setting a name again (any case) **replaces the value in place**.
   - `query(name, value)`: blank name → `InvalidName`; name and value are percent-encoded (every UTF-8 byte except `A-Za-z0-9-_.~` becomes `%XX`, upper-case hex). Appended in call order with `?`, or `&` if the URL already contains `?`.
   - `timeout_millis`: 1 to 60000, otherwise `InvalidTimeout`.
3. `build()` also fails with `MissingUrl` (no URL) or `BodyNotAllowed` (a body with `GET`/`DELETE`).
4. **Reuse:** the builder is `Clone`; building from a clone must not be affected by later changes to the original.
5. `HttpRequest` is immutable: private fields, accessors only (the stub already derives `Debug, Clone, PartialEq` — keep them).

## Running

```bash
cargo test -p p55_builder     # from rust/ — tests FAIL until you implement the stubs
cargo play run 55
```
