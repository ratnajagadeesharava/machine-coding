# Interviewer notes — 106 Job Scheduler

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Both criteria interleaved on the same machines: the counters feeding one criterion are changed by jobs submitted under the other.
- Ranking must be restricted to eligible machines: the globally least-loaded or most-experienced machine may not be eligible.
- Tie-break is plain string order of ids (`"m10" < "m2"`), under both criteria, independent of insertion order or hash-map iteration order.
- `MOST_FINISHED` keeps choosing the same machine however loaded it gets.
- A job that found no machine must not be recorded (id stays usable) while a job rejected for a duplicate id leaves the counters alone.
- A duplicate `addMachine` must not replace the capabilities of the original machine.
- A rejected call (unknown criterion, duplicate job, double finish) leaves no trace: no counter change, no consumed job id.
- 16 threads adding the same machine id: exactly one wins. 8 threads submitting the same 500 job ids: each is accepted once and the two machines stay within 1 of each other.
- Machines joining while 8 threads submit and finish: the counters add up (`unfinished + finished == jobs accepted`).
- Large fleet (200 machines, 3,000 jobs, mixed criteria): every assignment satisfies the capability subset rule.

## Likely interviewer questions
- Where does the selection criterion live and who passes it? What is its input (a machine's counters?) and what does it return?
- How would you add a third criterion? Count the files and methods you would touch.
- Why must "choose a machine" and "record the assignment" be one atomic step? Show the interleaving that breaks otherwise.
- How do you find the eligible machines, and what does that cost with 100,000 machines?
- What happens if `finishJob` is called twice concurrently for the same job?
- Why is `Ok(None)` a result and an unknown criterion an error?

## Invariants
- For every machine, `unfinished + finished` equals the number of jobs assigned to it, and a job is counted in exactly one of the two.
- A job is assigned at most once; its machine never changes.
- The chosen machine is always eligible and is the best by the criterion, with the smallest id among equals.
- Failed calls change nothing.

## Common mistakes
- Picking the best machine over all machines and only then checking eligibility (or falling back to a worse one).
- Breaking ties by insertion order or `HashMap` iteration order instead of the smallest id.
- Comparing ids with numeric suffix parsing, so `m10` sorts after `m2`.
- Treating `MOST_FINISHED` as "least loaded among the most finished" or flipping the comparison sign.
- Recording the job (or consuming the job id) before knowing a machine exists, so a failed submission blocks the id.
- Decrementing unfinished on a second `finishJob`, driving counters negative or double-counting finished.
- Keeping references to the caller's capability data instead of owning copies.
- Locking per call in two steps (read counters, then increment), allowing two threads to pick the same machine on stale data.
- Switch statement over criterion names inside the scheduling loop.

## Over-engineering traps
- A strategy hierarchy with factories, registries and plugin loading before the second criterion exists in a real way.
- A priority queue or index per criterion before the simple scan is correct.
- Event buses, job state machines (queued, running, failed, retried) that the requirements do not ask for.
- A `Job` entity graph with back-references when a map from job id to (machine, finished) is enough.

## Design and scaling discussion
- The criterion is naturally a function from a machine's counters (and id) to something comparable; a comparison over machine snapshots keeps the tie-break in one place.
- A global lock makes selection and assignment trivially atomic and is fine up to modest rates; finer locking needs a protocol (lock only the chosen machine and re-validate, or ordered multi-locking).
- Index machines by capability (inverted index, intersect posting lists, start with the rarest capability) to avoid scanning the fleet.
- Per-criterion ranked structures (a heap or `BTreeSet` keyed by counters and id) make selection O(log n) but must be updated on every assignment and finish.
- Memory: finished-job history is not needed beyond the (machine, finished) pair per job id; discuss when ids can be forgotten.

## Trade-off questions to ask when the candidate is done
- One lock for the whole scheduler vs. a lock per machine: what do you gain and what new bugs can appear?
- How do you keep a ranked index consistent with counters that change on `finishJob`?
- Should an unknown criterion fall back to the default or be an error? What is the cost of each in production?
- If criteria become user-defined code, what changes about atomicity and trust?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
