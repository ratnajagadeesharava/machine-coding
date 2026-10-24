# 127 · Browser History Manager (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

The brain of a web browser's navigation. A browser has several **tabs**; each tab remembers the pages it has shown so you can press Back and Forward, and following a link from the middle of that trail throws away the pages that were "ahead". Separately, the browser keeps one **history** for the whole window: a time-stamped log of every visit, from which you can list recent pages, look pages up by a piece of the address, find your most visited sites, forget a site, or wipe the last few minutes. The log is bounded, so the oldest visits fall off when it is full. Several threads (one per tab, say) use it at the same time.

## Problem

Build a browser history manager. Time comes from an injected clock, so every timestamp in the tests is predictable. Urls are opaque, case-sensitive strings. A tab shows exactly one page at a time.

## Requirements

1. **Creation.** The manager is created with a global-history `capacity` (at least 1) and a clock. A non-positive capacity or a missing clock is an error.
2. **`openTab()`** returns the new tab id: 1, 2, 3, ... in order of opening. Ids are never reused, even after a tab is closed. A new tab shows `about:blank` and has no earlier or later page. Opening a tab adds nothing to the global history.
3. **`closeTab(tabId)`** closes the tab; from then on its id is unknown (any call using it is an error, including closing it again). Global history entries made from that tab stay.
4. **`visit(tabId, url)`** navigates the tab to `url`:
   - If `url` differs from the tab's current page: the pages ahead of the current one are discarded, `url` becomes the new current page (the page you were on is now one step back), and the tab can no longer go forward.
   - If `url` equals the tab's current page: the tab is left exactly as it was (no new page, pages ahead are kept).
   - In both cases one entry is added to the global history with the clock's current time in milliseconds and the tab id. A blank url is an error. `about:blank` is an ordinary url when visited.
5. **`back(tabId, steps)` / `forward(tabId, steps)`** move the tab by up to `steps` pages (fewer if there are not that many) and return the url now shown. They never fail for lack of pages; `steps == 0` returns the current url; a negative `steps` is an error. Moving does not touch the global history.
6. **`currentUrl`, `canGoBack`, `canGoForward`** report the tab's state; `canGoBack` is true when at least one earlier page exists, `canGoForward` when at least one later page exists.
7. **Global history entries** are formatted `time|url|tabId`, e.g. `1000|https://a.com|2`. The order is by when the entry was recorded, most recent first (entries with the same time keep the order in which they were recorded).
8. **`history(limit)`** returns the `limit` most recent entries (fewer if fewer exist, none for 0; negative is an error).
9. **`historyBetween(fromMs, toMs)`** returns the entries whose time is in `[fromMs, toMs]`, both inclusive, most recent first. `fromMs > toMs` is an error.
10. **`search(substring)`** returns the distinct urls that contain `substring`, compared ignoring case (the returned urls keep their original spelling; two spellings that differ only by case are two different urls). Order: by each url's latest entry, most recent first. A null or blank `substring` is an error.
11. **`mostVisited(k)`** returns at most `k` distinct urls. Order: more entries in the global history first; then the later time of the url's latest entry first; then url ascending (natural string order). Negative `k` is an error. Counts are over the entries the history currently holds.
12. **`deleteFromHistory(url)`** removes every global entry for exactly that url and returns how many were removed (0 if none). Open tabs keep their back/forward pages. A blank url is an error.
13. **`clearHistory(sinceMs)`** removes every global entry whose time is `>= sinceMs` and returns how many were removed.
14. **Capacity.** The global history never holds more than `capacity` entries: when a visit would exceed it, the oldest entries (by recording order) are dropped. Deleted entries free their room. Tabs are not limited by the capacity.
15. **Invalid input.** An unknown tab id, a blank url, a negative count and similar misuse are errors that change nothing (no entry is recorded, no tab moves).

## Constraints

- The manager is used by many threads at once (different tabs, and the same tab). Every call must observe consistent state; no visit may be lost, ids must be unique, and the capacity must hold at every moment.
- Time only comes from the injected clock. Nothing sleeps, nothing reads the wall clock.
- Visiting must stay cheap even with a large capacity (hidden: 300,000 visits with capacity 50,000).

## Examples

```
clock = 1000
openTab()              -> 1
visit(1, "a")  ; visit(1, "b") ; visit(1, "c")      (each after the clock advanced by 10)
back(1, 2)             -> "a"
visit(1, "x")          -> pages ahead ("b", "c") are gone
canGoForward(1)        -> false          forward(1, 5) -> "x"
back(1, 1)             -> "a"            back(1, 1)    -> "about:blank"
```

```
clock = 1000, tabs 1 and 2 are open
visit(1, "https://a.com")      @1000
visit(2, "https://b.com")      @1005
visit(1, "https://c.com")      @1010
history(10)  -> ["1010|https://c.com|1", "1005|https://b.com|2", "1000|https://a.com|1"]
history(2)   -> ["1010|https://c.com|1", "1005|https://b.com|2"]
historyBetween(1000, 1005) -> ["1005|https://b.com|2", "1000|https://a.com|1"]
```

```
visits (one tab, 10 ms apart from 1000): p q r q p z y
mostVisited(10) -> [p, q, y, z, r]
   p: 2 visits, latest 1040      q: 2 visits, latest 1030      (count first, then later latest visit)
   y (latest 1060), z (1050), r (1020): one visit each, later latest visit first
```

```
capacity = 3, visits a b c d e (one tab, 10 ms apart from 1000)
history(10)  -> ["1040|e|1", "1030|d|1", "1020|c|1"]      (a and b were dropped)
search("a")  -> []                                          back(1, 4) -> "a"   (the tab still remembers it)
deleteFromHistory("e") -> 1        clearHistory(1030) -> 1 ("1030|d|1" removed)
```

### Edge cases

- A tab that visits the same url repeatedly (`a, a, a`) has one page `a` after `about:blank`; the global history still has three entries.
- A url equal to a page *ahead* (not the current one) is a normal navigation: the pages ahead are dropped and a new page is added.
- A url equal to an *earlier* page (not the current one) is a normal navigation too: the trail `a, b` plus a visit to `a` has three pages.
- Entries recorded at the same millisecond are ordered by recording order, most recent first. `mostVisited` uses the *time* of the latest entry, so two urls with the same count and the same latest time are ordered by url.
- `clearHistory(0)` empties the history; later visits are recorded normally.
- `deleteFromHistory` and `clearHistory` change what `search` and `mostVisited` see, and the room they free can be reused.
- Huge `steps` values (`Integer.MAX_VALUE` / `i32::MAX`) must not overflow or fail.
- Closing a tab never rewrites history entries; they keep their tab id.

## Glossary

- **Tab**: one browsing context with its own trail of pages.
- **Page / entry of a tab**: a url the tab has shown; the *current page* is the one shown now.
- **Global history**: the window-wide log of visits, shared by all tabs, bounded by the capacity.
- **Visit**: one global-history entry: when, which url, which tab.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for an unknown tab id, a null or blank url / search text, a negative `steps`, `limit` or `k`, `fromMs > toMs`, and for a non-positive capacity or a null clock in the factory. Tab ids are `int`, timestamps `long`; the clock is a `java.time.Clock` (`clock.millis()`).
- Timestamps are the clock's reading at the moment of the `visit` call. Tests only move the clock forward.
- A failed call changes nothing. Tab ids start at 1 and are never reused.
- `deleteFromHistory` and `clearHistory` return the number of removed entries.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below. It is **not** your domain model — it is the doorway. Create whatever you like next to it and wire it up in the factory.

```java
package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface BrowserHistory {
    int openTab();
    void closeTab(int tabId);
    void visit(int tabId, String url);
    String back(int tabId, int steps);
    String forward(int tabId, int steps);
    String currentUrl(int tabId);
    boolean canGoBack(int tabId);
    boolean canGoForward(int tabId);
    List<String> history(int limit);
    List<String> historyBetween(long fromMs, long toMs);
    List<String> search(String substring);
    List<String> mostVisited(int k);
    int deleteFromHistory(String url);
    int clearHistory(long sinceMs);
}
```

The full Javadoc is in `src/main/java/contract/BrowserHistory.java`. Your wiring point (`src/main/java/solution/Solution.java`):

```java
public static BrowserHistory create(int capacity, java.time.Clock clock)
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Tab ids and the blank start page, closing tabs, back/forward by several steps, clamping at both ends, and huge step counts.
- Forward truncation on a new visit, the "same url as current" rule, tabs staying independent.
- Global history format and order, `history(limit)`, inclusive time ranges, case-insensitive search with one row per url, `mostVisited` ordering and tie-breaks.
- `deleteFromHistory` (tabs unaffected), `clearHistory`, capacity eviction and capacity freed by deletes.
- Invalid input everywhere (unknown tab, blank url, negative counts, bad ranges, bad configuration).
- Concurrency: unique tab ids across 16 threads, no lost visits across 8 threads, the capacity holding under concurrent visits and deletes.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/127-browser-history test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`, `Concurrently`): `java/testkit`.
