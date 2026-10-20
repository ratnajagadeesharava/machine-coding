# 55 · Builder (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — one stub class, `HttpRequest`, with its nested `Builder`. |

## Problem

**What you are building.** An HTTP client library needs request objects with many optional parts (method, URL, headers,
query parameters, body, timeout). Callers assemble a request step by step with a fluent builder, and the finished
`HttpRequest` is an immutable value that is always consistent. Mistakes (bad method, bad URL, bad timeout…) are reported
as early as possible.

**Glossary**
- *Fluent* — every setter returns the same builder so calls chain: `builder().url(..).header(..).build()`.
- *Query parameter* — a `name=value` pair after `?` in the URL, separated by `&`.
- *Percent-encoding* — writing a byte as `%` plus two hex digits (a space becomes `%20`, `é` becomes `%C3%A9`).
- *Header name* — case-insensitive; stored and reported in lower case.

## Requirements

`HttpRequest` accessors: `method()`, `url()` (including the query string), `headers()` (a `Map<String,String>` that keeps insertion order), `header(name)` → `Optional<String>` (lookup ignores case), `body()` → `Optional<String>`, `timeoutMillis()`.

1. `HttpRequest.builder()` returns a `Builder`. Defaults: method `GET`, timeout `5000` ms, no headers, no body.
2. Setters (all return the builder) validate **immediately** with `IllegalArgumentException`, without changing the builder:
   - `method`: GET, POST, PUT, PATCH, DELETE in any case (stored upper-case); `null` or any other value (e.g. `FETCH`) rejected.
   - `url`: must start with `http://` or `https://` plus at least one more character (`ftp://x` and `https://` are rejected).
   - `header(name, value)`: blank name or `null` value rejected. Names are stored lower-case. Setting a name again (in any case) **replaces the value in place** (keeps its original position in `headers()`).
   - `query(name, value)`: blank name rejected; name and value are percent-encoded (every UTF-8 byte except `A-Za-z0-9-_.~` becomes `%XX`, upper-case hex). Parameters are appended to the URL in call order, the first with `?`, or with `&` if the URL itself already contains `?`. Query parameters belong to the builder and are applied to whatever URL is set at `build()` time.
   - `timeoutMillis`: 1 to 60000 inclusive.
3. `build()` throws `IllegalStateException` when no URL was set, or when a body is set together with method `GET` (the default) or `DELETE`.
4. **Reuse:** one builder can build several requests; changing the builder afterwards must not change requests already built.
5. `HttpRequest` is immutable: it is created only through the builder (no public constructor, no setters) and `headers()` cannot be modified (`put` throws `UnsupportedOperationException`).

## Examples

| Builder calls | Result |
|---|---|
| `builder().url("https://api.test/items").build()` | method `GET`, url `https://api.test/items`, timeout `5000`, no headers, body empty |
| `.method("post").url("http://api.test/items").header("Content-Type","application/json").header("X-Trace","abc").body("{\"a\":1}").timeoutMillis(250)` | method `POST`; headers in order `content-type=application/json`, `x-trace=abc`; `header("CONTENT-TYPE")` = `application/json`; timeout `250` |
| `.url("https://api.test/items").query("page","2").query("q","a b&c").query("name","café")` | url `https://api.test/items?page=2&q=a%20b%26c&name=caf%C3%A9` |
| `.url("https://api.test/items?sort=asc").query("page","2")` | url `https://api.test/items?sort=asc&page=2` |
| `.header("A","1").header("B","2").header("a","3")` | headers `a=3`, `b=2` (in that order) |
| `.url("https://x.test").body("data").build()` | `IllegalStateException` (body on GET); with `.method("PUT")` it is fine |
| `b = builder().url("https://api.test/a").header("x-one","1"); first = b.build(); b.header("x-two","2").query("page","9").url("https://api.test/b"); second = b.build()` | `first`: url `https://api.test/a`, one header; `second`: url `https://api.test/b?page=9`, two headers |

## Edge cases

- Boundaries: timeout `0` and `60001` rejected, `1` and `60000` accepted.
- `builder().build()` with nothing set → `IllegalStateException` (no URL).
- A rejected setter call throws at once; it does not wait for `build()`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement HttpRequest
```
