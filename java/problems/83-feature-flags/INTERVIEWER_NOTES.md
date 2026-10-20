# Interviewer notes — 83 Feature Flag Service

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Rollout set to 0 on a flag whose default is true: nobody is on.
- A rule that matches after a non-matching rule with the same attribute but a different value.
- The same attribute listed twice in different rules (first match wins even when the later one is more specific).
- Raising and lowering the percentage: lowering turns users off, but only from the top of the bucket order.
- Two flags with the same name prefix or the same user ids must not share buckets.
- Killed flag with rollout changes happening concurrently: the flag stays off.
- Blank user id with an otherwise valid flag is an error even if the flag has no rollout.

## Likely interviewer questions
- What exactly is the bucket function? Why include the flag name in it?
- What does `setRollout(30)` followed by `setRollout(60)` do to users that were on, and why?
- Where does the kill switch sit in the evaluation and why is it not just a rule?
- How do you keep `isEnabled` from blocking while an admin adds a rule?
- What do you return for an unknown flag in production? Is throwing the right call for a hot path?

## Invariants
- Evaluation order is always killed, first matching rule, rollout (if set), default.
- A user's bucket for a flag depends only on (flag, userId), never on the percentage, the time or other users.
- A reader sees either the configuration before an edit or after it, never a mixture.
- Rules are only ever appended; their relative order never changes.

## Common mistakes
- Using `Math.random()` or a counter instead of a hash, so the same user flips between calls.
- Hashing only the user id: all flags share the same buckets and rollouts are perfectly correlated.
- A weak hash or `hashCode() % 100` with negative values, giving a skewed or negative bucket.
- Treating the default as the 'outside rollout' answer instead of false.
- Evaluating the rollout before the rules, or letting the kill switch fall through to a rule.
- Iterating a plain `ArrayList` of rules while another thread appends (ConcurrentModification / lost rule).
- Making `percent` comparisons off by one (`<=` instead of `<`), so 0% leaves one bucket on.

## Over-engineering traps
- A rule engine with an expression parser and operators before equality matching works.
- An interface and a factory per rule type, and an event bus for flag changes.
- Persisting to a database or file as part of the core.

## Design and scaling discussion
- Read-mostly data: copy-on-write snapshots or an `RwLock` with short critical sections beat a global lock. The cost is allocation on each admin edit, which is rare.
- Consistent hashing of (flag, user) into 0..9999 gives finer rollouts (0.01%) without changing the scheme.
- Evaluation can be done in the SDK from a cached ruleset (local evaluation) or in the service (remote evaluation); the first is faster and needs a safe propagation story for kill switches.
- Percent rollouts are per-user sticky; per-request randomness is a different feature (sampling) and must not be mixed in.
- Storage: configs are small, so keep the whole set in memory on every node and replicate changes with a version number.

## Trade-off questions to ask when the candidate is done
- Snapshot per flag vs one global snapshot for all flags: what consistency do you want across two flags edited together?
- Would you validate attributes against a schema at the time a rule is added? What does it cost admins?
- How would you prove the bucket distribution is uniform in a test without being flaky?
- What is the blast radius of a bad hash change in a deploy, and how would you prevent it?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
