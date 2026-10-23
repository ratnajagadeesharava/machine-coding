# 107 · Excel-Like Autofill Engine (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

In a spreadsheet you type `1` and `3` into two cells, select them and drag the fill handle down: the sheet works out that you are counting by 2 and fills in `5`, `7`, `9`... Type `A` and `B` and it continues `C`, `D`. You are building that engine: given the text of the selected seed cells and the number of cells to fill, decide which series the seeds form and return the next values. It has no UI and no cells, only text in and text out.

## Problem

Seeds arrive as strings, top to bottom, exactly as they were typed. The engine must recognise two kinds of series, numeric and letter-based, continue them, and refuse anything it cannot recognise, rather than guess. Output must look like something a person would type: no scientific notation, no `2.0` for an integer, no binary floating-point noise such as `0.30000000000000004`, and the case of letters is kept.

## Requirements

1. `autofill(seeds, count)` returns the next `count` values of the series formed by `seeds`, in order, as strings. The seeds are not repeated in the result. The seed list is never modified.
2. **Numeric seed.** A seed is numeric when it is an optional `-`, then one or more digits, then optionally a `.` followed by one or more digits, and nothing else (no `+`, no spaces, no exponent, no thousands separators, no leading or trailing `.`). It has at most 15 digits in total (integer and fraction digits together). Leading zeros are accepted but padding is not preserved (`007` is 7). `-0` is zero.
3. **Letter seed.** A seed is a letter label when it is 1 to 10 ASCII letters, all of the same case. Labels are ordered like spreadsheet column names: `A`..`Z`, then `AA`, `AB`..`AZ`, `BA`..`ZZ`, then `AAA`... So `A` is position 1, `Z` is 26, `AA` is 27, `AZ` is 52, `BA` is 53.
4. **Series.** All seeds must be of the same kind (all numeric, or all letter labels of one case). With two or more seeds the step is the difference between the first two seeds (numeric difference, or difference of letter positions) and every consecutive pair of seeds must have exactly that difference; the series then continues from the last seed by adding the step again and again. A single seed has step `+1`. The step may be negative, zero, or (numeric only) a decimal.
5. **Numeric results.** Arithmetic is exact decimal arithmetic. Results are written in plain decimal notation: a leading `-` for negatives, no `+`, no exponent, no trailing zeros after the decimal point, no `.` when the fractional part is zero, and never `-0` (zero is `0`).
6. **Letter results.** Results are labels in the same case as the seeds. An increasing series wraps past `Z` into two letters (`Z`, `AA`, `AB`) and past `ZZ` into three, and so on. A decreasing series stops before position 1: values that would be before `A` (or `a`) are not produced and the series ends there, so the result may be shorter than `count`, even empty.
7. **Unrecognised seeds are an error**, never an empty result: an empty seed list, a blank or empty seed, mixed kinds (`1`, `A`), mixed case (`A`, `b`, or `Ab`), seeds that are not a single arithmetic progression (`1`, `2`, `4`), malformed numbers (`1e3`, `1.`, `.5`, `+1`), more than 15 digits, more than 10 letters, any other text (`Jan`, `A1`, `?`).
8. **Count.** `count` is the number of values to produce. `count == 0` returns an empty list. Seeds are validated first, so unrecognised seeds are an error even when `count` is 0.
9. The engine may be called from many threads at once; a call must never be affected by another call.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: `Result<Vec<String>, AutofillError>` with `InvalidArgument` (empty seed list) and `UnrecognizedSeries` (any other seeds that are not one recognised series, including blank text). `count` is a `usize`, so it cannot be negative.
- A recognised series with `count == 0` returns an empty list; a decreasing letter series that has run out of labels returns fewer than `count` values (possibly none) and is not an error.
- Numeric output never contains `e`/`E`, `+`, a trailing `.0`, a trailing zero after a decimal point, or `-0`.
- Letter output keeps the case of the seeds.
- Results do not depend on previous calls.

## Examples

```
autofill(&["1","3"], 4)         -> Ok(["5","7","9","11"])
autofill(&["5","3"], 4)         -> Ok(["1","-1","-3","-5"])
autofill(&["0.1","0.2"], 3)     -> Ok(["0.3","0.4","0.5"])
autofill(&["0.5","1.0"], 3)     -> Ok(["1.5","2","2.5"])
autofill(&["a","b"], 2)         -> Ok(["c","d"])
autofill(&["Y"], 3)             -> Ok(["Z","AA","AB"])
autofill(&["D","C"], 5)         -> Ok(["B","A"])      (stops before A)
autofill(&["1","2"], 0)         -> Ok([])
autofill(&["1","A"], 2)         -> Err(UnrecognizedSeries)   (mixed kinds)
autofill(&["1","2","4"], 2)     -> Err(UnrecognizedSeries)   (not arithmetic)
autofill(&[], 2)                -> Err(InvalidArgument)
```

### Worked traces

```
["0.5","1.0"], 3   step = 1.0 - 0.5 = 0.5      1.0 + 0.5 = 1.5   2.0 -> "2"   2.5
["1","0.5"], 3     step = -0.5                 0.0 -> "0"   -0.5   -1.0 -> "-1"
["X","Y"], 4       positions 24, 25 -> step 1  26 = Z   27 = AA   28 = AB   29 = AC
["A","C"], 3       positions 1, 3 -> step 2    5 = E   7 = G   9 = I
["E","C"], 3       positions 5, 3 -> step -2   1 = A   -1 is before A: stop   -> ["A"]
["Z","AA"], 2      positions 26, 27 -> step 1  AB, AC
["1","2","4"], 2   differences 1 and 2 differ  -> error
```

## Edge cases

- A single seed counts up by one, for numbers and letters alike (`["Z"]` gives `AA`, `AB`; `["-1"]` gives `0`, `1`).
- Step zero is a valid series: `["4","4"]` repeats `4`.
- Every consecutive pair is checked, not only the first two seeds.
- Seeds of different decimal lengths mix freely (`["1","1.5","2.0"]` is step 0.5 and continues `2.5`, `3`).
- Crossing zero is normal for numbers (`["3","1"]` gives `-1`, `-3`); there is no lower bound.
- A decreasing letter series can cross a length boundary (`["AB","AA"]` gives `Z`, `Y`).
- `count` may be large (100,000 values must be produced promptly).
- Unrecognised input is always an error, never a partial or empty answer.

## Glossary

- **Seed**: one of the cell values the user selected before dragging.
- **Series**: the sequence of values the seeds form, extended by the same step.
- **Step**: the constant difference between consecutive values of the series.
- **Position** (letters): the 1-based rank of a label in `A`..`Z`, `AA`.., as spreadsheet columns are numbered.
- **Fill count**: how many new values the user asked for.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum AutofillError {
    /// the seed list is empty
    InvalidArgument,
    /// the seeds are not one recognised series (mixed kinds, not arithmetic, malformed, mixed case, blank)
    UnrecognizedSeries,
}

pub trait AutofillService: Send + Sync {
    /// Recognises the series formed by `seeds` ... and returns the next `count` values (full doc in src/contract.rs).
    fn autofill(&self, seeds: &[&str], count: usize) -> Result<Vec<String>, AutofillError>;
}
```

Wire your own types up in `create()` in `src/lib.rs`, which returns `Option<Arc<dyn AutofillService>>`.

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Increasing, decreasing, negative, constant and multi-seed integer series; decimals and their formatting; single numeric seed.
- Letter series in both cases, wrap past `Z`, steps other than 1, single letter seed, a decreasing series stopping at `A`.
- `count` 0 and a large count.
- Every kind of unrecognised input: mixed kinds, non-arithmetic, mixed case, malformed numbers, other text, empty list, and validation when `count` is 0.
- Concurrency: 16 threads call `autofill` at once and every call returns its own correct answer.

## Running

```bash
cargo test -p p107_excel_autofill
STRICT=1 cargo test -p p107_excel_autofill   # fail instead of skip while unimplemented
cargo test -p p107_excel_autofill --features hidden   # interviewer scenarios
cargo play run 107             # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`sut!`, `run_concurrently`): `rust/support`.
