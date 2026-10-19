# 86 · Document Versioning with Merge (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the storage core of a collaborative document tool, like a wiki. Every save creates a new version
and old versions are never changed. Two people may open version 3, edit it for a while, and save - the second save is
based on a stale version. Instead of rejecting it, the system should merge the two sets of changes when they touch different
lines and report a conflict only when both people changed the same place. Users also want to see what changed between two
versions and to restore an old version.

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
v1 = "l1\nl2\nl3\nl4\nl5"
ann edits base 1 -> "L1\nl2\nl3\nl4\nl5"                  => v2
bob edits base 1 -> "l1\nl2\nl3\nl4\nL5"  (stale)            => merged v3 = "L1\nl2\nl3\nl4\nL5"
cy  edits base 1 -> "l1\nl2\nCY\nl4\nl5" after ann changed line 3 as well  => conflict, nothing created

diff("a\nb\nc\nd", "a\nB\nc\nd\ne"):
  "  a", "- b", "+ B", "  c", "  d", "+ e"
```

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

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>A document is text; its lines are {@code content.split("\n", -1)} and are joined again with "\n".
 * Versions are numbered 1, 2, 3 ... and are never rewritten (history is append-only).
 */
public interface DocStore {

    /** Creates version 1. IllegalArgumentException if the document already exists. */
    void create(String docId, String author, String content);

    /** Number of the newest version. IllegalArgumentException for an unknown document. */
    int latest(String docId);

    /** Content of a version. IllegalArgumentException for an unknown document or version. */
    String content(String docId, int version);

    /**
     * Saves {@code newContent}, which the author edited starting from {@code baseVersion}, and returns the version
     * that now holds the result.
     * <ul>
     *   <li>base == latest: if the content is unchanged nothing is created and latest is returned; else a new version.</li>
     *   <li>base &lt; latest (stale): three-way merge of base, latest and newContent (see the README for the exact
     *       rules). A clean merge creates a new version with the merged content (or returns latest if the merge equals
     *       latest); an overlapping change throws {@link MergeConflictException} and creates nothing.</li>
     *   <li>IllegalArgumentException if base is not in [1, latest] or the document is unknown.</li>
     * </ul>
     */
    int edit(String docId, String author, int baseVersion, String newContent);

    /** Creates a new version holding the content of an old one (returns latest, creating nothing, if equal). */
    int restore(String docId, String author, int version);

    /** One entry per version, oldest first, formatted "version:author". */
    List<String> history(String docId);

    /**
     * Line diff from one version to another, one entry per output line: "  text" (unchanged), "- text" (removed),
     * "+ text" (added). Exact algorithm in the README.
     */
    List<String> diff(String docId, int fromVersion, int toVersion);
}

package contract;

/** Thrown by {@link DocStore#edit} when a stale edit overlaps a change made since its base version. Provided. */
public class MergeConflictException extends RuntimeException {
    public MergeConflictException(String message) {
        super(message);
    }
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static DocStore create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Create, latest, content, history of a single-version document; duplicate create, unknown document, versions 0 and `latest+1` for content and as a base.
- Sequential edits append versions and keep old ones; an unchanged edit at `latest` creates nothing; restore creates a new version and returns `latest` if the content already is the latest.
- Line diff in both directions plus identity, matching the exact output format.
- Merge: stale edits on different lines, insertion before + deletion after, adjacent lines (no conflict), same line (conflict), two insertions at the same place (conflict), same change twice (accepted, no duplicate version).
- Concurrency: 8 threads edit 8 different lines from base 1 and all are merged (9 versions); 8 threads edit the same line, exactly one wins; mixed writers/readers keep history gapless; 100x repeat with 4 threads.
- More edge cases (200 versions of a 2,000-line document, two stale edits against a moved base) are in `interviewer/`.

## Language notes

`split("\n", -1)` keeps trailing empty lines; `String.join("\n", lines)` reverses it. A `ReentrantLock` per document (or `synchronized (doc)`) around read-merge-append is enough; the concurrency tests only need atomic compare-and-append.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/86-document-versioning test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`): `java/testkit`.
