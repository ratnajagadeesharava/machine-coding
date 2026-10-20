# 44 · YAGNI — You Aren't Gonna Need It (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 25 min | Level 2 — a `UrlShortener` stub is provided, plus `ShortenError`; you write the bodies. |

## Problem

Marketing needs a tiny URL shortener for one campaign. Two operations: **shorten** a URL and **resolve** a code.
That is all.

## Requirements

1. `shorten(url)` returns a short code. Codes are the sequence numbers `1, 2, 3, ...` in lower-case base 36 (`"1"`..`"9"`, `"a"`..`"z"`, `"10"`...). The same URL always gets the same code. The URL is trimmed first.
2. A URL must start with `http://` or `https://` followed by at least one more character; otherwise `Err(ShortenError::InvalidUrl)`.
3. `resolve(code)` returns the trimmed original URL, or `None` for an unknown or empty code.

## Running

```bash
cargo test -p p44_yagni     # from rust/ — tests FAIL until you implement UrlShortener
cargo play run 44
```
