# 58 · Facade (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — four subsystem traits and `UploadResult` are provided in `src/contract.rs`; `PhotoUploadFacade` is a stub. |

## Problem

Uploading a profile photo means: validate, virus-scan, resize, store, and add to the search index — and if the last
step fails, undo the storage step. Every screen in the app was copying these five steps (badly). Provide a single `upload` call for it.

## Requirements

`PhotoUploadFacade::new(scanner, resizer, storage, index).upload(file_name, content)` returns an `UploadResult { status, url }`:

1. File name must be non-blank and end with `.jpg` or `.png` (any case), content must be non-empty. Otherwise `("REJECTED_INVALID", "")` and **no subsystem is called**.
2. `scanner.is_clean(content)` is false → `("REJECTED_VIRUS", "")`; nothing after the scan runs.
3. Resize with `resizer.resize(content, 1024)`.
4. `storage.store(file_name, <resized content>)`. On `Err` → `("FAILED_STORAGE", "")`; nothing is indexed.
5. `index.index(url, file_name)`. On `Err` → `storage.delete(url)` (compensation) and `("FAILED_INDEXING", "")`.
6. Otherwise `("OK", url)`.

## Running

```bash
cargo test -p p58_facade     # from rust/ — tests FAIL until you implement PhotoUploadFacade
cargo play run 58
```
