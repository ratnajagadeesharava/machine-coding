# 58 · Facade (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — four subsystem interfaces and the `UploadResult` record are provided; `PhotoUploadFacade` is a stub. |

> **Pattern:** *Facade* — provide a single, simple interface to a complex subsystem. Clients call one method; the
> facade knows the order, the error handling and the clean-up across the pieces.

## Problem

Uploading a profile photo means: validate, virus-scan, resize, store, and add to the search index — and if the last
step fails, undo the storage step. Every screen in the app was copying these five steps (badly). Hide them behind a
facade with a single `upload` call.

## Requirements

`PhotoUploadFacade(scanner, resizer, storage, index).upload(fileName, content)` returns an `UploadResult(status, url)`:

1. File name must be non-blank and end with `.jpg` or `.png` (any case), content must be non-empty. Otherwise `("REJECTED_INVALID", "")` and **no subsystem is called**.
2. `scanner.isClean(content)` is false → `("REJECTED_VIRUS", "")`; nothing after the scan runs.
3. Resize with `resizer.resize(content, 1024)`.
4. `storage.store(fileName, <resized content>)`. If it throws → `("FAILED_STORAGE", "")`; nothing is indexed.
5. `index.index(url, fileName)`. If it throws → `storage.delete(url)` (compensation) and `("FAILED_INDEXING", "")`.
6. Otherwise `("OK", url)`.

## Constraints (what the tests catch)

- The subsystems are recording fakes; the tests check the exact call sequence, that the **resized** content is stored, and that a failed index call is followed by the delete.
- The facade can be reused for many uploads.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement PhotoUploadFacade
```
