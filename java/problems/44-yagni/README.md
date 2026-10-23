# 44 · YAGNI — You Aren't Gonna Need It (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 25 min | Level 2 — a `UrlShortener` stub is provided; you write the bodies. |

## Problem

**What you are building.** Marketing is running one campaign and wants short links for a handful of long URLs. You build a tiny URL shortener with exactly two operations: **shorten** a URL (get back a short code) and **resolve** a code (get the original URL back). Nothing else has been asked for: no deleting, statistics, custom aliases or expiry. You write the `UrlShortener` class: a public no-argument constructor, `shorten` and `resolve`.

## Requirements

Operations: `new UrlShortener()`, `String shorten(String url)`, `Optional<String> resolve(String code)`.

1. `shorten(url)` returns a short code. Codes are the sequence numbers `1, 2, 3, ...` written in lower-case base 36: digits `0-9` then letters `a-z`, so the 1st URL gets `"1"`, the 9th `"9"`, the 10th `"a"`, the 11th `"b"`, the 35th `"z"`, the 36th `"10"`, the 37th `"11"`, and so on. A new `UrlShortener` is empty; the first valid URL gets code `"1"`.
2. The URL is **trimmed** (leading and trailing whitespace removed) before anything else. The same (trimmed) URL always gets the same code, and a repeat does not use up a sequence number. Different URLs get consecutive numbers in the order they were first shortened.
3. A URL must start with `http://` or `https://` followed by at least one more character (checked after trimming); otherwise `IllegalArgumentException`, including for `null` and the empty string. A rejected call does not use up a sequence number (the next valid URL still gets the next number).
4. `resolve(code)` returns an `Optional<String>`: the trimmed original URL for a known code, or `Optional.empty()` for an unknown, `null` or empty code. It never throws.

## Examples

Sequence and de-duplication:

```
s = new UrlShortener()
s.shorten("https://example.com/a")      -> "1"
s.shorten("https://example.com/b")      -> "2"
s.shorten("https://example.com/a")      -> "1"   (same URL, same code, no new number used)
s.shorten("  https://example.com/b  ")  -> "2"   (trimmed first, so same URL)
```

Base 36: after shortening 10 distinct URLs (codes `"1"` to `"a"`), the 11th distinct URL gets `"b"`.

Resolving:

```
s = new UrlShortener()
s.shorten("  https://example.com/long/path?q=1  ")  -> "1"
s.resolve("1")    -> Optional.of("https://example.com/long/path?q=1")   (trimmed)
s.resolve("zzz")  -> Optional.empty()     s.resolve("") / s.resolve(null) -> Optional.empty()
```

Invalid URLs (the shortener is empty beforehand; the next valid URL still gets `"1"`):

| Call | Result |
|---|---|
| `shorten("ftp://example.com")` | `IllegalArgumentException` (wrong scheme) |
| `shorten("example.com")` | `IllegalArgumentException` (no scheme) |
| `shorten("https://")` | `IllegalArgumentException` (nothing after the scheme) |
| `shorten("")` / `shorten(null)` | `IllegalArgumentException` |

## Edge cases

- Whitespace is trimmed before validation, so `"  https://example.com  "` is valid and equal to `"https://example.com"`.
- `null` and empty input to `shorten` are rejected like any other invalid URL; `null` or empty input to `resolve` simply yields empty.
- Only `http://` and `https://` are accepted; the scheme must be followed by at least one character.

## Glossary

- **Short code**: the string returned by `shorten`, the sequence number of the URL written in lower-case base 36.
- **Base 36**: counting with 36 symbols, `0-9` then `a-z` (`10` in decimal is `"a"`, `35` is `"z"`, `36` is `"10"`).
- **Resolve**: look a short code up and get back the original URL.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement UrlShortener
```
