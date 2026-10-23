# 69 · Template Method (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `ImportReport` and `ImportError` are provided; the `DataImporter` trait and `CsvImporter` are stubs. |

## Problem

A data team imports files from many sources (CSV, JSON, a REST API). Every importer repeats the same dance — open,
read records, skip bad ones, transform, save, always clean up — and each copy handles errors a little differently.
Write the dance once.

**What you are building:** the shared skeleton for data imports. `DataImporter::run` performs the whole import in a
fixed sequence, and each concrete importer only supplies the source-specific steps (how to open the source, how to
read the next record, how to save a record, optionally how to validate/transform it and how to clean up). On top of
that you write one real importer, `CsvImporter`, that loads `name,email,city` lines into a shared list. The result
of a run is an `ImportReport { read, saved, skipped, failed }` of counts.

### Provided types

- `ImportReport { read, saved, skipped, failed }` (all `usize`) — the counts returned by a run.
- `ImportError(pub String)` — the error type of the fallible steps.

### Glossary

- **Record**: one raw item from the source (a `String`), e.g. one CSV line.
- **Step**: one of `open`, `read_record`, `validate`, `transform`, `save`, `close`; implementors (including the test's own types) supply or override them.
- **Skipped**: read but not valid, so neither transformed nor saved.
- **Failed**: valid, but `transform` or `save` returned `Err`.

## Requirements

1. `DataImporter::run` imports as follows:
   - `open()`; on `Err`, `run` returns that error and `close()` is **not** called (and nothing is read).
   - For each record from `read_record()` (until `Ok(None)`): count it as *read*; if `validate(record)` is false count it *skipped*; else `transform(record)` then `save(transformed)`, counting *saved*; if either returns `Err` count it *failed* (not saved) and carry on with the next record.
   - `close()` **always** runs, exactly once, once `open` succeeded — also when `read_record()` fails, in which case that error is returned after `close()`. Records handled before the failure were already saved but no report is returned.
   - Returns `Ok(ImportReport { read, saved, skipped, failed })`; `read = saved + skipped + failed`.
2. Steps every importer provides: `open`, `read_record`, `save`. Steps with defaults: `validate` (valid when the record is not blank — empty or whitespace only is invalid), `transform` (returns `Ok` of the record unchanged), `close` (does nothing).
3. `CsvImporter::new(lines, sink)` (`sink` is an `Rc<RefCell<Vec<String>>>`): reads the given lines in order; the first line is a header and is skipped (it is not counted as read). A record is valid with exactly 3 comma-separated fields, none blank after trimming (an empty last field as in `"cy,c@x.io,"` still counts as a field, a blank one, so the record is invalid; a blank line is also invalid); transform trims each field and joins them with `|`; save pushes the transformed text onto the shared `sink`.

## Examples

A test type whose `transform` upper-cases, with records `a`, `b`. Events in the order they happen:

```
open, read, validate:a, transform:a, save:A,
      read, validate:b, transform:b, save:B,
      read (-> Ok(None)), close        =>  Ok(ImportReport { read: 2, saved: 2, skipped: 0, failed: 0 })
```

Counting rules (records `a`, `X`, `c`):

| Situation | Report `(read, saved, skipped, failed)` |
|---|---|
| `X` is invalid | `(3, 2, 1, 0)` — `transform:X` and `save:X` never happen |
| `transform` returns `Err` for `X` | `(3, 2, 0, 1)` — `save` is not called for `X`, `c` is still processed |
| `save` returns `Err` for `X` | `(3, 2, 0, 1)` |
| `read_record` fails on the 2nd record | `run` returns `Err(ImportError("read failed"))`; `close` was still called once; the 1st record had been saved |
| `open` fails | `run` returns that `Err`; events are only `open` |

Defaults: with only the three required steps, input `"x"`, `"   "`, `"y"`, `""` gives saved `x`, `y` and `Ok(ImportReport { read: 4, saved: 2, skipped: 2, failed: 0 })`.

CSV:

```
lines: "name,email,city"          (header, skipped)
       "ann , a@x.io, Rome"      -> saved "ann|a@x.io|Rome"
       "bob,b@x.io"              -> skipped (2 fields)
       ""                        -> skipped (blank line)
       "cy,c@x.io,"              -> skipped (blank city)
       "dee,d@x.io,Oslo"         -> saved "dee|d@x.io|Oslo"
report: read 5, saved 2, skipped 3, failed 0
```

## Edge cases

- No lines at all, or only a header line → all-zero report and nothing in the sink.
- Whitespace around fields is removed only in the output; validity is decided on trimmed fields.

## Running

```bash
cargo test -p p69_template_method     # from rust/ — tests FAIL until you implement the stubs
cargo play run 69
```
