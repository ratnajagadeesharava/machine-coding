# 55 · Builder (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — one stub class, `HttpRequest`, with its nested `Builder`. |

## Problem

An HTTP client library needs request objects with many optional parts (method, URL, headers, query parameters, body,
timeout). Provide a fluent builder for them.

## Requirements

1. `HttpRequest.builder()` returns a `Builder`. Defaults: method `GET`, timeout `5000` ms, no headers, no body.
2. Setters (all return the builder) validate **immediately** with `IllegalArgumentException`:
   - `method`: GET, POST, PUT, PATCH, DELETE in any case (stored upper-case).
   - `url`: must start with `http://` or `https://` plus at least one more character.
   - `header(name, value)`: blank name or `null` value rejected. Names are stored lower-case. Setting a name again (any case) **replaces the value in place** (keeps its original position).
   - `query(name, value)`: blank name rejected; name and value are percent-encoded (every UTF-8 byte except `A-Za-z0-9-_.~` becomes `%XX`, upper-case hex). Appended in call order with `?`, or `&` if the URL already contains `?`.
   - `timeoutMillis`: 1 to 60000.
3. `build()` throws `IllegalStateException` when no URL was set, or when a body is combined with `GET` or `DELETE`.
4. **Reuse:** one builder can build several requests; changing the builder afterwards must not change requests already built.
5. `HttpRequest` is immutable: it is created only through the builder and `headers()` cannot be modified.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement HttpRequest
```
