# 55 · Builder (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — `HttpRequest` and `HttpRequestBuilder` are stubs in `src/lib.rs` (`todo!()`); `BuildError` is provided. |

## Problem

**What you are building.** An HTTP client library needs request objects with many optional parts (method, URL, headers,
query parameters, body, timeout). Callers assemble a request step by step with a fluent builder, and the finished
`HttpRequest` is an immutable value that is always consistent. Mistakes (bad method, bad URL, bad timeout…) are all
reported by `build()`.

**Glossary**
- *Fluent* — every setter takes the builder by value and returns it so calls chain: `HttpRequest::builder().url(..).header(..).build()`.
- *Query parameter* — a `name=value` pair after `?` in the URL, separated by `&`.
- *Percent-encoding* — writing a byte as `%` plus two hex digits (a space becomes `%20`, `é` becomes `%C3%A9`).
- *Header name* — case-insensitive; stored and reported in lower case.

## Requirements

`HttpRequest` accessors: `method() -> &str`, `url() -> &str` (including the query string), `headers() -> &[(String, String)]` (insertion order), `header(name) -> Option<&str>` (lookup ignores case), `body() -> Option<&str>`, `timeout_millis() -> u32`. `BuildError` variants: `MissingUrl`, `InvalidUrl`, `InvalidMethod`, `InvalidName`, `InvalidTimeout`, `BodyNotAllowed`.

1. `HttpRequest::builder()` returns an `HttpRequestBuilder`. Defaults: method `GET`, timeout `5000` ms, no headers, no body.
2. Setters take and return `self` and never fail. The **first** problem found (in call order) is remembered and returned by `build()`; later problems do not replace it:
   - `method`: GET, POST, PUT, PATCH, DELETE in any case (stored upper-case), otherwise `BuildError::InvalidMethod`.
   - `url`: `http://` or `https://` plus at least one more character (`ftp://x` and `https://` fail), otherwise `InvalidUrl`.
   - `header(name, value)`: blank name → `InvalidName`. Names stored lower-case; setting a name again (any case) **replaces the value in place** (keeps its original position in `headers()`).
   - `query(name, value)`: blank name → `InvalidName`; name and value are percent-encoded (every UTF-8 byte except `A-Za-z0-9-_.~` becomes `%XX`, upper-case hex). Parameters are appended to the URL in call order, the first with `?`, or with `&` if the URL itself already contains `?`. Query parameters belong to the builder and are applied to whatever URL is set at `build()` time.
   - `timeout_millis`: 1 to 60000 inclusive, otherwise `InvalidTimeout`.
3. `build(self) -> Result<HttpRequest, BuildError>` reports the first remembered problem; otherwise it fails with `MissingUrl` (no URL set) or `BodyNotAllowed` (a body together with `GET` — the default — or `DELETE`).
4. **Reuse:** the builder is `Clone`; building from a clone must not be affected by later changes to the original, and requests already built never change.
5. `HttpRequest` is immutable: accessors only (the stub already derives `Debug, Clone, PartialEq` — keep them).

## Examples

| Builder calls | Result |
|---|---|
| `builder().url("https://api.test/items").build()` | `Ok`: method `GET`, url `https://api.test/items`, timeout `5000`, no headers, body `None` |
| `.method("post").url("http://api.test/items").header("Content-Type","application/json").header("X-Trace","abc").body("{\"a\":1}").timeout_millis(250)` | method `POST`; headers `[("content-type","application/json"), ("x-trace","abc")]`; `header("CONTENT-TYPE")` = `Some("application/json")`; timeout `250` |
| `.url("https://api.test/items").query("page","2").query("q","a b&c").query("name","café")` | url `https://api.test/items?page=2&q=a%20b%26c&name=caf%C3%A9` |
| `.url("https://api.test/items?sort=asc").query("page","2")` | url `https://api.test/items?sort=asc&page=2` |
| `.header("A","1").header("B","2").header("a","3")` | headers `[("a","3"), ("b","2")]` |
| `.method("FETCH").url("ftp://x").timeout_millis(0).build()` | `Err(InvalidMethod)` (the first problem wins) |
| `builder().build()` / `.url("https://x.test").body("data").build()` | `Err(MissingUrl)` / `Err(BodyNotAllowed)`; with `.method("PUT")` the body is fine |
| `base = builder().url("https://api.test/a").header("x-one","1"); first = base.clone().build(); second = base.header("x-two","2").query("page","9").url("https://api.test/b").build()` | `first`: url `https://api.test/a`, 1 header; `second`: url `https://api.test/b?page=9`, 2 headers |

## Edge cases

- Boundaries: timeout `0` and `60001` → `InvalidTimeout`; `1` and `60000` accepted.
- Setting a valid value after an invalid one does not clear the remembered problem.

## Running

```bash
cargo test -p p55_builder     # from rust/ — tests FAIL until you implement the stubs
cargo play run 55
```
