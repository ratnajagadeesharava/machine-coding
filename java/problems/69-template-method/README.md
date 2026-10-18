# 69 · Template Method (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `ImportReport` is provided; the abstract `DataImporter` (with its steps declared) and `CsvImporter` are stubs. |

> **Pattern:** *Template Method* — define the **skeleton of an algorithm** in a base class and let subclasses override
> specific steps **without changing the algorithm's structure**. The base class calls the steps ("don't call us,
> we'll call you"); resource clean-up and error handling are written once, in the template.

## Problem

A data team imports files from many sources (CSV, JSON, a REST API). Every importer repeats the same dance — open,
read records, skip bad ones, transform, save, always clean up — and each copy handles errors a little differently.
Write the dance once.

## Requirements

1. `DataImporter.run()` is `final` and runs the algorithm in the class Javadoc:
   - `open()`; if it throws, `run` throws and `close()` is **not** called.
   - Then, for each record from `readRecord()` (until it returns empty): count it as *read*; if `validate` is false count it *skipped*; else `transform` then `save`, counting *saved*; if either of those two **throws a `RuntimeException`** count it *failed* and carry on.
   - `close()` **always** runs once reading started — even if `readRecord()` throws, in which case the exception propagates after `close()`.
   - Returns `ImportReport(read, saved, skipped, failed)`.
2. Required steps (abstract, protected): `open`, `readRecord`, `save`. Hooks with defaults (protected, not abstract): `validate` (not blank), `transform` (unchanged), `close` (nothing).
3. `CsvImporter(lines, sink)`: skips the header line; a record is valid with exactly 3 comma-separated fields, none blank after trimming; transform trims the fields and joins them with `|`; save appends to `sink`. Null arguments → `IllegalArgumentException`.

## Constraints (what the tests catch)

- A `Recording` subclass in the test logs every step: the exact order of `open / read / validate / transform / save / close` is asserted.
- A minimal anonymous subclass implementing only the three required steps must work, using your defaults.
- Reflection: `run` is `final`; `open`, `readRecord`, `save` are `protected abstract`; `validate`, `transform`, `close` are `protected` and not abstract.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
