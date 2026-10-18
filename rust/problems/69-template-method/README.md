# 69 · Template Method (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `ImportReport` and `ImportError` are provided; the `DataImporter` trait (steps declared, defaults and `run` are `todo!()`) and `CsvImporter` are stubs. |

> **Pattern:** *Template Method* — define the **skeleton of an algorithm** in a base type and let implementors
> override specific steps **without changing the algorithm's structure**. In Rust the "base class" is a trait: the
> required methods are the steps, and the **provided methods** (`run` and the hook defaults) are the template.

## Problem

A data team imports files from many sources (CSV, JSON, a REST API). Every importer repeats the same dance — open,
read records, skip bad ones, transform, save, always clean up — and each copy handles errors a little differently.
Write the dance once.

## Requirements

1. `DataImporter::run` is the algorithm in the trait's doc comment:
   - `open()`; on `Err`, `run` returns it and `close()` is **not** called.
   - For each record from `read_record()` (until `Ok(None)`): count it as *read*; if `validate` is false count it *skipped*; else `transform` then `save`, counting *saved*; if either returns `Err` count it *failed* and carry on.
   - `close()` **always** runs once `open` succeeded — also when `read_record()` fails, in which case that error is returned after `close()`.
   - Returns `Ok(ImportReport { read, saved, skipped, failed })`.
2. Required steps: `open`, `read_record`, `save`. Provided hooks with defaults: `validate` (not blank), `transform` (unchanged), `close` (nothing). You write these default bodies in the trait.
3. `CsvImporter::new(lines, sink)`: skips the header line; a record is valid with exactly 3 comma-separated fields, none blank after trimming; transform trims the fields and joins them with `|`; save appends to the shared `sink`.

## Constraints (what the tests catch)

- A `Recording` implementor in the test logs every step: the exact order of `open / read / validate / transform / save / close` is asserted.
- A `Minimal` implementor with only the three required steps must work, using your defaults.

## Running

```bash
cargo test -p p69_template_method     # from rust/ — tests FAIL until you implement the stubs
cargo play run 69
```
