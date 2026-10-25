# Hints — 126 Alert Monitoring System (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Write down, for one rule, the list of things that can change when `evaluate()` looks at it once: its state, its history, its timestamps, the notifications. Which of these depend only on the rule's own past and the current aggregate, and which depend on the outside world (channels, silence)?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Separate three questions: "what is the number and is the condition true?" (the aggregate), "which state does the rule move to?" (the life cycle), and "who is told?" (the channels). The first one is a small replaceable piece of behaviour per aggregation, the second is a state machine with a handful of transitions each having a precise side effect, the third is a set of listeners that filter by severity and must not be able to break each other. Cooldown and silence are decisions made between the second and the third.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: a sample store per metric (time-ordered, newest at the back) with "samples in `(now - w, now]`" and "drop everything at or before `cutoff`" operations; a `Rule` holding its definition, its current state, `pendingSince`, `lastFiringNotifiedAt`, `silencedUntil`, a suppressed counter and its history; an aggregation that maps the window's samples to `(value, conditionIsTrue)`; a single transition step `(rule, conditionTrue, now) -> maybe new state`; and a notification step run only for transitions into `FIRING` / `RESOLVED` that checks silence, then (for `FIRING`) cooldown, then loops over the channels with a per-channel guard. `evaluate()` reads the clock once, walks the rules in id order, then prunes each metric by the largest window of its rules.

**Java:** a `Notifier` that throws is a plain `try/catch` around each single delivery; one `synchronized` (or `ReentrantLock`) around `record`/`evaluate`/queries is enough to start; an `enum` for the state and one for the aggregation with an abstract method is a compact way to keep "what each one does" in one place.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
