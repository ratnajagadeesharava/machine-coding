# Interviewer notes — 129 Job Posting with Candidate Matching

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- `2 * expected_salary <= max_salary` overflows `u64` (or `i64`) for huge salaries; the correct comparison is `expected_salary <= max_salary / 2` (integer division is exact for non-negative values).
- Tie-breaks compare ids as strings: `"c10"` sorts before `"c2"`, and upper case before lower case.
- A closed job returns 0 from `match_score` even for a perfectly eligible candidate, and an empty list from `top_candidates`, but `applications` still works.
- `apply` must check "closed" before "already applied" before "eligible"; the Rust contract has one error per failure, so the order is visible (`JobClosed`, `AlreadyApplied`, `NotEligible`).
- A skill present in both sets of a job is required only; duplicates after normalisation (`"Go"`, `"go "`) count once.
- A candidate who updated themselves out of eligibility keeps their old application.
- A random-data oracle (rank every candidate by `match_score`, years, id) must agree with `top_candidates` and `top_jobs` for every job and candidate.
- `close_job` racing `apply`: an `apply` that returns normally after `close_job` returned must not exist, and everything accepted must appear in `applications`.
- `top_candidates` while profiles change: no duplicates, no more than `k`, no exceptions; a final quiet read agrees with `match_score`.

## Likely interviewer questions
- Why is 0 a safe "not eligible" sentinel? What would break if the base were 0?
- Where is eligibility checked in `apply`, and could the candidate become ineligible between the check and the insert?
- How does a profile update reach the rankings? What state has to change together?
- What does one `top_candidates` call cost with 5,000 candidates and 500 jobs, and what would you do at 5 million candidates?
- What are the rules for ties, and why does having them fully specified matter for testing and for users?

## Invariants
- Score 0 if and only if not eligible or closed; every eligible score is in 50..=140.
- Normalised skills and locations are the only form stored and compared.
- A job's applications contain each candidate at most once, in application order, and never grow after the job is closed.
- A candidate has exactly one current profile; queries see the old one or the new one, never a mixture.
- A rejected call changes nothing.

## Common mistakes
- Normalising at compare time with `to_lowercase()` on every comparison, or forgetting to trim.
- Treating a skill in both sets as both required and nice-to-have, giving +10 for a skill that is already required.
- Using `years >= min_years` correctly but `salary < max` (off by one), or `2 * salary` overflow.
- Capping the sum instead of each term, or capping the nice-to-have count after multiplying wrongly.
- Giving the location bonus to remote candidates in any city, or withholding it from a remote candidate in the job's city.
- Sorting by score then id and forgetting years in `top_candidates`.
- Scanning and fully sorting every candidate for every call with `to_lowercase` inside the loop (the scale test punishes this).
- Returning a reference into locked state, or holding the lock while the caller uses the result.
- Checking eligibility, then adding the application outside the same critical section as `close_job`.
- A write lock held while sorting, so rankings block every writer for the duration of the sort.

## Over-engineering traps
- A rules engine with pluggable predicates before the five-term formula works.
- An event bus and notification layer for the base requirements (that is V4).
- A persistence layer, a scoring-plugin registry or a generic specification framework.

## Design and scaling discussion
- Patterns an interviewer hopes to hear, each only when justified: a **scoring strategy** (the formula behind one function, swappable per job for V3), an **observer** for notifying matching candidates when a job is posted (V4), an **inverted index by skill** so a job only looks at candidates holding its rarest required skill, immutable value objects for jobs and candidates, and a state machine for the V5 pipeline.
- Trade-off: scan-and-sort is O(candidates log candidates) per call, fine at 5,000; an index brings it to the size of the posting list; a bounded heap brings the sort down to O(n log k).
- Locking: one read-write lock is simplest and correct; per-job locks for applications plus copy-on-write candidate values reduce contention. Index updates must be atomic with profile replacement.
- Skills dictionary: interning skills to small ints makes `containsAll` a bit-set operation.
- At larger scale the index becomes a search engine query with the hard rules as filters and the score as a ranking function; the exactness of the tie-breaks then needs a final stable sort.

## Trade-off questions to ask when the candidate is done
- Would you store the computed score? When does it become stale, and what invalidates it?
- Should `apply` re-check eligibility at the time of the call, or trust an earlier ranking?
- How would you test the rankings without hand-writing 500 expected lists?
- What changes if the score must be explained to a candidate ("why am I a 94?")?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
