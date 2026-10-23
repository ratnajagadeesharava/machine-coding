# Interviewer notes — 107 Excel-Like Autofill Engine

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Labels are bijective base 26: `Z, AA` continues `AB`; `ZY, ZZ` continues `AAA, AAB`; `zz` continues `aaa`.
- A decreasing letter series shrinks across the length boundary (`AB, AA` gives `Z, Y, X`).
- A decreasing letter series whose first step already leaves the alphabet (`C, A`) returns an empty list, not an error.
- Letter step zero (`B, B, B`) repeats the letter.
- Decimals are exact: `0.1, 0.2, 0.3` continues `0.4`; seeds of mixed scale (`1, 1.5, 2.0`) continue `2.5, 3`.
- Negative zero never appears (`-2, -1` continues `0`; seed `-0` is zero).
- Leading zeros are accepted and dropped (`007, 008` continues `9`).
- 15 digits are accepted, 16 are rejected; 10 letters accepted, 11 rejected.
- A bad seed anywhere in the list, including the last one, is an error, also when `count` is 0.
- Every consecutive pair is checked (`2, 4, 6, 9` is an error; only checking the first pair accepts it).
- `count` = 100,000 must be fast (no quadratic string building).

## Likely interviewer questions
- How do you decide which kind a seed list is? What if two kinds could accept the same seeds? (Today they cannot: digits versus letters. Ask again after V3/V4.)
- Why is a mixed list an error and not "treat as text and copy"? What would a real spreadsheet do and why did this spec choose differently?
- What does a decreasing letter series do at `A`, and what alternatives exist (error, wrap to `Z`, repeat)? Why is each surprising?
- Where does number formatting live, and how do you prove it has no floating-point noise?
- How would you add months without touching the numeric code?

## Invariants
- The result length is `count`, except for a decreasing letter series that ran out of labels.
- Output depends only on `(seeds, count)`; there is no state between calls.
- Numeric output matches the plain-decimal format on every path, including zero and negatives.
- An unrecognised seed list always fails, whatever `count` is.

## Common mistakes
- Using `double`/`float`: `0.1 + 0.2` gives `0.30000000000000004`, and `Double.toString` gives `1.0E7` and `3.0`.
- Stripping trailing zeros of an integer: `10` becomes `1` (`stripTrailingZeros` on a scale-0 value with a plain string conversion is fine, text-trimming `0` characters is not).
- Zero-based base 26 for labels: `Z + 1` becomes `BA` (or `A@`), and `ZZ` fails to roll to `AAA`.
- Character arithmetic (`c + 1`) producing `[` after `Z`.
- Losing the case when wrapping (`z` gives `AA`), or accepting `A, b`.
- Computing the step from the last two seeds, or only checking the first pair.
- Treating a single seed as step 0 (copying) instead of step 1.
- Returning an empty list for unrecognised seeds (hides the error), or skipping validation when `count` is 0.
- Emitting `-0` (from `-0.5 + 0.5` in some decimal types) or `0.0`.
- Not trimming the stop rule: producing `@` or an empty string for position 0.

## Over-engineering traps
- A plugin registry, service loader or expression language before two kinds work.
- An abstraction hierarchy with a visitor per series kind and a result type per kind.
- A generic "cell" object, a grid or formulas: the spec is text in, text out.

## Design and scaling discussion
- A detector per kind (`recognise(seeds)` yielding something that can produce the next value) is the natural extension point; the order of detectors becomes semantic as soon as kinds can overlap.
- Series generation is a pure function, so thread-safety is free; any registry added by V4 is the first shared state.
- Producing values lazily (an iterator) versus a list: the contract returns a list, but the internal stream avoids building intermediate collections.
- Number representation: exact decimals (scaled integers) are enough for 15 digits; arbitrary precision is a bounded extension.

## Trade-off questions to ask when the candidate is done
- Error versus lenient copy for unrecognised seeds: who pays for each choice, the user or the caller?
- Why does the step come from the first two seeds and not a regression over all of them? (Excel fits a trend for linear fills; this spec requires an exact progression.)
- How would you test the formatting rules exhaustively without enumerating strings by hand? (Property: parse the output with an independent decimal and compare, plus a regex for the format.)

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
