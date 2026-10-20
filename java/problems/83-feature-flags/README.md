# 83 · Feature Flag Service (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A company wants to switch features on and off without redeploying. A **feature flag** is a named on/off switch that application code checks with "is this flag on for this user?". You are building the evaluation core: product managers define flags with a default, add targeting rules ("users with `country = IN` get it"), roll the flag out to a percentage of users, and can flip a kill switch that turns it off for everyone at once. The answer for a given user has to be stable (no flickering between calls, and widening a rollout never switches anyone off), and the service must serve many concurrent checks while admins edit flags.

## Problem

Your company ships behind feature flags. Product managers create a boolean flag, target it at specific groups of users
("everyone in India", "plan = pro"), roll it out gradually to a percentage of users, and, when something goes wrong, flip a kill
switch that turns it off for everyone immediately. Application servers ask `is this flag on for this user?` millions of times
a minute while admins edit flags from a console. Build the evaluation core. A user must not flicker: once a user is in a 30%
rollout, the same user stays in it, and raising the rollout to 60% must keep all of those users on.

## Requirements

1. `define(flag, default)` creates a boolean flag. A blank name or an already existing flag is an error.
2. `addRule(flag, attribute, value, result)` appends a targeting rule: a user whose attribute equals `value` gets `result`. Rules are evaluated in insertion order; the first matching rule wins (even if a later rule would give another answer).
3. `setRollout(flag, percent)` with `0 <= percent <= 100` applies to users who match no rule. Once a rollout is set, users inside the percentage get true and everybody else false; the default is no longer used.
4. The same user always lands in the same bucket for the same flag; different flags bucket independently; raising the percentage never turns anybody off.
5. `setKilled(flag, killed)` makes the flag false for everybody regardless of rules, rollout and default. It is reversible.
6. `isEnabled(flag, userId, attributes)` evaluates in this order: killed, first matching rule, rollout (if set), default.
7. An unknown flag, a blank/null `userId`, or a rollout outside 0..100 is an error. Attributes may be empty but not null.

## Constraints

- Many threads call `isEnabled` while others call `define`, `addRule`, `setRollout` and `setKilled`. The service must be thread-safe.
- Evaluations must see a consistent snapshot of a flag: never a half-applied edit (for example a rule list that is being modified).
- No rule may be lost when 16 threads append rules at once; a kill switch set concurrently with rollout changes must keep the flag off.
- Evaluation is the hot path: do not hold a global lock while matching rules (hidden: 1,000,000 evaluations while 2,000 rules are added).
- Bucketing is deterministic and does not use a random generator or the clock; the same inputs give the same answer in every run and on every JVM/process.

## Examples

```
define("beta", false)
addRule("beta", "country", "IN", true)
addRule("beta", "plan", "free", false)
isEnabled("beta", "u1", {country: IN})              -> true    (rule 1)
isEnabled("beta", "u1", {plan: free, country: US})  -> false   (rule 2)
isEnabled("beta", "u1", {})                         -> false   (default)

setRollout("beta", 30)
isEnabled("beta", "u1", {})   -> true for roughly 30% of user ids, the same answer each call
setRollout("beta", 60)        -> everyone who was on at 30% is still on
setKilled("beta", true)       -> false for everybody, including {country: IN}
```

### Worked traces

```
define("on", true); define("off", false)
isEnabled("on", "u1", {})  -> true          isEnabled("off", "u1", {}) -> false      (defaults)

define("beta", false)
addRule("beta","country","IN",true); addRule("beta","country","IN",false)
addRule("beta","plan","free",false); addRule("beta","plan","pro",true)
isEnabled("beta","u",{country:IN})            -> true    (rule 1 matches first; rule 2 is never reached)
isEnabled("beta","u",{plan:pro})              -> true
isEnabled("beta","u",{plan:free, country:US}) -> false   (rule 3 is the first match)
isEnabled("beta","u",{country:US})            -> false   (no rule matches: default)

define("x", true); setRollout("x", 0)
isEnabled("x", any user, {}) -> false                    (rollout set: the default true is ignored)
addRule("x","vip","yes",true)
isEnabled("x","anyone",{vip:yes}) -> true                (a matching rule beats the rollout)
setRollout("x", 100)           -> everybody without a matching rule is on, even if the default is false

define("k", true); addRule("k","vip","yes",true); setRollout("k",100); setKilled("k", true)
isEnabled("k","u",{vip:yes}) -> false;  isEnabled("k","u",{}) -> false
setKilled("k", false)        -> isEnabled("k","u",{}) -> true   (everything is back as it was)

setRollout("x", 101) or -1   -> IllegalArgumentException      isEnabled("nope", ...) -> IllegalArgumentException
```

### Edge cases

- Evaluation order, strictly: killed, then the first matching rule, then the rollout (only if one was ever set), then the default.
- A rule matches when the user's attribute with that exact name has exactly that value (exact string equality). If the user does not supply the attribute, the rule does not match.
- With no rollout set, the default is used for users that match no rule. Once a rollout has been set (even `0`), the default is ignored for those users.
- Rollout `0` turns it off and `100` turns it on for every user without a matching rule.
- Percentages are approximate over many users: 30% of 20,000 users must give 5,400 to 6,600 enabled, and the 30% to 60% growth over 10,000 users must give 5,600 to 6,400 with nobody turned off.
- Two flags with the same percentage must pick largely different users: two 50% flags overlap in 4,400 to 5,600 of 20,000 users (not ~10,000).
- The same user, flag and rollout always give the same answer, in every run and process (no randomness, no clock).
- Adding a rule changes neither the rollout nor the default. Adding a rule to an unknown flag is an error.
- A blank flag name, a null/blank user id, or a rollout outside 0..100 is an error. Empty attributes are fine; null attributes are not (Java).

## Glossary

- **Flag**: a named boolean switch with a default value.
- **Default**: the answer when nothing else applies (and no rollout has been set).
- **Attributes**: the facts about the user passed with each check, e.g. `country = IN`, `plan = pro`.
- **Targeting rule**: "if attribute A equals value V, the answer is R".
- **Rollout**: enabling the flag for a percentage of users who matched no rule.
- **Bucket**: the fixed slot (think 0-99) a given user and flag always map to; a user is in the rollout when their slot falls inside the percentage.
- **Kill switch**: an override that forces the flag off for everyone until switched back.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for blank/duplicate flag names, unknown flags, null or blank `userId`, rollout outside 0..100. `isEnabled` takes a `Map<String,String>`.
- Rust: `Result<_, FlagError>` with `UnknownFlag`, `AlreadyDefined`, `InvalidArgument`; `set_rollout` takes `u32`; attributes are `&[(&str, &str)]`.
- A flag with a rollout of 0 is off for all users who match no rule, even if its default is true; with 100 it is on for all of them even if the default is false.
- Rules override the rollout: a matching rule beats the bucket (rollout 0 plus a matching `result=true` rule is on).
- Killed beats everything, including a rule or a 100% rollout, and `setKilled(false)` restores the previous behaviour with nothing lost.
- Bucketing quality: with 20,000 users a 30% rollout must give between 5,400 and 6,600 enabled; two independent 50% flags must overlap in 4,400-5,600 of 20,000 users; growing 30% -> 60% over 10,000 users must give 5,600-6,400 and turn nobody off.
- Adding a rule to a flag does not change the rollout or default; adding a rule for an unknown flag is an error.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Map;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface FlagService {

    /** Creates a boolean flag. IllegalArgumentException if the name is blank or the flag already exists. */
    void define(String flag, boolean defaultValue);

    /**
     * Appends a targeting rule: users whose attribute {@code attribute} equals {@code value} get {@code result}.
     * Rules are evaluated in the order they were added; the first matching rule wins.
     */
    void addRule(String flag, String attribute, String value, boolean result);

    /**
     * Percentage rollout 0..100 for users that match no rule. Once a rollout is set, users inside the percentage get
     * true and everybody else false (the default value is ignored). Each user must land in the same bucket every time
     * for the same flag, buckets of different flags are independent, and raising the percentage never turns anybody off.
     */
    void setRollout(String flag, int percent);

    /** A killed flag is false for everybody, whatever its rules, rollout or default say. Reversible. */
    void setKilled(String flag, boolean killed);

    /**
     * Evaluation order: killed -> first matching rule -> rollout (if set) -> default.
     * IllegalArgumentException for an unknown flag or a null/blank userId. Attributes may be empty but not null.
     */
    boolean isEnabled(String flag, String userId, Map<String, String> attributes);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static FlagService create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Defaults, and the error cases: unknown flag, duplicate define, blank name, null/blank user id, rollout -1 and 101.
- First-match-wins with duplicate and overlapping rules, no matching rule, and empty attributes.
- Rule over rollout, kill switch over everything (and its reversal), rollout 0/100 extremes over 2,000 users.
- Percentage accuracy (30% of 20,000), stability over repeated calls, monotonic growth 30% -> 60%, independence of two flags.
- Concurrency: 16 threads appending rules lose none; 8 threads defining 200 flags each while evaluating; a 100x repeat of 8 threads adding rules and setting the rollout.
- More edge cases (heavy evaluation during rule edits, kill switch under concurrent toggling) are in `interviewer/`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/83-feature-flags test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`): `java/testkit`.
