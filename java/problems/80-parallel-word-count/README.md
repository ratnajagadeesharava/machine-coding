# 80 · Parallel Word Count (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A search team needs word-frequency statistics over a big pile of text: thousands of documents, or a single enormous one. You are building the **word counter**: given a list of documents, it returns how many times each distinct word occurs across all of them, using up to `parallelism` threads to finish faster. The answer must always be exactly what a simple one-thread loop would produce, regardless of how many threads you use or how the text is divided between them.

## Problem

Count how often every word occurs across a large collection of text documents using several threads. The sequential
solution is five lines; the threads' results must be combined without losing counts. A document can also be huge - splitting *inside* a document must never cut a
word in two.

## Requirements

1. `count(documents, parallelism)` returns a map from word to number of occurrences over **all** documents.
2. A word is a maximal run of ASCII letters `a-z` / `A-Z`. Digits, punctuation, whitespace and non-ASCII characters separate words.
3. Words are case-insensitive and reported in lower case.
4. The result contains no zero entries. An empty input gives an empty map.
5. At most `parallelism` worker threads; `parallelism < 1` is rejected. The input is not modified (it may be an immutable list).

## Constraints and thread-safety requirements

- The result must equal the sequential answer exactly, every time: no lost updates, no double counting.
- A split point inside a document must not turn one word into two.
- The counter may be used from several threads at once for different inputs.
- Scale: 200 documents x 500 words and one 300,000-word document within a second or two; the hidden tests use millions of words.

## Examples

```
docs: ["Hello, hello WORLD!", "world of Hello"]   parallelism 2
-> {hello: 3, world: 2, of: 1}

"abc123def a-b_c"   ->  abc, def, a, b, c       (digits, '-' and '_' separate words)
```

### Worked traces

```
["Hello, hello WORLD!", "world of Hello"]  parallelism 2
  -> {hello: 3, world: 2, of: 1}                      (case folded, punctuation ignored)

["abc123def a-b_c\nd\te", "x.y,z;é"]       parallelism 1
  -> {abc:1, def:1, a:1, b:1, c:1, d:1, e:1, x:1, y:1, z:1}
     digits, '-', '_', newline, tab, ';' and the non-ASCII 'é' all separate words

["café"]                                    -> {caf: 1}            ('é' ends the word, is not part of one)
["ab", "cd"]                                -> {ab: 1, cd: 1}      (a word never continues into the next document)
["", "   ", "123 !!"]  or  []               -> {}                  (empty map, no zero entries)
count(["a"], 0)                             -> IllegalArgumentException
```

### Edge cases

- A word ends at the end of its document: the last word of one document and the first word of the next are never joined.
- Any non-ASCII-letter character is a separator, including accented letters and digits; `x2y` is two words, `x` and `y`.
- Upper and lower case are the same word; keys are always lower case.
- The sum of all counts equals the total number of words in all documents.
- Fewer documents than threads, or a single huge document, still gives the exact answer; `parallelism` is only an upper bound.
- The input list is not modified and may be immutable. Different threads may call `count` on different inputs at once.

## Glossary

- **Word**: a maximal run of consecutive ASCII letters `a-z` / `A-Z`; "maximal" means it cannot be extended on either side.
- **Separator**: any character that is not an ASCII letter.
- **Case-insensitive**: `Hello`, `HELLO` and `hello` are the same word, reported as `hello`.
- **Parallelism**: the maximum number of worker threads.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Map<String, Integer> count(List<String>, int)`, `IllegalArgumentException` for `parallelism < 1`. Rust: `Result<HashMap<String, u64>, InvalidParallelism>`.
- You may parallelise across documents, inside documents, or both; the tests only check the answer (and that one huge document still comes out right).
- The returned map need not be thread-safe or immutable.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Map;

/** Thin test entry point. Your design lives behind it. May be called from several threads at once. */
public interface WordCounter {

    /**
     * Counts word occurrences over all documents using at most {@code parallelism} worker threads.
     *
     * <p>A word is a maximal run of ASCII letters (a-z, A-Z); every other character (digits, punctuation, spaces,
     * newlines, non-ASCII) separates words. Words are compared case-insensitively and reported in lower case.
     * The result has no zero entries; it may be a plain HashMap. The input list is not modified.
     * IllegalArgumentException if parallelism < 1.
     */
    Map<String, Integer> count(List<String> documents, int parallelism);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static WordCounter create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Concurrency tests run many threads against one object, use deadlock timeouts (a hang becomes a failure) and repeat a workload 100 times to catch rare interleavings.

- Case-folding, separators, empty and blank documents, empty input, invalid parallelism.
- 200 generated documents counted at parallelism 1, 2, 3, 4, 8, 16, 64: always equal to a sequential reference.
- One document with 300,000 words at parallelism 1, 4, 8 (split boundaries).
- Fewer documents than threads; immutable input; total of counts equals total words.
- Six threads counting different inputs through the same counter.
- **Repeat 100x** with random shapes and parallelism - a lost update shows up as a wrong count.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/80-parallel-word-count test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Concurrently`, `Eventually`, `Candidate`): `java/testkit`.
