# 58 · Facade (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — four subsystem interfaces and the `UploadResult` record are provided; `PhotoUploadFacade` is a stub. |

## Problem

You are building the "upload profile photo" feature of an app. Uploading a photo means: validate it, virus-scan it,
resize it, store it, and add it to the search index — and if the last step fails, undo the storage step so no orphan
file is left behind. Every screen in the app was copying these five steps (badly). Provide a single `upload` call that
runs the whole workflow and reports one simple outcome, so callers never touch the four subsystems.

## Terms

- **Blank**: `null`, empty, or only whitespace.
- **Compensation**: undoing an earlier completed step (here `storage.delete(url)`) because a later step failed.
- **Status**: one of `OK`, `REJECTED_INVALID`, `REJECTED_VIRUS`, `FAILED_STORAGE`, `FAILED_INDEXING`.

## Requirements

`PhotoUploadFacade(scanner, resizer, storage, index).upload(fileName, content)` returns an `UploadResult(status, url)`. `url` is `""` for every status except `OK`. Steps run in this order and stop at the first one that ends the upload:

1. File name must be non-blank and end with `.jpg` or `.png` (any case, e.g. `dog.PNG` is valid); content must be non-empty (`null` and `""` are invalid). Otherwise `("REJECTED_INVALID", "")` and **no subsystem is called**.
2. `scanner.isClean(content)` is false → `("REJECTED_VIRUS", "")`; nothing after the scan runs.
3. Resize with `resizer.resize(content, 1024)`.
4. `storage.store(fileName, <resized content>)` (the resized content, not the original). If it throws → `("FAILED_STORAGE", "")`; nothing is indexed and nothing is deleted.
5. `index.index(url, fileName)`, where `url` is what `store` returned. If it throws → `storage.delete(url)` (compensation) and `("FAILED_INDEXING", "")`.
6. Otherwise `("OK", url)`.

The facade can be used for many uploads; one upload does not affect the next.

## Examples

Subsystems: resize returns `content + "@" + maxWidth`; storage returns `https://cdn.test/<fileName>`.

| Upload | Subsystem calls, in order | Result |
|---|---|---|
| `("cat.jpg", "PIXELS")` | `scan(PIXELS)`, `resize(1024)`, `store(cat.jpg, PIXELS@1024)`, `index(https://cdn.test/cat.jpg, cat.jpg)` | `("OK", "https://cdn.test/cat.jpg")` |
| `("bad.jpg", "VIRUS")`, scanner says infected | `scan(VIRUS)` | `("REJECTED_VIRUS", "")` |
| `("notes.txt", "x")`, `(" ", "x")`, `(null, "x")`, `("a.jpg", "")`, `("a.jpg", null)` | none | `("REJECTED_INVALID", "")` each |
| `("a.jpg", "x")`, storage throws | `scan`, `resize`, `store(a.jpg, x@1024)` | `("FAILED_STORAGE", "")` |
| `("a.jpg", "x")`, index throws | `scan`, `resize`, `store`, `index(https://cdn.test/a.jpg, a.jpg)`, `delete(https://cdn.test/a.jpg)` | `("FAILED_INDEXING", "")` |

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement PhotoUploadFacade
```
