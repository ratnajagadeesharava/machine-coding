# Hints — 129 Job Posting with Candidate Matching (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Which part of "is this candidate good for this job?" is a yes/no question, and which part is a number? Can the number be computed without first asking the yes/no question?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Treat the scoring rule as one function from (job, candidate) to a number, with 0 meaning "no", and let both rankings be built on it. Normalise skills and locations once when data enters the service and keep them in a form that makes "has all required skills" a cheap check. For the scale test, think about which candidates can possibly be eligible for a job before you score any of them.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Job` and `Candidate` values (normalised sets), a scoring strategy used by both rankings, and one place that owns the maps, the applications and the open/closed flag. Reads rank by sorting only the eligible hits (or keeping a bounded heap of size `k`); writes replace a whole candidate value. An inverted index from skill to candidate ids narrows the candidates of a job to the holders of its rarest required skill; the reverse index (required skill to jobs) does the same for `top_jobs`. Keep `apply` and `close_job` under the same guard so "closed" and "application added" cannot interleave.

**Rust:** an `RwLock` around the whole state is enough to start with; an index of `HashMap<String, HashSet<String>>` must be updated inside the same write guard as the candidate replacement, or an update can be seen half applied.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
