# Interviewer notes — 81 Parallel Web Crawler

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: race, deadlock, starvation, memory visibility, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- The whole web is the start page (no links).
- A chain: only one worker busy, the rest idle but must not quit early.
- The same URL discovered by many workers at the same moment.
- Fetcher throws on the start page.
- More workers than pages.
- A worker thread dying from an uncaught exception without decrementing `pending` (hang!).

## Likely interviewer questions
- How exactly does your crawl detect that it is finished? What if the check is done before the last worker enqueues its links?
- Why must `visited.add` and the enqueue decision be one atomic step?
- Why fetch outside the lock? What would a lock around the fetch do to throughput?
- What is the memory footprint for 100 million URLs? How would you bound it?
- Per-host politeness: how does it complicate termination?

## Invariants
- `pending == queued + in-flight`; the crawl ends exactly when `pending == 0`.
- A URL is in `visited` iff it has been (or will be) queued once.
- No fetch starts after `crawl` returns.

## Common mistakes
- Workers exit when the queue is momentarily empty (truncated crawl, flaky).
- Check-then-add on the visited set (duplicate fetches).
- Decrementing `pending` before the new links were enqueued (early termination).
- Not catching fetcher exceptions: the worker dies and `pending` never reaches zero.
- Holding the lock during the fetch (parallelism = 1).
- Leaving worker threads alive after returning.

## Over-engineering traps
- A scheduler/priority-queue/URL-frontier architecture.
- Robots.txt, retries and politeness before termination works.

## Concurrency discussion
- The core is distributed-termination detection on a single machine: 'queue empty' is not 'done'. A pending counter (or a phaser/`Phaser`/wait-group) is the standard tool.
- Executor-per-URL is concise but the recursion of submit-from-task hides the queue; an explicit worker loop makes shutdown and back-pressure visible.

## Trade-off questions to ask when the candidate is done
- Why this termination rule? Walk me through a race you avoided.
- What happens at 10,000 workers? What bounds the number of fetches really worth running?
- How would you test termination bugs, which are timing dependent? (chains, slow fetchers, 100 random graphs)
- Could the visited set be lock-free? What about the frontier?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
