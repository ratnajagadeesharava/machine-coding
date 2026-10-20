# 69 · Template Method (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `ImportReport` is provided; `DataImporter` and `CsvImporter` are stubs. |

## Problem

A data team imports files from many sources (CSV, JSON, a REST API). Every importer repeats the same dance — open,
read records, skip bad ones, transform, save, always clean up — and each copy handles errors a little differently.
Write the dance once.

## Requirements

1. `DataImporter.run()` imports as follows:
   - `open()`; if it throws, `run` throws and `close()` is **not** called.
   - Then, for each record from `readRecord()` (until it returns empty): count it as *read*; if `validate` is false count it *skipped*; else `transform` then `save`, counting *saved*; if either of those two **throws a `RuntimeException`** count it *failed* and carry on.
   - `close()` **always** runs once reading started — even if `readRecord()` throws, in which case the exception propagates after `close()`.
   - Returns `ImportReport(read, saved, skipped, failed)`.
2. Steps every importer provides: `open`, `readRecord`, `save`. Steps with defaults: `validate` (not blank), `transform` (unchanged), `close` (nothing).
3. `CsvImporter(lines, sink)`: skips the header line; a record is valid with exactly 3 comma-separated fields, none blank after trimming; transform trims the fields and joins them with `|`; save appends to `sink`. Null arguments → `IllegalArgumentException`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
