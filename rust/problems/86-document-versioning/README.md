# 86 · Document Versioning with Merge (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the storage core of a collaborative document tool, like a wiki. Every save creates a new, immutable version of a document; old versions stay readable forever and a user can restore any of them. Two people may open version 3, edit for a while and both save: the second save is based on a stale version. Instead of rejecting it, the store must merge the two sets of changes when they touch different lines, and report a conflict only when both people changed the same place. Users also ask for a line-by-line diff between any two versions.

## Glossary

- **Version**: a numbered, never-modified snapshot of the full text (1, 2, 3, ... no gaps).
- **Base version**: the version the author started editing from. An edit is *stale* when its base is older than the current latest.
- **Line**: the document text split on `\n`; `"a\nb"` has 2 lines, `""` has 1 empty line, `"a\n"` has 2 lines (`"a"` and `""`).
- **Three-way merge**: combine the changes made `base -> latest` (by others) with the changes made `base -> newContent` (by this author), both measured against the same base.
- **Hunk**: one changed region: base lines `[start, end)` replaced by some new lines. A pure insertion has `start == end`.
- **Conflict**: both sides changed overlapping places, so the merge cannot decide which text wins.

## Requirements

1. A document is text. Its lines are the text split on `\n` (an empty document is one empty line; a trailing `\n` gives a final empty line) and are joined with `\n` again.
2. `create(docId, author, content)` makes version 1; creating an existing document is an error.
3. Versions are numbered 1, 2, 3, ... without gaps, are never changed, and `content(docId, v)` returns any of them. `latest(docId)` is the newest number.
4. `history(docId)` has one entry per version, oldest first, formatted `version:author`.
5. `edit(docId, author, baseVersion, newContent)`: if `base == latest`, unchanged content creates nothing and returns `latest`; changed content creates the next version and returns it.
6. If `base < latest` (stale), do a three-way merge of `base`, `latest` and `newContent`. A clean merge creates a new version holding the merged text and returns it (if the merged text equals `latest`, nothing is created and `latest` is returned). A conflict throws and creates nothing.
7. A `base` outside `[1, latest]` or an unknown document is an error.
8. `restore(docId, author, version)` creates a new version with the content of an old one, or returns `latest` without creating anything when that content already is the latest.
9. `diff(docId, from, to)` returns a line diff: `"  text"` for unchanged lines, `"- text"` for lines removed from `from`, `"+ text"` for lines added in `to`.

## Constraints

- Many authors call the store at once (Java: `Must be thread-safe`; Rust: `&self`, `Send + Sync`).
- The check 'is my base still the latest' and the append of the new version are one atomic step: 8 concurrent edits from the same base must produce a gapless history, and exactly one of several edits to the same line may win.
- Readers never see a half-written version: `latest(d)` is always a number for which `content(d, latest)` and `history(d)` already exist.
- Merging and diffing a 2,000-line document across 200 versions must be fast (hidden test): trim common prefix/suffix or use an O(N*D) diff, avoid O(N^2) memory per diff.
- No clock; version numbers are the only ordering.

## Examples

```
create("doc", "ann", "a")                 -> v1
edit("doc", "bob", 1, "b")                -> returns 2   history: ["1:ann", "2:bob"]
edit("doc", "cy", 2, "b")                 -> returns 2   (unchanged at latest: nothing created)
restore("doc", "cy", 1)                   -> returns 3, content(3) = "a", history ends "3:cy"
restore("doc", "cy", 3)                   -> returns 3   (already the latest content)

Merge, different lines (v1 = "l1\nl2\nl3\nl4\nl5")
ann: edit(base 1) -> "L1\nl2\nl3\nl4\nl5"                          => v2
bob: edit(base 1) -> "l1\nl2\nl3\nl4\nL5"   (stale, other line)     => returns 3, content "L1\nl2\nl3\nl4\nL5", author bob

Merge, adjacent lines are fine
ann: base 1 -> "l1\nl2\nX\nl4\nl5" (line 3)   bob: base 1 -> "l1\nY\nl3\nl4\nl5" (line 2)
                                         => merged "l1\nY\nX\nl4\nl5"

Merge, insertion and deletion elsewhere
ann: base 1 -> "l1\nnew\nl2\nl3\nl4\nl5"   bob: base 1 -> "l1\nl2\nl3\nl5" (deletes l4)
                                         => merged "l1\nnew\nl2\nl3\nl5"

Conflicts (nothing created, latest unchanged)
ann: base 1 -> "l1\nl2\nANN\nl4\nl5"; bob: base 1 -> "l1\nl2\nBOB\nl4\nl5"   => conflict error
v1 = "a\nb"; ann inserts "x" between, bob inserts "y" between (both base 1)    => conflict error

Same change twice is accepted
ann and bob both turn line 3 into "SAME" from base 1 -> bob's call returns 2, history stays length 2

diff(from = "a\nb\nc\nd", to = "a\nB\nc\nd\ne")
  "  a", "- b", "+ B", "  c", "  d", "+ e"
diff in the other direction:
  "  a", "- B", "+ b", "  c", "  d", "- e"
diff of a version with itself: every line prefixed with two spaces
```

## Edge cases

- Invalid input is an error (Java `IllegalArgumentException`, Rust `UnknownDocument` / `UnknownVersion` / `AlreadyExists`): creating an existing document; `latest`, `content`, `edit`, `restore`, `history` or `diff` on an unknown document; version `0` or `latest + 1` for `content`; a base of `0` or `latest + 1` for `edit`.
- Eight concurrent stale edits from base 1, each changing a different line of an 8-line document, must all succeed and end as version 9 with all eight changes applied.
- Eight concurrent edits from base 1 changing the same line: exactly one succeeds, the other seven get the conflict error, and latest is 2.
- Under concurrent writers and readers, `latest(d)` always names a version whose `content` and `history` entry already exist, and `history` entry `i` (1-based) starts with `i:`.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `DocStore`; invalid arguments throw `IllegalArgumentException`; a stale edit that overlaps throws `MergeConflictException` (provided in `contract`). Lines come from `content.split("\n", -1)`.
- Rust: `Result<_, DocError>` with `AlreadyExists`, `UnknownDocument`, `UnknownVersion`, `Conflict`; version numbers are `usize`; lines come from `content.split('\n')`.
- **Diff rule:** compute a longest common subsequence of the two line lists. Output walks both lists in order: common lines are `"  line"`; within every changed region ALL removed lines (`- `) come before ALL added lines (`+ `). Diff of a version with itself lists every line as unchanged. When several LCS exist any one is accepted by the tests.
- **Merge rule:** diff `base -> latest` and `base -> new` into hunks, each hunk replacing a range of base lines `[start, end)` with new lines (a pure insertion has `start == end`). Hunks of the two sides conflict when their base ranges overlap (`a.start < b.end && b.start < a.end`), when both are insertions at the same position, or when an insertion lies strictly inside the other side's changed range. Ranges that only touch (adjacent lines) do not conflict.
- Two identical hunks (same range, same replacement) are applied once (the same change made twice is accepted). Otherwise all hunks of both sides are applied to base in order to produce the merged text.
- A conflict throws/returns the conflict error and leaves `latest`, `content` and `history` unchanged.
- A merge whose result equals `latest` creates no version, and the call returns `latest` (this is why the same change made twice leaves a history of length 2).
- Version author in `history` is the author of the `edit`/`restore` call, including for merged versions.
- `restore` of an old version authored by someone else is recorded as the restoring author.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! A document is text; its lines are `content.split('\n')` and are joined again with "\n".
//! Versions are numbered 1, 2, 3 ... and are never rewritten (history is append-only).

#[derive(Debug, PartialEq, Eq)]
pub enum DocError {
    AlreadyExists,
    UnknownDocument,
    /// version outside 1..=latest
    UnknownVersion,
    /// a stale edit overlaps a change made since its base version
    Conflict,
}

pub type Doc<T> = Result<T, DocError>;

pub trait DocStore: Send + Sync {
    /// Creates version 1.
    fn create(&self, doc_id: &str, author: &str, content: &str) -> Doc<()>;

    /// Number of the newest version.
    fn latest(&self, doc_id: &str) -> Doc<usize>;

    /// Content of a version.
    fn content(&self, doc_id: &str, version: usize) -> Doc<String>;

    /// Saves `new_content`, which the author edited starting from `base_version`, and returns the version that now
    /// holds the result.
    /// * base == latest: if the content is unchanged nothing is created and latest is returned; else a new version.
    /// * base < latest (stale): three-way merge of base, latest and `new_content` (see the README for the exact
    ///   rules). A clean merge creates a new version with the merged content (or returns latest if the merge equals
    ///   latest); an overlapping change returns `Err(Conflict)` and creates nothing.
    /// * `Err(UnknownVersion)` if base is not in 1..=latest.
    fn edit(
        &self,
        doc_id: &str,
        author: &str,
        base_version: usize,
        new_content: &str,
    ) -> Doc<usize>;

    /// Creates a new version holding the content of an old one (returns latest, creating nothing, if equal).
    fn restore(&self, doc_id: &str, author: &str, version: usize) -> Doc<usize>;

    /// One entry per version, oldest first, formatted "version:author".
    fn history(&self, doc_id: &str) -> Doc<Vec<String>>;

    /// Line diff between two versions, one entry per output line: "  text" (unchanged), "- text" (removed),
    /// "+ text" (added). Exact algorithm in the README.
    fn diff(&self, doc_id: &str, from_version: usize, to_version: usize) -> Doc<Vec<String>>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn DocStore>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Create, latest, content, history of a single-version document; duplicate create, unknown document, versions 0 and `latest+1` for content and as a base.
- Sequential edits append versions and keep old ones; an unchanged edit at `latest` creates nothing; restore creates a new version and returns `latest` if the content already is the latest.
- Line diff in both directions plus identity, matching the exact output format.
- Merge: stale edits on different lines, insertion before + deletion after, adjacent lines (no conflict), same line (conflict), two insertions at the same place (conflict), same change twice (accepted, no duplicate version).
- Concurrency: 8 threads edit 8 different lines from base 1 and all are merged (9 versions); 8 threads edit the same line, exactly one wins; mixed writers/readers keep history gapless; 100x repeat with 4 threads.
- More edge cases (200 versions of a 2,000-line document, two stale edits against a moved base) are in `interviewer/`.

## Running

```bash
cargo test -p p86_document_versioning
STRICT=1 cargo test -p p86_document_versioning   # fail instead of skip while unimplemented
cargo test -p p86_document_versioning --features hidden   # interviewer scenarios
cargo play run 86              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.
