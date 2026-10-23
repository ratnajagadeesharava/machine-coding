# 82 · URL Shortener (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A marketing team wants short links such as `https://sho.rt/Ab3` instead of long campaign URLs. You are building the **core of that service**, without any web layer: `shorten` turns a long URL into a short code; `resolve` turns a code back into the URL and counts the visit; `clicks` reports the visit count; `delete` retires a code. Customers may also pick their own code (an *alias* like `summer-sale`). Many request threads use the service at once, so the same URL must always get one code, two people cannot win the same alias, and no click may be lost.

## Problem

Marketing wants short links for campaign emails and SMS. Build the core of a link-shortening service: give it a long
http(s) URL and get back a short code; later, someone opens the short link and the service looks the original URL up and counts
the visit. Customers may also ask for a vanity alias such as `summer-sale`, and links can be deleted when a campaign ends.
Many request handlers call the service at the same time, and a code that once pointed at a URL must never silently point at a
different one.

## Requirements

1. `shorten(url)` returns a short code for an `http://` or `https://` URL. Shortening the same URL again returns the same code; different URLs get different codes.
2. A null, blank, or non-http(s) URL (`example.com/a`, `ftp://...`) is rejected.
3. Generated codes consist only of `[0-9A-Za-z]` and are at most 6 characters long, even after a million URLs.
4. `shorten(url, alias)` registers a caller-chosen code. The alias must match `[A-Za-z0-9_-]{3,20}`. Registering the same alias for the same URL again returns it.
5. An alias is rejected if it is malformed, if it already means a different URL, or if it equals an existing generated code. A rejected call changes nothing.
6. `resolve(code)` returns the URL and counts exactly one click. An unknown or deleted code yields no URL and no click.
7. `clicks(code)` is the number of successful resolves; it is an error (Java) / `None` (Rust) for an unknown or deleted code.
8. `delete(code)` removes a code and reports whether it existed. A deleted code is never handed out again to another URL.

## Constraints

- The service is called from many threads at once (Java: `Must be thread-safe`; Rust: `&self`, `Send + Sync`).
- Every URL gets exactly one code even when 16 threads shorten it at the same instant; exactly one thread wins a contested alias.
- No click may be lost: 8 threads x 1,000 resolves of one code must read back 8,000.
- Shortening and resolving must stay O(1) on average; the tests insert up to 32,000 URLs concurrently (hidden: 1,000,000 in Java, 300,000 in Rust).
- No clock or randomness is required; if you use randomness, collisions with existing codes and aliases must be handled.

## Examples

```
shorten("https://example.com/a?b=c#d")  -> "1"            (any [0-9A-Za-z]{1,6} code)
shorten("https://example.com/a?b=c#d")  -> "1"            (same URL, same code)
shorten("https://example.com/other")    -> "2"
shorten("https://example.com/x", "summer-sale") -> "summer-sale"
resolve("summer-sale") x3               -> the URL; clicks("summer-sale") == 3
resolve("nope")                         -> empty; clicks("nope") -> error / None
delete("1") -> true; delete("1") -> false; a later shorten never returns "1"
```

### Worked traces

```
shorten("https://example.com/a?b=c#d")  -> Ok(X)  (X is 1-6 chars of 0-9 A-Z a-z)
resolve(X)                              -> Some("https://example.com/a?b=c#d")  (query and fragment intact)

shorten_with_alias(u1, "promo") -> Ok("promo")   again with the same URL -> Ok("promo")
resolve("promo") x2                     -> Some(u1);  clicks("promo") = Some(2)
shorten_with_alias(u1, "promo") again   -> Ok("promo"); clicks("promo") is still Some(2) (not reset)
shorten_with_alias(u2, "promo")         -> Err(AliasTaken); resolve("promo") is still Some(u1)

shorten_with_alias(u3, "ab")        -> Err(InvalidAlias)   (2 chars: too short)
shorten_with_alias(u3, "x" * 21)    -> Err(InvalidAlias)   (21 chars: too long)
shorten_with_alias(u3, "has space") -> Err(InvalidAlias)   (illegal character)
shorten_with_alias(u3, X)           -> Err(AliasTaken) or Err(InvalidAlias)   (X is an existing generated code)
shorten("") / shorten("   ") / shorten("example.com/a") / shorten("ftp://example.com/a") -> Err(InvalidUrl)

5 x resolve(X) + resolve("unknown")     -> clicks(X) = Some(5)   (the unknown resolve counts nowhere)
delete(X) -> true; delete(X) -> false; resolve(X) -> None; clicks(X) -> None
shorten(u_new) for 490 more URLs        -> none of them is X
shorten(url_of_X)                       -> Ok(code) that resolves to it again (and is not X)
```

### Edge cases

- Shortening the same URL twice returns the same code; two different URLs never share a code.
- A URL is valid only if it starts with `http://` or `https://`; null, empty, blank and every other scheme are invalid.
- An alias may contain `-` and `_`, but generated codes never do; generated codes are at most 6 characters even after a million URLs.
- A rejected `shorten` (bad URL, bad alias, taken alias) changes nothing: existing mappings and click counts stay as they were.
- An unknown or deleted code: `resolve` gives nothing and counts nothing; `clicks` reports an error (Java) / `None` (Rust).
- Re-registering an alias for the URL it already points to is a success and keeps its click count.
- `delete` of an unknown or already-deleted code returns `false` and changes nothing.
- A deleted generated code is never given to another URL, but its original URL can be shortened again and gets a working code.
- Racing callers: 16 threads shortening the same URL end up with one code; 16 threads claiming one alias leave exactly one winner.

## Glossary

- **Short code**: the short string that stands for a long URL.
- **Generated code**: a code the service picks itself; only characters `0-9A-Za-z`, at most 6 long.
- **Alias (vanity code)**: a code the caller chooses; `A-Za-z0-9_-`, 3 to 20 characters.
- **Resolve**: look up the URL behind a code, which counts as one click (one visit).
- **Click**: one successful `resolve`.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `shorten(String)` and `shorten(String, String alias)`; invalid input throws `IllegalArgumentException`. `resolve` returns `Optional<String>`, `clicks` returns `int` and throws `IllegalArgumentException` for an unknown/deleted code.
- Rust: `shorten` and `shorten_with_alias` return `Result<String, ShortenError>` with `InvalidUrl`, `InvalidAlias`, `AliasTaken`. `resolve` returns `Option<String>`, `clicks` returns `Option<u64>`.
- Rust tests expect `InvalidUrl` for `""` and `"   "`; a taken alias is `AliasTaken`, a malformed one (length 2, length 21, contains a space) is `InvalidAlias`.
- Using an existing generated code as an alias is an error. Rust accepts either `InvalidAlias` or `AliasTaken` for it (a short generated code may be shorter than 3 characters).
- Generated codes must match `[0-9A-Za-z]+` (no `-` or `_`) and have length <= 6. Aliases may contain `-` and `_`.
- Re-registering an alias for the URL it already means returns the alias and counts as success; it does not reset clicks.
- A deleted generated code is never produced again. Shortening the same URL after its code was deleted works and returns a code that resolves to it.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum ShortenError {
    /// null/blank/not an http:// or https:// URL
    InvalidUrl,
    /// alias does not match [A-Za-z0-9_-]{3,20}
    InvalidAlias,
    /// alias already means a different URL (or is an existing generated code)
    AliasTaken,
}

pub trait UrlShortener: Send + Sync {
    /// Short code for an http(s) URL. Shortening the same URL again returns the same code.
    fn shorten(&self, long_url: &str) -> Result<String, ShortenError>;

    /// Like `shorten` but with a caller-chosen code. Re-registering the same alias for the same URL returns it again.
    fn shorten_with_alias(&self, long_url: &str, alias: &str) -> Result<String, ShortenError>;

    /// The URL behind a code, counting one click. `None` (and no click) for an unknown or deleted code.
    fn resolve(&self, code: &str) -> Option<String>;

    /// Number of successful resolves; `None` for an unknown or deleted code.
    fn clicks(&self, code: &str) -> Option<u64>;

    /// Removes a code. Returns false if it did not exist. A deleted code is never handed out again.
    fn delete(&self, code: &str) -> bool;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn UrlShortener>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Round trip: a URL with query and fragment resolves back unchanged; codes are alphanumeric.
- Same URL -> same code; different URLs -> different codes; 10,000 URLs all produce codes of length <= 6.
- Invalid URLs (null/blank/no scheme/ftp) are rejected; unknown codes resolve to nothing.
- Aliases: valid alias works and is idempotent for the same URL; taken, generated-code, too short, too long and illegal-character aliases are rejected and the original mapping is untouched.
- Clicks count only successful resolves (5 resolves + 1 unknown = 5).
- Delete returns true then false, the code stops resolving, `clicks` fails, and 490 later codes never contain the deleted one.
- Concurrency: 16 threads x 2,000 distinct URLs give 32,000 distinct codes that resolve back correctly; 16 threads on one URL get one code; 16 threads racing for one alias produce exactly one winner; 8,000 concurrent clicks are all counted; a 100x repeat with 8 threads x 50 URLs yields exactly 50 codes.
- More edge cases (a large URL volume, aliases colliding with the generator under load) are in `interviewer/`.

## Running

```bash
cargo test -p p82_url_shortener
STRICT=1 cargo test -p p82_url_shortener   # fail instead of skip while unimplemented
cargo test -p p82_url_shortener --features hidden   # interviewer scenarios
cargo play run 82              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.
