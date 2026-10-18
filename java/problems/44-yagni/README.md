# 44 · YAGNI — You Aren't Gonna Need It (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 25 min | Level 2 — one stub class, `UrlShortener`. Bodies throw `UnsupportedOperationException`. |

> **Principle:** build what is needed *now*. Features added "just in case" cost time, add bugs, and are usually
> wrong when the real requirement finally arrives.

## Problem

Marketing needs a tiny URL shortener for one campaign. Two operations: **shorten** a URL and **resolve** a code.
That is all.

## Requirements

1. `shorten(url)` returns a short code. Codes are the sequence numbers `1, 2, 3, ...` in lower-case base 36 (`"1"`..`"9"`, `"a"`..`"z"`, `"10"`...). The same URL always gets the same code. The URL is trimmed first.
2. A URL must start with `http://` or `https://` followed by at least one more character; otherwise `IllegalArgumentException` (also for `null`/empty).
3. `resolve(code)` returns the trimmed original URL, or empty for an unknown/`null`/empty code.

## NOT required — do not build (the tests check)

Expiry, custom aliases, click counts / analytics, deleting links, persistence, pluggable code generators,
thread-safety, random codes, interfaces "for later".

- The public surface is **exactly** `shorten`, `resolve` and one public constructor; no public fields.
- The `solution` package has **one** source file, and it contains none of: `interface`, `abstract`, `implements`, `synchronized`, `volatile`, `ConcurrentHashMap`, `Atomic*`, `Executor`, `Thread`, `Instant`, `Duration`, `Random`.

When expiry or analytics really are requested, you will add them then — with real requirements.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement UrlShortener
```
