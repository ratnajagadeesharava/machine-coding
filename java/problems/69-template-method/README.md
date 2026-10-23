# 69 · Template Method (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `ImportReport` is provided; `DataImporter` and `CsvImporter` are stubs. |

## Problem

A data team imports files from many sources (CSV, JSON, a REST API). Every importer repeats the same dance — open,
read records, skip bad ones, transform, save, always clean up — and each copy handles errors a little differently.
Write the dance once.

**What you are building:** the shared skeleton for data imports. `DataImporter.run()` performs the whole import in
a fixed sequence, and each concrete importer only supplies the source-specific steps (how to open the source, how to
read the next record, how to save a record, optionally how to validate/transform it and how to clean up). On top of
that you write one real importer, `CsvImporter`, that loads `name,email,city` lines into a list. The result of a run
is an `ImportReport(read, saved, skipped, failed)` of counts.

### Provided types

- `ImportReport(int read, int saved, int skipped, int failed)` — the counts returned by a run.

### Glossary

- **Record**: one raw item from the source (a `String`), e.g. one CSV line.
- **Step**: one of `open`, `readRecord`, `validate`, `transform`, `save`, `close`; all are `protected` methods that importers (including the test's own subclasses) override.
- **Skipped**: read but not valid, so neither transformed nor saved.
- **Failed**: valid, but `transform` or `save` threw.

## Requirements

1. `DataImporter.run()` imports as follows:
   - `open()`; if it throws, `run` throws that exception and `close()` is **not** called (and nothing is read).
   - Then, for each record from `readRecord()` (until it returns an empty `Optional`): count it as *read*; if `validate(record)` is false count it *skipped*; else `transform(record)` then `save(transformed)`, counting *saved*; if either of those two **throws a `RuntimeException`** count it *failed* (not saved) and carry on with the next record.
   - `close()` **always** runs, exactly once, after `open()` succeeded — also if `readRecord()` throws, in which case that exception propagates out of `run` after `close()`. Records handled before the failure were already saved but no report is returned.
   - Returns `ImportReport(read, saved, skipped, failed)`; `read = saved + skipped + failed`.
2. Steps every importer provides: `open`, `readRecord`, `save`. Steps with defaults: `validate` (valid when the record is not blank — empty or whitespace only is invalid), `transform` (returns the record unchanged), `close` (does nothing).
3. `CsvImporter(lines, sink)`: reads the given lines in order; the first line is a header and is skipped (it is not counted as read). A record is valid with exactly 3 comma-separated fields, none blank after trimming (an empty last field as in `"cy,c@x.io,"` still counts as a field, a blank one, so the record is invalid; a blank line is also invalid); transform trims each field and joins them with `|`; save appends the transformed text to `sink`. `null` lines or `null` sink → `IllegalArgumentException`.

## Examples

A test subclass whose `transform` upper-cases, with records `a`, `b`. Events in the order they happen:

```
open, read, validate:a, transform:a, save:A,
      read, validate:b, transform:b, save:B,
      read (-> empty), close          =>  ImportReport(read=2, saved=2, skipped=0, failed=0)
```

Counting rules (records `a`, `X`, `c`):

| Situation | Report `(read, saved, skipped, failed)` |
|---|---|
| `X` is invalid | `(3, 2, 1, 0)` — `transform:X` and `save:X` never happen |
| `transform` throws for `X` | `(3, 2, 0, 1)` — `save` is not called for `X`, `c` is still processed |
| `save` throws for `X` | `(3, 2, 0, 1)` |
| `readRecord` throws on the 2nd record | `run` throws that exception; `close` was still called once; the 1st record had been saved |
| `open` throws | `run` throws; events are only `open` |

Defaults: with only the three required steps, input `"x"`, `"   "`, `"y"`, `""` gives saved `x`, `y` and `ImportReport(4, 2, 2, 0)`.

CSV:

```
lines: "name,email,city"          (header, skipped)
       "ann , a@x.io, Rome"      -> saved "ann|a@x.io|Rome"
       "bob,b@x.io"              -> skipped (2 fields)
       ""                        -> skipped (blank line)
       "cy,c@x.io,"              -> skipped (blank city)
       "dee,d@x.io,Oslo"         -> saved "dee|d@x.io|Oslo"
report: ImportReport(5, 2, 3, 0)
```

## Edge cases

- No lines at all, or only a header line → `ImportReport(0, 0, 0, 0)` and nothing in the sink.
- Whitespace around fields is removed only in the output; validity is decided on trimmed fields.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
