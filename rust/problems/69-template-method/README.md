# 69 · Template Method (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `ImportReport` and `ImportError` are provided; the `DataImporter` trait and `CsvImporter` are stubs. |

## Problem

A data team imports files from many sources (CSV, JSON, a REST API). Every importer repeats the same dance — open,
read records, skip bad ones, transform, save, always clean up — and each copy handles errors a little differently.
Write the dance once.

## Requirements

1. `DataImporter::run` imports as follows:
   - `open()`; on `Err`, `run` returns it and `close()` is **not** called.
   - For each record from `read_record()` (until `Ok(None)`): count it as *read*; if `validate` is false count it *skipped*; else `transform` then `save`, counting *saved*; if either returns `Err` count it *failed* and carry on.
   - `close()` **always** runs once `open` succeeded — also when `read_record()` fails, in which case that error is returned after `close()`.
   - Returns `Ok(ImportReport { read, saved, skipped, failed })`.
2. Steps every importer provides: `open`, `read_record`, `save`. Steps with defaults: `validate` (not blank), `transform` (unchanged), `close` (nothing).
3. `CsvImporter::new(lines, sink)`: skips the header line; a record is valid with exactly 3 comma-separated fields, none blank after trimming; transform trims the fields and joins them with `|`; save appends to the shared `sink`.

## Running

```bash
cargo test -p p69_template_method     # from rust/ — tests FAIL until you implement the stubs
cargo play run 69
```
