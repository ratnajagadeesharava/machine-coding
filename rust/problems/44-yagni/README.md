# 44 · YAGNI — You Aren't Gonna Need It (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 25 min | Level 2 — one stub struct, `UrlShortener`, in `src/lib.rs` (`todo!()`); `ShortenError` is provided. |

> **Principle:** build what is needed *now*. Features added "just in case" cost time, add bugs, and are usually
> wrong when the real requirement finally arrives.

## Problem

Marketing needs a tiny URL shortener for one campaign. Two operations: **shorten** a URL and **resolve** a code.
That is all.

## Requirements

1. `shorten(url)` returns a short code. Codes are the sequence numbers `1, 2, 3, ...` in lower-case base 36 (`"1"`..`"9"`, `"a"`..`"z"`, `"10"`...). The same URL always gets the same code. The URL is trimmed first.
2. A URL must start with `http://` or `https://` followed by at least one more character; otherwise `Err(ShortenError::InvalidUrl)`.
3. `resolve(code)` returns the trimmed original URL, or `None` for an unknown or empty code.

## NOT required — do not build (the tests check)

Expiry, custom aliases, click counts, deleting links, persistence, pluggable code generators, thread-safety,
random codes, traits "for later", async.

- `src/lib.rs` has exactly **3** `pub fn` (`new`, `shorten`, `resolve`), exactly one `pub struct`, no `pub trait`, no `pub enum`.
- It does not use: `trait `, `dyn `, `Mutex`, `RwLock`, `Arc<`, `thread`, `Instant`, `Duration`, `rand`, `async `.

When expiry or analytics really are requested, you will add them then — with real requirements.

## Running

```bash
cargo test -p p44_yagni     # from rust/ — tests FAIL until you implement UrlShortener
cargo play run 44
```
