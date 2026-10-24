# Interviewer notes — 138 Multi-Threaded Page Visit Counter

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.
4. The sneaky part is the snapshot. A candidate who has a perfectly good per-page counter will usually discover in the last 10 minutes that "iterate all pages and copy" is not a point-in-time view. Let them find it, then ask what the test would look like (the alternating X/Y writer is the answer).

## Hidden edge cases (see `interviewer/`)
- 30,000,000 visits (5,000,000 visits in the Rust version) from 100 visitors on one page: a list of visit records, or a timestamp per visit, shows up.
- Six spellings of one page hammered by 16 threads: the normalisation must be applied before any map lookup, and `computeIfAbsent`-style creation must not produce two pages for one address.
- 330,000 distinct visitors on one page from 16 threads: exact unique counts need a real set per page, and an HLL-style shortcut fails.
- Snapshots taken while writers visit three pages in a fixed order (a, b, c): the invariant `a >= b >= c >= a - 1` must hold in every snapshot, and the totals per page must be non-decreasing between snapshots.
- Rankings queried while counts change: sorted correctly, never longer than `k`, tie-break on url intact.
- Concurrent `reset` / `resetAll` with visits: the per-visitor sums in `snapshotDetail` must equal the totals in `snapshot` once the dust settles, and no half-reset page may appear (total reset but visitors kept, or the reverse).
- A reset racing with visits to the same page: counts never negative, unique never above the number of visitors that exist, and the page is usable afterwards.

## Likely interviewer questions
- Why can the grand total not simply be a separate counter that every visit increments? (It can, and it is fine; what must it agree with in a snapshot?)
- What is the memory cost of one visit? Of one new visitor on a page? Of a new page?
- What happens between `computeIfAbsent` returning a page and the increment if another thread calls `reset` on that page? Is that visit lost, and is losing it acceptable?
- Which of the read methods are linearizable on their own, and which only weakly consistent (`topPages`, `uniqueVisitors`)? Does the spec require more?
- How would you verify the snapshot guarantee in a test without sleeps?

## Invariants
- A page's total equals the sum of its per-visitor counts, and unique equals the number of visitors, at every snapshot.
- The page identity is the normalised url, computed identically on every code path.
- A rejected call (blank url / visitor, bad `k`) changes nothing, in particular creates no empty page.
- A snapshot corresponds to a serial prefix: it never contains a visit without every visit that completed before it on the same thread.

## Common mistakes
- Normalising in `visit` but not in the reads, or forgetting `reset`.
- Treating `http://a.com` and `http://a.com/` as different pages; stripping only one trailing slash; stripping the root slash completely.
- Lower-casing the whole url including the path, or stripping the query after the slash handling (`/a/?x=1` leaves `/a/`).
- Creating the page entry before validating the visitor id, leaving an empty page behind after the error.
- A `HashMap`-with-`synchronized` check-then-put or `get`-then-`put` on counters: classic lost update under 16 threads.
- Two separate maps (totals and uniques) updated one after the other, so a reader sees `total < unique` or a total that does not match the visitor sum.
- A snapshot that copies page by page without any common point in time: passes single-threaded tests, fails the X/Y alternation check.
- A single global lock around everything: correct but all 16 writers serialise; fine to start with, interesting to improve.
- Sorting all pages for `topPages(1)`; storing one record per visit; `String.format` or regex normalisation on the hot path.
- Locking pages in arbitrary order for the snapshot (deadlock against another multi-page operation).

## Over-engineering traps
- An event log of visits with a background aggregator before the exact counters work.
- HyperLogLog or Bloom filters (that is follow-up V2, and the requirement here is an exact count).
- Time-bucketed windows (that is a different problem, 137) or a plugin system for url rules.
- A class per ranking order or per output format.

## Design and scaling discussion
Patterns and structures an interviewer hopes to hear:
- `RwLock<HashMap<String, Mutex<Page>>>` (visits take the map's read lock and the page's mutex; snapshot takes the map's write lock) or a sharded `Vec<Mutex<HashMap<..>>>` with a fixed lock order; `AtomicU64` for totals where no coupled invariant exists.
- Striped locks / sharding by url hash for writes, with all stripes taken in a fixed order for the snapshot, versus a single read-write lock that makes writers shared and the (rare) snapshot exclusive. The second is simpler and makes the guarantee obvious; the first scales further when snapshots are frequent.
- Alternatives for the snapshot: a global version/sequence counter with per-page copy-on-write, epoch-based copying, or freezing writers briefly. Each trades writer latency against snapshot cost.
- Per-visit cost is O(1) amortised; memory is O(distinct pages + distinct (page, visitor) pairs).
- `topPages(k)` with a bounded heap is O(P log k) instead of O(P log P); with a ranked index maintained on each visit you trade write cost for read cost.
- Sharded counters with periodic aggregation (V6) give up the strict snapshot for throughput; an approximate unique sketch (V2) gives up exactness for memory.

## Trade-off questions to ask when the candidate is done
- Lock striping versus a read-write lock versus lock-free adders: where does each win and lose with 16 threads?
- How stale may a dashboard be? Does it still need a serial-prefix snapshot, or is "eventually correct" enough?
- What would you change if one page got 90 percent of the traffic (hot key)?
- How do you test a consistency guarantee deterministically, and how much does it cost in test time?
- What does `reset` mean for the invariant "totals never decrease", and how would you document it?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test -p p138_page_visit_counter --features hidden`. They only use the same public contract.
