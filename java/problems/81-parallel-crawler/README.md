# 81 · Parallel Web Crawler (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A search engine's crawler starts from one URL, downloads that page, reads the links on it, then visits those pages, and so on until nothing new turns up. You are building that **crawler** for a simulated web: the caller supplies a `Fetcher` that returns the links of a page (slowly, and sometimes with an error), and you use up to `workers` threads to visit everything reachable. Pages link to each other in cycles and many pages share the same targets, so every URL must be fetched exactly once. The crawl must also decide for itself when it is finished and only then return the set of URLs it reached.

## Problem

Crawl a web of pages starting from one URL, following every link, using up to N worker threads. A page may be linked
from many places, the links form cycles, pages are slow, and some pages are broken. The crawl must fetch each URL exactly
once, never run more than N fetches at the same time, survive failing pages and - the hard part - know when it is
finished: there is no fixed amount of work, and workers that find the queue empty may still be fed by a worker that is
busy fetching.

## Requirements

1. `crawl(start, workers, fetcher)` returns the set of **all URLs reached**, including `start`.
2. The fetcher (`links(url)`) is provided by the caller. It may be slow, thread-safe, and may return duplicate links and links to itself.
3. Every URL is fetched **exactly once**, however many pages link to it.
4. At most `workers` fetches run at the same time.
5. A page whose fetch fails (exception / `Err`) counts as reached, contributes no links, and does not stop the crawl.
6. `crawl` returns only when all work is done, and leaves no thread running behind it.
7. `workers < 1` is rejected.

## Constraints and thread-safety requirements

- Two workers discovering the same URL must not both fetch it.
- Termination: the crawl is over when the queue is empty **and** no worker is fetching. An idle worker must not exit while another worker may still add links.
- Deep chains (200 pages, one busy worker at a time) and wide fans (1 page linking to 100) both finish.
- Idle workers do not burn CPU.
- Scale: 900-page heavily linked grid, 2,047-page tree; the hidden tests crawl 100,000 pages.

## Examples

```
a -> b, c, b, a        b -> d        c -> d, a        d -> a, e
crawl("a", 3 workers)  -> {a, b, c, d, e}, each fetched once

b is broken, d is broken:  a -> b, c; c -> d
-> {a, b, c, d}   (e is unknown because d failed)
```

### Worked traces

```
web: a -> [b, c, b, a]   b -> [d]   c -> [d, a]   d -> [a, e]   e -> []   lonely -> [a]
crawl("a", 3, fetcher)   -> {a, b, c, d, e}       each of the five fetched exactly once
                           ("lonely" links to a but nothing reachable links to it, so it is not included)

web: a -> [b, c]  b -> [d]  c -> [d]  d -> [e]; fetching b and d throws
crawl("a", 4, fetcher)   -> {a, b, c, d}          b and d are reached but contribute no links, e is never seen
                           b and d were each fetched once, not retried

web: a -> []
crawl("a", 4, fetcher)   -> {a}

chain p0 -> p1 -> ... -> p199, 8 workers   -> all 200 pages (only one fetch can run at a time here, the crawl must not stop early)
crawl("a", 0, fetcher)   -> IllegalArgumentException
```

### Edge cases

- The start URL is always in the result, even if its own fetch fails.
- A failed fetch is never retried; the URL still counts as fetched once and as reached.
- Duplicate links, links to the page itself and links back to earlier pages cause no extra fetches.
- URLs are compared as exact strings (`a` and `A` are different pages; no normalisation).
- `workers` larger than the number of pages is fine; `workers = 1` means fetches never overlap.
- The order in which pages are fetched is not specified.
- Several crawls may run at the same time on the same crawler, each with its own fetcher; they must not mix up their results.
- When `crawl` returns, no fetch is still running and none starts later.

## Glossary

- **Reached**: the start URL, plus every URL listed on a page that was fetched successfully. A page whose own fetch failed is still reached.
- **Fetch**: one call to `Fetcher.links(url)`, which returns the outgoing links of that page.
- **Broken page**: a page whose fetch fails (Java: throws a `RuntimeException`; Rust: returns `Err(FetchError)`).
- **Worker**: one of the at most `workers` threads doing fetches at the same time.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Crawler.crawl(String, int, Fetcher)` returning `Set<String>`; `Fetcher.links(String)` returns `List<String>` and throws a `RuntimeException` for a broken page. Rust: `Fetcher::links -> Result<Vec<String>, FetchError>`, `Crawler::crawl(&self, &str, usize, Arc<dyn Fetcher>) -> Result<HashSet<String>, InvalidWorkers>`.
- `Fetcher` (and `FetchError`) are provided in the contract; you do not write them.
- The tests count fetches per URL, the maximum number of simultaneous fetches, and any fetch that happens *after* `crawl` returned.
- URLs are opaque strings; no normalisation is expected.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Set;

/** Thin test entry point. Your design lives behind it. May be called from several threads (separate crawls). */
public interface Crawler {

    /**
     * Crawls every page reachable from {@code start} and returns the set of all URLs reached (including
     * {@code start} and broken pages). Rules:
     * <ul>
     *   <li>every URL is fetched exactly once, however many pages link to it;</li>
     *   <li>at most {@code workers} fetches run at the same time;</li>
     *   <li>a page whose fetch throws counts as reached but contributes no links; the crawl goes on;</li>
     *   <li>returns only when no work is left, and leaves no thread running behind it.</li>
     * </ul>
     * IllegalArgumentException if workers < 1.
     */
    Set<String> crawl(String start, int workers, Fetcher fetcher);
}

package contract;

import java.util.List;

/**
 * Provided by the tests (the "internet"). Thread-safe, may be slow, may contain duplicate and self links.
 * Throws a RuntimeException for a broken page.
 */
public interface Fetcher {

    /** The outgoing links of a page. */
    List<String> links(String url);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static Crawler create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Concurrency tests run many threads against one object, use deadlock timeouts (a hang becomes a failure) and repeat a workload 100 times to catch rare interleavings.

- No links; cycles, diamonds, self-links and duplicate links.
- 200-page chain with a 1 ms fetch and 8 workers (premature-exit bugs hang or truncate this).
- Binary tree of 2,047 pages; 30x30 grid where every page links to four neighbours (each page is discovered four times).
- Broken pages are reached but not expanded; the rest of the web is still crawled.
- Star with 100 slow leaves and 4 workers: at most 4 and at least 2 fetches ran at once; 1 worker never overlaps.
- After `crawl` returns, no more fetches happen (150 ms later).
- Four crawls running at once through the same crawler.
- **Repeat 100x** on random graphs with random worker counts: result equals a reference BFS and every page is fetched once.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/81-parallel-crawler test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Concurrently`, `Eventually`, `Candidate`): `java/testkit`.
