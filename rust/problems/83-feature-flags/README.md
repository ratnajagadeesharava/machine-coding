# 83 · Feature Flag Service (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

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

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for blank/duplicate flag names, unknown flags, null or blank `userId`, rollout outside 0..100. `isEnabled` takes a `Map<String,String>`.
- Rust: `Result<_, FlagError>` with `UnknownFlag`, `AlreadyDefined`, `InvalidArgument`; `set_rollout` takes `u32`; attributes are `&[(&str, &str)]`.
- A flag with a rollout of 0 is off for all users who match no rule, even if its default is true; with 100 it is on for all of them even if the default is false.
- Rules override the rollout: a matching rule beats the bucket (rollout 0 plus a matching `result=true` rule is on).
- Killed beats everything, including a rule or a 100% rollout, and `setKilled(false)` restores the previous behaviour with nothing lost.
- Bucketing quality: with 20,000 users a 30% rollout must give between 5,400 and 6,600 enabled; two independent 50% flags must overlap in 4,400-5,600 of 20,000 users; growing 30% -> 60% over 10,000 users must give 5,600-6,400 and turn nobody off.
- Adding a rule to a flag does not change the rollout or default; adding a rule for an unknown flag is an error.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum FlagError {
    UnknownFlag,
    AlreadyDefined,
    /// blank name / blank user id / rollout outside 0..=100
    InvalidArgument,
}

pub trait FlagService: Send + Sync {
    /// Creates a boolean flag.
    fn define(&self, flag: &str, default_value: bool) -> Result<(), FlagError>;

    /// Appends a targeting rule: users whose attribute `attribute` equals `value` get `result`.
    /// Rules are evaluated in the order they were added; the first matching rule wins.
    fn add_rule(
        &self,
        flag: &str,
        attribute: &str,
        value: &str,
        result: bool,
    ) -> Result<(), FlagError>;

    /// Percentage rollout 0..=100 for users that match no rule. Once a rollout is set, users inside the
    /// percentage get true and everybody else false (the default value is ignored). Each user must land in the
    /// same bucket every time for the same flag, buckets of different flags are independent, and raising the
    /// percentage never turns anybody off.
    fn set_rollout(&self, flag: &str, percent: u32) -> Result<(), FlagError>;

    /// A killed flag is false for everybody, whatever its rules, rollout or default say. Reversible.
    fn set_killed(&self, flag: &str, killed: bool) -> Result<(), FlagError>;

    /// Evaluation order: killed -> first matching rule -> rollout (if set) -> default.
    fn is_enabled(
        &self,
        flag: &str,
        user_id: &str,
        attributes: &[(&str, &str)],
    ) -> Result<bool, FlagError>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn FlagService>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Defaults, and the error cases: unknown flag, duplicate define, blank name, null/blank user id, rollout -1 and 101.
- First-match-wins with duplicate and overlapping rules, no matching rule, and empty attributes.
- Rule over rollout, kill switch over everything (and its reversal), rollout 0/100 extremes over 2,000 users.
- Percentage accuracy (30% of 20,000), stability over repeated calls, monotonic growth 30% -> 60%, independence of two flags.
- Concurrency: 16 threads appending rules lose none; 8 threads defining 200 flags each while evaluating; a 100x repeat of 8 threads adding rules and setting the rollout.
- More edge cases (heavy evaluation during rule edits, kill switch under concurrent toggling) are in `interviewer/`.

## Language notes

`RwLock<HashMap<String, Arc<FlagConfig>>>` lets readers clone the `Arc` and release the lock before evaluating. Do not use `DefaultHasher` with a random seed for buckets (`RandomState` differs per process); write or import a fixed hash such as FNV-1a.

## Running

```bash
cargo test -p p83_feature_flags
STRICT=1 cargo test -p p83_feature_flags   # fail instead of skip while unimplemented
cargo test -p p83_feature_flags --features hidden   # interviewer scenarios
cargo play run 83              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.
