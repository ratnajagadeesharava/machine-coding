# 80 · Parallel Word Count (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Count how often every word occurs across a large collection of text documents using several threads. The sequential
solution is five lines; the real question is how the threads combine their results without losing counts and without
turning one shared map into a bottleneck. A document can also be huge - splitting *inside* a document must never cut a
word in two.

## Requirements

1. `count(documents, parallelism)` returns a map from word to number of occurrences over **all** documents.
2. A word is a maximal run of ASCII letters `a-z` / `A-Z`. Digits, punctuation, whitespace and non-ASCII characters separate words.
3. Words are case-insensitive and reported in lower case.
4. The result contains no zero entries. An empty input gives an empty map.
5. At most `parallelism` worker threads; `parallelism < 1` is rejected. The input is not modified (it may be an immutable list).

## Constraints and thread-safety requirements

- The result must equal the sequential answer exactly, every time: no lost updates, no double counting.
- Workers must not fight over one hot map entry for every word (think about what happens to 'the' in 1,000 documents).
- A split point inside a document must not turn one word into two.
- The counter may be used from several threads at once for different inputs.
- Scale: 200 documents x 500 words and one 300,000-word document within a second or two; the hidden tests use millions of words.

## Examples

```
docs: ["Hello, hello WORLD!", "world of Hello"]   parallelism 2
-> {hello: 3, world: 2, of: 1}

"abc123def a-b_c"   ->  abc, def, a, b, c       (digits, '-' and '_' separate words)
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Map<String, Integer> count(List<String>, int)`, `IllegalArgumentException` for `parallelism < 1`. Rust: `Result<HashMap<String, u64>, InvalidParallelism>`.
- You may parallelise across documents, inside documents, or both; the tests only check the answer (and that one huge document still comes out right).
- The returned map need not be thread-safe or immutable.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

use std::collections::HashMap;

/// `parallelism` was 0.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidParallelism;

pub trait WordCounter: Send + Sync {
    /// Counts word occurrences over all documents using at most `parallelism` worker threads.
    ///
    /// A word is a maximal run of ASCII letters (a-z, A-Z); every other character (digits, punctuation, spaces,
    /// newlines, non-ASCII) separates words. Words are compared case-insensitively and reported in lower case.
    /// The result has no zero entries. `Err(InvalidParallelism)` if `parallelism == 0`.
    fn count(
        &self,
        documents: &[String],
        parallelism: usize,
    ) -> Result<HashMap<String, u64>, InvalidParallelism>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn WordCounter>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Concurrency tests run many threads against one object, use deadlock timeouts (a hang becomes a failure) and repeat a workload 100 times to catch rare interleavings.

- Case-folding, separators, empty and blank documents, empty input, invalid parallelism.
- 200 generated documents counted at parallelism 1, 2, 3, 4, 8, 16, 64: always equal to a sequential reference.
- One document with 300,000 words at parallelism 1, 4, 8 (split boundaries).
- Fewer documents than threads; immutable input; total of counts equals total words.
- Six threads counting different inputs through the same counter.
- **Repeat 100x** with random shapes and parallelism - a lost update shows up as a wrong count.

## Language notes

`std::thread::scope` with each worker returning its own `HashMap`; merge on the main thread. Chunks of `&[String]` or `&str` can be borrowed without cloning.

## Running

```bash
cargo test -p p80_parallel_word_count
STRICT=1 cargo test -p p80_parallel_word_count   # fail instead of skip while unimplemented
cargo test -p p80_parallel_word_count --features hidden   # interviewer scenarios
cargo play run 80              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `eventually`, `stays`, `watchdog`, `sut!`): `rust/support`. `run_concurrently` aborts the test run with a `TIMEOUT` message after 60 s, so a deadlock fails instead of freezing `cargo test`.
