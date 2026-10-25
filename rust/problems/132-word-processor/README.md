# 132 · Word Processor (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 75 min | Level 2 — contract and tests provided; the document model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

The editing core of a word processor like MS Word. A document is a sequence of characters and **every character has its own formatting**: font, size, bold, italic, underline and colour. People type text, select a range and make it bold, search, replace, and set the alignment of a paragraph. Nobody notices that a long document might hold a million characters that all look the same; the design must not either. You are building the document itself: editing, formatting, searching, counting and paragraph alignment, safe to use from several threads. (Undo/redo is out of scope.)

## Problem

A document is text plus, for every character, a **format**: the tuple (font name, size, bold, italic, underline, colour name). A new document is empty. Text is inserted and deleted by position, formats are changed for a half-open range of positions, and the document can report its text, its formatting as a list of **runs**, and answer search, replace and counting questions. Lines (separated by `'\n'`) carry a paragraph alignment.

The document may become large (the tests build one with **1,000,000 characters** in a handful of formats, edit it and query it, all within a generous time limit of a few seconds), so the work per operation must not be proportional to the number of characters when the formatting itself is simple.

## Requirements

1. **Default format**: `("Calibri", 11, not bold, not italic, not underlined, "black")`.
2. **`insert(pos, text)`**, `0 <= pos <= length`. The inserted characters take the format of the character just before `pos`. At `pos == 0`, or in an empty document, they take the default format (even if the character that is currently first is formatted). Inserting `""` is allowed and changes nothing. `pos > length` is an error.
3. **`delete(from, to)`** removes the half-open range `[from, to)`, valid when `0 <= from <= to <= length`. An empty range changes nothing. Anything else is an error.
4. **`applyFormat(from, to, change)`**: same range rules as `delete`. `change` is a small set of optional fields; only the fields that are given change, the other fields of each character keep their values. Fields: `font` (non-blank, no `|`), `size` (decimal digits only, `1..=1000`), `bold`, `italic`, `underline` (exactly `"true"` or `"false"`), `color` (non-blank, no `|`). An unknown field, a repeated field or an invalid value is an error, and then **nothing** is applied. An empty change, or an empty range, changes nothing.
5. **`formatAt(pos)`**, `0 <= pos < length`, returns the format string `font|size|B|I|U|color`: the font name, the size in decimal, then `B`, `I`, `U` if bold, italic, underlined and `-` if not, then the colour name, separated by `|`. For example `Calibri|11|-|-|-|black` and `Arial|14|B|-|U|red`.
6. **`text()`** is the whole text, **`length()`** its number of characters.
7. **`runs()`** lists the **maximal runs of equal format** in document order, each as `start-end:formatString` with `end` exclusive, for example `0-5:Calibri|11|-|-|-|black`. Runs partition `[0, length)`; two adjacent runs never have equal formats (they are always merged, whatever sequence of edits produced them). An empty document has no runs.
8. **`find(pattern, caseSensitive)`** returns the ascending start indexes of **all** matches, overlapping ones included (`"aa"` in `"aaaa"` is found at 0, 1 and 2). Case-insensitive matching treats the ASCII letters `A-Z` and `a-z` as equal and every other character exactly. Formatting is irrelevant. An empty pattern is an error.
9. **`replaceAll(pattern, replacement, caseSensitive)`** finds matches scanning the **original** text from left to right, **without overlap** (after a match, scanning continues right after it), replaces each one and returns how many were replaced. The replacement is not scanned again. Every character of a replacement takes the format of the **first character of the match it replaces**. The replacement may be empty (the matches are deleted) and may contain `'\n'`. An empty pattern is an error. Alignment follows the rule of requirement 13 as if each match were deleted and its replacement then inserted at the match start.
10. **`wordCount()`** is the number of maximal sequences of non-whitespace characters. Whitespace is exactly space, tab, `'\n'` and `'\r'`.
11. **`lineCount()`** is the number of `'\n'` characters plus one (an empty document has 1 line).
12. **`setAlignment(line, alignment)`** and **`alignment(line)`**, `0 <= line < lineCount`. Alignment is one of `LEFT`, `CENTER`, `RIGHT`, `JUSTIFY` (exact upper-case names). Every line is `LEFT` initially.
13. **Alignment follows the lines.** When text containing `k` newlines is inserted into line `L` (the line that contains position `pos` before the insertion), the `k` new lines created directly after `L` get the alignment of `L`; lines after them shift down unchanged. When a deleted range contains `k` newlines, the `k` lines following the line that contains `from` disappear; the surviving merged line keeps the alignment of the line that contained `from`.
14. **`distinctFormatCount()`** is the number of distinct format values currently used by at least one character (0 for an empty document).
15. Every invalid call leaves the document exactly as it was.

## Constraints

- Thread-safe: any number of threads may call any operation. A single document-wide lock is acceptable. No character may be lost when threads insert concurrently, and a reader must never see a half-applied edit (text and runs always describe the same document).
- A document of 1,000,000 characters that is all in one format apart from 10 formatted ranges must be built, edited (a few thousand inserts and `formatAt` calls) and queried (`runs`, `find`, `wordCount`) in well under 20 seconds. The number of distinct formats, not the number of characters, is what should drive memory.
- Characters are plain code units / chars; the tests use ASCII only.

## Examples

### Worked trace 1: typing and formatting

```
insert(0, "hello world")
runs()                         -> ["0-11:Calibri|11|-|-|-|black"]
applyFormat(0, 5, {bold: true, color: red})
runs()                         -> ["0-5:Calibri|11|B|-|-|red", "5-11:Calibri|11|-|-|-|black"]
insert(5, "!")                 text "hello! world"; "!" takes the format of 'o' (index 4)
runs()                         -> ["0-6:Calibri|11|B|-|-|red", "6-12:Calibri|11|-|-|-|black"]
insert(0, ">")                 text ">hello! world"; position 0 always gets the default format
runs()                         -> ["0-1:Calibri|11|-|-|-|black", "1-7:Calibri|11|B|-|-|red", "7-13:Calibri|11|-|-|-|black"]
distinctFormatCount()          -> 2
applyFormat(1, 7, {bold: false, color: black})
runs()                         -> ["0-13:Calibri|11|-|-|-|black"]      (equal neighbours are merged)
distinctFormatCount()          -> 1
```

### Worked trace 2: search and replace

```
insert(0, "aaaa")
find("aa", true)               -> [0, 1, 2]                (overlapping matches)
replaceAll("aa", "b", true)    -> 2, text "bb"             (scan 0-1 matches, continue at 2)
insert(2, " Cat cAT")          text "bb Cat cAT"
find("cat", true)              -> []
find("cat", false)             -> [3, 7]

insert(0, "the cat the")       (fresh document)
applyFormat(0, 3, {bold: true})
replaceAll("the", "a", true)   -> 2, text "a cat a"
runs()                         -> ["0-1:Calibri|11|B|-|-|black", "1-7:Calibri|11|-|-|-|black"]
                               (first replacement copies the bold 't'; the second copies a plain 't')
```

### Worked trace 3: lines and alignment

```
insert(0, "one\ntwo");  lineCount() -> 2;  wordCount() -> 2
setAlignment(0, CENTER)
insert(3, "\nnew")             text "one\nnew\ntwo"; the new line copies line 0
alignment(0), alignment(1), alignment(2)  -> CENTER, CENTER, LEFT
delete(3, 7)                   text "one\ntwo"; merged lines keep the first line's alignment
alignment(0), alignment(1)     -> CENTER, LEFT
setAlignment(2, LEFT)          -> error (only lines 0 and 1 exist)
```

### Edge cases

- Position 0 always inserts with the default format; a non-zero position copies the preceding character, so typing at the end of bold text is bold.
- Inserting into the middle of a formatted run does not split it into several runs.
- `delete` can make two runs with equal formats adjacent; `runs()` still reports one run.
- A format change that gives every character the same format collapses the document to a single run.
- `find` on an empty document, or with a pattern longer than the text, returns an empty list; `replaceAll` returns 0.
- `replaceAll("aa", "b")` on `"aaa"` replaces once and gives `"ba"`.
- `replaceAll("ab", "abab")` does not loop: the replacement is never scanned again.
- `delete(len, len)` and `applyFormat(len, len, ..)` are valid empty ranges; `formatAt(len)` is an error.
- `applyFormat` with a valid first field and an invalid second field applies neither.
- `wordCount` treats punctuation as part of a word (`"don't stop-me now."` is 3 words); `"\r\n"` is whitespace.
- `lineCount` of `"a\n"` is 2 (an empty last line).

## Glossary

- **Format**: the tuple (font, size, bold, italic, underline, colour) of one character.
- **Run**: a maximal block of consecutive characters whose formats are equal.
- **Change**: the optional fields given to `applyFormat`; fields not given are left alone.
- **Line**: the text between newlines; the unit that carries an alignment.
- **Half-open range**: `[from, to)` includes `from`, excludes `to`.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: `Result<_, WordError>` with `OutOfRange` (a position, range or line outside the document) and `InvalidArgument` (empty pattern, unknown, repeated or invalid format field, unknown alignment name). Positions and lines are `usize`; `apply_format` takes `&[(&str, &str)]`. If a call has both kinds of problem, either error is acceptable; the tests never combine them.
- Format strings, run strings, alignment names and field names are exactly as written above (case-sensitive).
- `runs()` is always the canonical, merged form; `distinctFormatCount()` counts format values, not runs (the same format in two separate runs counts once).
- A failed call changes nothing; a successful one is atomic with respect to other threads.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below. It is **not** your domain model — it is the doorway. Create whatever types you like and wire them up in the entry point.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Positions are character indexes (the tests use ASCII text only). Every error leaves the document unchanged.

#[derive(Debug, PartialEq, Eq)]
pub enum WordError {
    /// a position, range or line outside the document
    OutOfRange,
    /// empty pattern, unknown/invalid format field, duplicate format field, unknown alignment name
    InvalidArgument,
}

pub trait WordProcessor: Send + Sync {
    /// Inserts `text` before character `pos` (0 <= pos <= length; `pos == length` appends).
    /// The new characters take the format of the character before `pos`; at position 0, or in an empty document,
    /// they take the default format ("Calibri", 11, not bold, not italic, not underlined, "black").
    /// Inserting an empty string is allowed and changes nothing. `pos > length` is `OutOfRange`.
    fn insert(&self, pos: usize, text: &str) -> Result<(), WordError>;

    /// Deletes the half-open range [from, to). Requires from <= to <= length (else `OutOfRange`);
    /// an empty range changes nothing.
    fn delete(&self, from: usize, to: usize) -> Result<(), WordError>;

    /// Changes the format of the characters in [from, to) (same range rules as `delete`, `OutOfRange`).
    /// `change` holds only the fields to change as (key, value) pairs; the other fields keep their values.
    /// Keys and values: "font" (non-blank, no '|'), "size" (integer 1..=1000), "bold" / "italic" / "underline"
    /// ("true" or "false"), "color" (non-blank, no '|'). An unknown key, a repeated key or an invalid value is
    /// `InvalidArgument`. An empty slice changes nothing. Nothing is applied if anything is invalid.
    fn apply_format(&self, from: usize, to: usize, change: &[(&str, &str)]) -> Result<(), WordError>;

    /// The format of the character at `pos` (pos < length, else `OutOfRange`) as "font|size|B|I|U|color",
    /// e.g. "Calibri|11|-|-|-|black" or "Arial|14|B|-|U|red".
    fn format_at(&self, pos: usize) -> Result<String, WordError>;

    /// The whole text.
    fn text(&self) -> String;

    /// Number of characters.
    fn length(&self) -> usize;

    /// Maximal runs of equal format, in document order, as "start-end:formatString" (end exclusive),
    /// e.g. "0-5:Calibri|11|-|-|-|black". Empty document: empty vector.
    fn runs(&self) -> Vec<String>;

    /// Start indexes (ascending) of every match, overlapping ones included. An empty pattern is `InvalidArgument`.
    fn find(&self, pattern: &str, case_sensitive: bool) -> Result<Vec<usize>, WordError>;

    /// Replaces the non-overlapping matches found scanning left to right in the original text and returns how many
    /// were replaced. Every character of a replacement takes the format of the first character of the match it
    /// replaced. An empty pattern is `InvalidArgument`.
    fn replace_all(&self, pattern: &str, replacement: &str, case_sensitive: bool) -> Result<usize, WordError>;

    /// Number of maximal sequences of non-whitespace characters (whitespace: space, tab, '\n', '\r').
    fn word_count(&self) -> usize;

    /// Number of '\n' characters plus one.
    fn line_count(&self) -> usize;

    /// Sets the alignment of a line (line < line_count, else `OutOfRange`); alignment is one of
    /// "LEFT", "CENTER", "RIGHT", "JUSTIFY" (else `InvalidArgument`).
    fn set_alignment(&self, line: usize, alignment: &str) -> Result<(), WordError>;

    /// The alignment of a line (line < line_count, else `OutOfRange`); "LEFT" unless set or inherited.
    fn alignment(&self, line: usize) -> Result<String, WordError>;

    /// Number of distinct formats currently used by at least one character (0 for an empty document).
    fn distinct_format_count(&self) -> usize;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn WordProcessor>>
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works.

- Insert/delete/format basics with exact format strings; the default format at position 0 and in an empty document; inheritance from the preceding character.
- Range and value validation, with no partial application.
- Canonical maximal runs: merging after format changes, deletes and inserts; exclusive ends.
- Overlapping `find`, case sensitivity, `replaceAll` counts, non-overlap, no rescan, replacement formatting.
- Word and line counting, alignment set/read/inheritance/merging, distinct format counts.
- Concurrency: 8 threads inserting at the end lose nothing; formatting and reading in parallel stay consistent.
- Scale: one million characters with 10 formatted ranges, built, edited and queried within the time limit.
- More edge cases (a model-based random comparison, heavy fragmentation, million-character replace, concurrent editing) are in `interviewer/`.

## Running

```bash
cargo test -p p132_word_processor
STRICT=1 cargo test -p p132_word_processor   # fail instead of skip while unimplemented
cargo test -p p132_word_processor --features hidden   # interviewer scenarios
cargo play run 132              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`sut!`, `run_concurrently`): `rust/support`.
