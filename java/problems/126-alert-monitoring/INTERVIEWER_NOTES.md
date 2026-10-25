# Interviewer notes — 126 Alert Monitoring System

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Flapping rule with a 10 s cooldown: of 10 fire/resolve cycles 3 are notified and 7 are counted as suppressed; every `RESOLVED` is notified.
- A silenced `FIRING` neither starts the cooldown nor counts as suppressed; ending the silence does not replay it, but a later `RESOLVED` is delivered.
- Deleting the rule with the largest window shrinks retention at the next `evaluate`; a metric nobody watches is never pruned; retention is per metric, not global.
- A rule defined after samples exist sees the retained samples. A sample recorded at the same millisecond as the evaluation is inside the window.
- The pending timestamp is the evaluation that first saw the condition true, and it is reset when the condition drops and returns.
- `RESOLVED` with the condition true again goes straight to `PENDING`/`FIRING` without an `OK` history entry; the history then shows two entries with the same time.
- Deleting a firing rule notifies nobody and a redefinition carries over no cooldown.
- 300 rules x 3 channels: exact delivery counts, ordering by severity then id.
- Concurrent `record` with a concurrent `evaluate`: exactly 8,000 samples retained, then all pruned after the clock moves.

## Likely interviewer questions
- Where does the rule's state live and who moves it? List every side effect of a transition. Where would a fifth state go?
- How is "aggregate over a window" represented? How many files do you touch to add a sixth aggregation, or a rate-of-change rule?
- Why does `COUNT` behave differently on an empty window? What would break if `AVG` of nothing were `0`?
- Walk me through `forMs = 120000` with evaluations every 60 s: at which evaluation does it fire, and why not earlier?
- What is the difference between cooldown and silence? What happens to each when a rule flaps?
- A channel's notifier hangs for 30 s. What happens to `record` and to the other channels in your design?
- How do you keep memory bounded, and what is the cost of an evaluation with 1 million retained samples?

## Invariants
- A rule makes at most one transition per evaluation; history has one entry per transition and its times never decrease.
- Notifications are produced only by a transition into `FIRING` or `RESOLVED`, never for staying in a state.
- `activeAlerts()` is exactly the set of rules whose state is `FIRING`.
- The sample store is never mutated by `evaluate` other than the final pruning, and pruning never removes a sample a current rule's window still includes.
- Failed calls change nothing.

## Common mistakes
- Closed window `[now - windowMs, now]` (a sample exactly `windowMs` old is wrongly included).
- Pending measured from the first sample time or from the rule definition instead of from the evaluation that first saw the condition true; or using `>` instead of `>=` for `forMs`.
- Notifying on every evaluation while `FIRING`, or on `PENDING` / `OK`.
- Treating an empty window as value 0 for every aggregation (so `MIN LT 5` fires on no data), or forgetting that `COUNT` is the exception.
- Going `RESOLVED` -> `OK` before looking at the condition, losing the transition `RESOLVED` -> `FIRING`.
- Applying the cooldown to `RESOLVED`, or letting a silenced `FIRING` start the cooldown, or counting silenced notifications as suppressed.
- Using `<=` for the cooldown boundary (a re-fire exactly `cooldownMs` later is allowed) or for silence (it ends at `untilMs`).
- One failing notifier aborting the rest of the loop (missing try/catch around each delivery, not around the whole loop).
- Severity routing by string comparison (`"WARN" > "INFO"` alphabetically is accidentally right, `"CRITICAL" < "INFO"` is not).
- Pruning by one global window or pruning in `record`, so a rule defined later cannot see data.
- Iterating a hash map of rules, giving non-deterministic delivery order.
- Returning a live internal list from `history` / `activeAlerts`.
- Reading the clock twice in one evaluation, so rules disagree about `now`.

## Over-engineering traps
- A full event bus, plugin system or message queue for notifications before one synchronous loop is correct.
- A scheduler thread that calls `evaluate` itself (the tests drive it) or any wall-clock dependence.
- A class per state with a transition table loaded from configuration, when five states and one switch/match are enough (a state abstraction is fine if it removes duplication, not by reflex).
- A time-series database layer; an interval tree for the window.

## Design and scaling discussion
- The rule's life cycle is a small state machine; the interesting design choice is whether states are values handled by one transition function or objects that own their transitions, and where "pending since" and "last firing notification" live.
- Aggregation is a function of the samples in the window; a strategy per aggregation (or an enum with behaviour) makes a new aggregation or rate-of-change a local change. Comparators likewise.
- Channels are observers of transitions: the monitor produces events, channels filter by severity and deliver. Keep delivery failures contained per channel.
- Samples: per-metric append-only deque ordered by time, so the window is a suffix found by binary search or backward scan, and retention pops from the front. Running aggregates (sum/count, monotonic deques for min/max) make evaluation O(1) amortised instead of O(window).
- Concurrency: one lock is simple and correct; contention appears when `record` is hot. Alternatives: per-metric locks, a lock-free append buffer drained at evaluation, or copy-on-evaluate. Discuss calling the notifier outside the lock (collect notifications under the lock, deliver afterwards) and the ordering guarantees you give up.
- Evaluate cost is rules x window size; mention sharing the scan between rules on one metric and skipping rules whose metric received no data.

## Trade-off questions to ask when the candidate is done
- Why should `RESOLVED` not be throttled by the cooldown, and what does the on-call person see if it were?
- Is a silenced alert that ends while still firing something the operator would want to hear about? How would you implement "notify when silence ends if still firing"?
- Should delivery failures be retried? Where would the retry queue live and what does it do to ordering?
- Pull vs push: `evaluate()` on demand versus event-driven evaluation on `record`.
- What changes if time can go backwards or samples arrive late with their own timestamps?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
