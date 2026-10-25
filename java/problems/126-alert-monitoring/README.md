# 126 · Alert Monitoring System (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 75 min | Level 2 — contract, a provided notification sink and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A service receives a stream of numeric measurements (CPU load, request latency, error counts) and must tell people when something is wrong, without spamming them. Operators define **rules** such as "if the average CPU over the last minute is above 80 for at least two minutes, raise a WARN alert". Samples are recorded as they arrive; a periodic **evaluation** looks at each rule, decides whether its condition holds, and moves the rule through its life cycle (quiet, waiting to be sure, alerting, recovered). When a rule starts alerting or recovers, interested **channels** (pager, chat, mail) are told, but only if the alert is serious enough for that channel, only once per event, and not again too soon after the previous page. Operators can also silence a rule while they work on it. Time is read from an injected clock, so tests drive it by hand.

## Problem

Build the monitoring core described above.

- Samples are recorded per metric name and stamped with the clock's time at the moment of the call.
- A rule watches one metric over a sliding time window, aggregates the samples in it (average, maximum, minimum, count or sum), compares the result with a threshold and has a severity.
- Each call to `evaluate()` evaluates every rule at the clock's current time and moves each rule through the states `OK`, `PENDING`, `FIRING`, `RESOLVED`.
- Notifications go to registered channels through a provided `Notifier`, only on the transition into `FIRING` and into `RESOLVED`, filtered by channel severity, throttled by a per-rule cooldown, and withheld while the rule is silenced.
- Old samples are discarded so memory stays bounded.

## Requirements

1. **Time.** All times are integer milliseconds read from the injected clock. The clock never goes backwards. `record` stamps the sample with the clock's value at the call; `evaluate` uses the clock's value at the call as `now`.
2. **Rule condition.** A rule has `metric`, `comparator` (`GT`, `GTE`, `LT`, `LTE`), `threshold`, `windowMs`, `aggregation` (`AVG`, `MAX`, `MIN`, `COUNT`, `SUM`), `severity` (`INFO` < `WARN` < `CRITICAL`), `forMs` and `cooldownMs`. At an evaluation at time `now` the rule's aggregate is computed over the samples of its metric whose timestamp `t` satisfies `now - windowMs < t <= now` (a sample exactly `windowMs` old is outside). The condition is **true** when `aggregate comparator threshold` holds (`GT` is strict `>`, `GTE` is `>=`, `LT` is `<`, `LTE` is `<=`). `AVG` is sum divided by count. `COUNT` is the number of samples as a number.
3. **No samples.** When the window holds no samples the condition is **false** and the rule's value is `0.0`, with one exception: `COUNT` over an empty window is `0` and is compared normally (so `COUNT LT 1` is true on an empty window).
4. **States.** A new rule is `OK` with an empty history. At every evaluation, each rule makes **at most one** transition, decided by its current state and whether the condition is true now:
   - `OK` or `RESOLVED`, condition true: if `forMs == 0` go to `FIRING`, otherwise go to `PENDING` and remember `now` as the time the pending period began. (A rule in `RESOLVED` whose condition is true again does not pass through `OK` first.)
   - `OK`, condition false: stay `OK`.
   - `RESOLVED`, condition false: go to `OK`.
   - `PENDING`, condition false: go to `OK`. Condition true: go to `FIRING` if `now - pendingSince >= forMs`, otherwise stay `PENDING`.
   - `FIRING`, condition false: go to `RESOLVED`. Condition true: stay `FIRING`.
5. **History.** Every transition appends `"<now>:<STATE>"` (the state entered) to the rule's history, for example `"1060000:FIRING"`. A rule that stays in its state adds nothing, and defining a rule adds nothing.
6. **Notifications.** The only events that notify are the transition into `FIRING` and the transition into `RESOLVED`. A notification goes to every registered channel whose `minSeverity` is lower than or equal to the rule's severity, as one `Notifier` call per channel carrying: the channel name, the rule id, the new state (`FIRING` or `RESOLVED`), the rule's severity, the aggregate value computed at this evaluation (`0.0` for an empty window) and `now`. Within one evaluation, rules are processed in ascending rule id (plain string order); for each rule, channels are called in registration order.
7. **Delivery failures.** A `Notifier` call that fails (throws; in Rust returns `Err`) must not stop deliveries to other channels, the evaluation of other rules, or the rule's own state change. A failed delivery still counts as emitted for the cooldown below.
8. **Cooldown.** After a `FIRING` notification was emitted for a rule at time `s`, a later transition into `FIRING` of the same rule at time `now` with `now - s < cooldownMs` sends **no** notification; the state still becomes `FIRING` and the rule's suppressed counter goes up by one. A `FIRING` notification counts as emitted at the transition even if no channel matched or no channel is registered. Cooldown never applies to `RESOLVED` notifications. `cooldownMs == 0` means no throttling.
9. **Silence.** `silence(ruleId, untilMs)` withholds every notification for that rule while `now < untilMs` (at `now == untilMs` the rule is no longer silenced). State tracking and history continue normally. A silenced notification is simply lost: it is not replayed when the silence ends, it does not count as suppressed, and a silenced `FIRING` does not start the cooldown. A later `silence` call for the same rule replaces the earlier one (`untilMs` in the past ends it).
10. **Queries.** `state(ruleId)` returns `OK`, `PENDING`, `FIRING` or `RESOLVED`. `activeAlerts()` returns the ids of rules whose state is `FIRING` (silenced ones included), highest severity first, then rule id ascending. `history(ruleId)` returns the transitions oldest first. `suppressedCount(ruleId)` returns the suppressed counter.
11. **Deleting.** `deleteRule(ruleId)` removes the rule with its state, history, silence and counters; nothing is notified. The id can be defined again and then starts as a brand-new rule.
12. **Retention.** At the end of each `evaluate()`, for every metric watched by at least one rule, samples with `t <= now - W` are discarded, where `W` is the largest `windowMs` among the rules that currently watch that metric. Samples of a metric no rule watches are kept. `sampleCount(metric)` returns the number of retained samples (0 for a metric never recorded). Nothing is discarded by `record`.
13. **Rules see earlier samples.** A rule defined after samples were recorded evaluates against those samples that are still retained.
14. **Concurrency.** `record` is called from many threads, also while another thread evaluates. No recorded sample may be lost and an evaluation must see a consistent set of samples.
15. **Errors**, each leaving all state unchanged: a blank/null metric, rule id or channel name; a `NaN` or infinite sample value or threshold; an unknown (or differently cased) comparator, aggregation or severity name; `windowMs <= 0`; `forMs < 0`; `cooldownMs < 0`; a rule id or channel name that already exists; an unknown rule id in any query, `silence` or `deleteRule`.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for every error in requirement 15 (blank/null names, `NaN`/infinite numbers, unknown names, bad numeric ranges, duplicates, unknown rules). A `Notifier` may throw any `RuntimeException`. `Solution.create(notifier, clock)` throws `IllegalArgumentException` for a null notifier or clock.
- `record`, `defineRule`, `evaluate`, ... are all on one interface; `evaluate()` returns nothing, `activeAlerts()` and `history(...)` return lists of strings.
- Names are the exact upper-case strings listed in the contract constants; they are case-sensitive.
- `Notifier` is **provided**: the tests implement it and record what it receives. `state` in a delivery is `"FIRING"` or `"RESOLVED"`, `severity` is the rule's severity name.
- Fresh monitor: no rules, no channels, no samples.

## Examples

```
t0 = 1_000_000 (clock start)      registerChannel("chat", INFO)
defineRule("cpu-high", metric "cpu", GT, 80, windowMs 60_000, AVG, WARN, forMs 120_000, cooldownMs 300_000)

record("cpu", 90)            evaluate()   -> PENDING   (t0:PENDING)
advance 60_000;  record("cpu", 95); evaluate()
     window is (t0, t0+60_000]: the sample at t0 is out, AVG = 95 > 80, true for 60 s of 120 s -> still PENDING
advance 60_000;  record("cpu", 85); evaluate()
     window (t0+60_000, t0+120_000] holds only 85; true for 120_000 >= forMs -> FIRING
     chat receives (chat, cpu-high, FIRING, WARN, 85.0, t0+120_000)
advance 60_000;  evaluate()
     window (t0+120_000, t0+180_000] is empty -> condition false -> RESOLVED
     chat receives (chat, cpu-high, RESOLVED, WARN, 0.0, t0+180_000)
advance 1;       evaluate()   -> OK (no notification)
history("cpu-high") = [t0:PENDING, t0+120000:FIRING, t0+180000:RESOLVED, t0+180001:OK]
```

```
registerChannel("all", INFO); registerChannel("chat", WARN); registerChannel("pager", CRITICAL)
defineRule("i", "x", GT, 0, 1000, MAX, INFO,     0, 0)
defineRule("w", "x", GT, 0, 1000, MAX, WARN,     0, 0)
defineRule("c", "x", GT, 0, 1000, MAX, CRITICAL, 0, 0)
record("x", 1); evaluate()    deliveries, in this order (rules by id: c, i, w; channels by registration order):
   (all,c)  (chat,c)  (pager,c)       CRITICAL reaches all three channels
   (all,i)                            INFO reaches only "all"
   (all,w)  (chat,w)                  WARN reaches "all" and "chat"
activeAlerts() = [c, w, i]            (CRITICAL, WARN, INFO)
```

```
defineRule("flap", "e", GT, 0, 1000, COUNT, WARN, forMs 0, cooldownMs 10_000); channel "c" at INFO
t0:        record e; evaluate  -> FIRING, notified             (cooldown starts at t0)
t0+1000:   evaluate            -> RESOLVED, notified (cooldown never applies to RESOLVED)
t0+2000:   evaluate            -> OK
t0+3000:   record e; evaluate  -> FIRING again; 3000 < 10_000 so NOT notified; suppressedCount = 1
t0+10000..: a later FIRING at t0+12000 would be notified (12000 - 0 >= 10000)
```

```
defineRule("r", "x", GT, 0, 1000, MAX, WARN, 0, 0); silence("r", t0+5000)
record("x", 1); evaluate()   -> FIRING (state tracked), activeAlerts() = [r], history [t0:FIRING], nothing delivered
```

### Edge cases

- `forMs == 0` goes straight from `OK`/`RESOLVED` to `FIRING`; the history shows only `FIRING`.
- A sample recorded at the same millisecond as the evaluation is inside the window; one exactly `windowMs` older is not.
- `GT` with the aggregate equal to the threshold is false; `GTE` is true.
- An empty window never fires `AVG`, `MAX`, `MIN` or `SUM` rules, whatever the comparator (`MIN LT 5` with no data stays `OK`); only `COUNT` can fire on no data.
- A pending period restarts from scratch if the condition drops and later returns.
- A suppressed (cooldown) `FIRING` is still a real state change: `state` is `FIRING`, `activeAlerts` lists the rule, the history has the entry.
- Two rules with the same severity are listed by rule id; ids compare as plain strings (`"r10" < "r2"`).
- A channel registered after a rule fired does not receive that earlier notification.
- Registering no channel at all is fine; rules still change state and the cooldown still starts.
- Deleting the rule with the largest window shrinks retention at the next `evaluate`, not immediately.

## Glossary

- **Sample**: one numeric measurement of a metric with the clock time at which it was recorded.
- **Window**: the half-open interval `(now - windowMs, now]` of sample times a rule looks at.
- **Aggregate / value**: the single number computed from the samples in the window.
- **Condition**: whether `aggregate comparator threshold` holds at an evaluation.
- **Pending period**: time during which the condition has been continuously true but `forMs` has not yet elapsed.
- **Cooldown**: minimum time between two delivered `FIRING` notifications for one rule.
- **Silence**: an operator-set period during which a rule notifies nobody.
- **Channel**: a named destination with a minimum severity it is willing to receive.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;

/** Provided: implemented by the tests. May throw. */
public interface Notifier {
    void notify(String channelName, String ruleId, String state, String severity, double value, long timeMs);
}

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface AlertMonitor {
    // names: GT GTE LT LTE | AVG MAX MIN COUNT SUM | INFO WARN CRITICAL | OK PENDING FIRING RESOLVED (String constants)
    void record(String metric, double value);
    void defineRule(String ruleId, String metric, String comparator, double threshold, long windowMs,
                    String aggregation, String severity, long forMs, long cooldownMs);
    void deleteRule(String ruleId);
    void registerChannel(String name, String minSeverity);
    void evaluate();
    String state(String ruleId);
    List<String> activeAlerts();
    List<String> history(String ruleId);
    void silence(String ruleId, long untilMs);
    int sampleCount(String metric);
    int suppressedCount(String ruleId);
}
```

Behaviour of each method is documented in the Javadoc of `src/main/java/contract/AlertMonitor.java`.

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static AlertMonitor create(Notifier notifier, java.time.Clock clock)
}
```


## What the tests check

These tests are black-box: they only call the contract above (plus the provided `Notifier` and a fake clock), so any class design behind it works. Nothing depends on wall-clock time or sleeping.

- Comparator strictness, every aggregation, window boundaries, empty windows.
- The full state machine: immediate fire, pending and its boundary, drop back to `OK`, `RESOLVED` then `OK`, `RESOLVED` straight back to firing, restart of the pending period.
- Notification content, order and routing; cooldown including its boundary; silence including its boundary; failing notifiers.
- Active alert ordering, history format, deletion and re-definition, retention, input validation.
- Concurrency: eight threads recording 4,000 samples without loss, and recording while another thread evaluates.
- Harder scenarios (flapping, silence interplay with cooldown, retention after deletion, many rules and channels) are in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/126-alert-monitoring test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`, `Concurrently`): `java/testkit`.
