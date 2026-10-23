# 110 Runtime Data Source Access Layer (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

Business code in a service reads and updates key-value data (customer settings, session snapshots, whatever). Where that data physically lives differs per deployment and sometimes has to change while the service is running: a **database**, a remote **API**, or a local **file**. You are building the access layer: one stable set of operations that business code calls, while operations staff can switch, at runtime, which of the three sources those calls land on. Each source holds its own independent records and has small, precisely specified differences in what it accepts. Sources are simulated in memory; nothing touches a real database, network or disk.

## Problem

Callers use `get`, `put`, `remove` and `size` without knowing which source is active. `switchSource` changes the active source for all later calls; it never copies or merges records. Switching away and back shows the original records again. Other threads keep reading and writing while an operator switches the source, and every call must land entirely on one source, never half on one and half on another.

## Requirements

1. There are exactly three sources, named `DATABASE`, `API` and `FILE` (exact, upper case). The layer starts on `DATABASE`. Every source starts empty.
2. `get(key)` returns the value stored under the key in the current source, or nothing if there is none. `put(key, value)` stores or overwrites. `remove(key)` deletes and reports whether a record existed. `size()` is the number of records in the current source.
3. Records of different sources are independent: a record written while `API` is active is invisible while `DATABASE` is active, and vice versa. `switchSource` neither copies, moves nor deletes records; switching back shows exactly what that source held before.
4. `switchSource(name)` with a name that is not exactly one of the three (null, blank, wrong case, unknown) is an error and leaves the current source unchanged. Switching to the already current source is allowed and changes nothing. `currentSource()` returns the active name.
5. A blank key (null, empty or whitespace only) is an error for `get`, `put` and `remove`, in every source. A null value is an error for `put`; the empty string is a valid value.
6. Source differences, all observable through the same operations:
   - `DATABASE` is unrestricted: keys are used exactly as given (case-sensitive, whitespace matters) and values may have any length.
   - `API` rejects a `put` whose value is longer than 32 characters (exactly 32 is fine) and treats keys case-insensitively (`"Token"` and `"token"` are the same record; whitespace still matters).
   - `FILE` trims leading and trailing whitespace from every key before using it (`" a "` and `"a"` are the same record) and is case-sensitive (`"A"` and `"a"` are different records). Values are unrestricted.
7. A refused or invalid `put` changes nothing (no record created, no existing record altered, `size` unchanged). A value is never altered by the layer: what was stored is exactly what `get` returns.
8. Records already stored in a source stay as they are after switching: a 40-character value stored on `DATABASE` is still returned unchanged after switching to `API` and back (the `API` limit only applies to `put` while `API` is active).

## Constraints

- Many threads call `get`/`put`/`remove`/`size` while others call `switchSource`. The layer must be thread-safe.
- Each call is applied to exactly one source, the one that was current at some single instant during the call. No write may be lost or end up in two sources, and a read must never fail or see a mixture of two sources because a switch happened at the same time.
- Reads are the hot path: do not serialize them behind one global lock if you can avoid it (hidden: several threads read continuously while the source is switched thousands of times).
- Time is not involved in this problem; no clock, no sleeping.

## Examples

```
get("x")                      -> Optional.empty             (starts on DATABASE, empty)
put("user:1", "alice"); put("user:1", "bob")
get("user:1")                 -> Optional[bob]; size() -> 1  (put overwrites)

put("k", "db-value")          // on DATABASE
switchSource("API")
get("k")                      -> Optional.empty; size() -> 0 (API has its own records)
put("k", "api-value")
switchSource("DATABASE")
get("k")                      -> Optional[db-value]          (the original records are back)

switchSource("FILE")
put("  report ", "v1")
get("report")                 -> Optional[v1]                (FILE trims keys)
get("Report")                 -> Optional.empty              (FILE is case-sensitive)

switchSource("API")
put("k", "x".repeat(32))      -> ok
put("k", "x".repeat(33))      -> IllegalArgumentException; get("k") still returns the 32-char value
put("Token", "t"); get("token") -> Optional[t]               (API keys ignore case)

switchSource("SQL")           -> IllegalArgumentException; currentSource() unchanged
get(" ")                      -> IllegalArgumentException
```

### Edge cases

- The same key in two sources is two unrelated records; removing it in one source leaves the other alone.
- `size()` counts only the current source, and counts records after key normalization: on `FILE`, `put(" a ")` and `put("a")` make one record; on `API`, `put("Ab")` and `put("aB")` make one; on `DATABASE` all four of `"a"`, `"A"`, `" a"`, `"a "` are different records.
- A key that is only whitespace is blank on every source, including `FILE` where trimming would make it empty.
- `remove` of a missing key returns false and changes nothing; removing twice returns true then false.
- A 33-character value is refused on `API` even if it overwrites an existing key; the old value stays. The same value is accepted on `DATABASE` and `FILE`.
- Length counts characters; the tests only use ASCII.
- A failed `switchSource` (unknown name) must not disturb later calls: they still go to the previous source.

## Glossary

- **Source**: one of `DATABASE`, `API`, `FILE`; an independent set of records plus its own acceptance rules.
- **Current source**: the one every `get`/`put`/`remove`/`size` goes to right now.
- **Record**: a key and its value inside one source.
- **Blank key**: null (Java), empty, or only whitespace.
- **Switch**: changing the current source; records are never moved by it.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for a null/blank key, a null value, an API value longer than 32 characters, and an unknown/null/blank source name.
- The layer starts on `DATABASE`; source names are exact, upper case.
- `put`/`remove` apply only to the current source; `size` counts only the current source.
- API: values up to 32 characters, keys compared ignoring case. FILE: keys trimmed, case-sensitive. DATABASE: nothing normalized.
- A refused put changes nothing; stored values are returned unchanged.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface DataAccess {

    /** Value stored under {@code key} in the current source. IllegalArgumentException for a null/blank key. */
    Optional<String> get(String key);

    /**
     * Stores or overwrites a record in the current source only. IllegalArgumentException for a null/blank key, a null value,
     * or a value the current source refuses (see README); a refused put changes nothing.
     */
    void put(String key, String value);

    /** Removes the record from the current source; true if there was one. IllegalArgumentException for a null/blank key. */
    boolean remove(String key);

    /** Number of records in the current source. */
    int size();

    /**
     * Makes {@code sourceName} (exactly "DATABASE", "API" or "FILE") the current source for every later call.
     * IllegalArgumentException for any other name (null, blank, different case); the current source is then unchanged.
     */
    void switchSource(String sourceName);

    /** Name of the current source. Before any switch it is "DATABASE". */
    String currentSource();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static DataAccess create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Start state, put/get/overwrite/remove/size, missing keys, empty value.
- Blank keys on every operation and every source; null value; unknown source names and the unchanged current source.
- Independence of the three sources and restoration after switching back.
- The `API` 32-character limit and case-insensitive keys, the `FILE` trimming and case sensitivity, `DATABASE` verbatim keys.
- Concurrency: writers racing a switcher lose nothing; readers racing a switcher never fail.
- More edge cases (refused overwrite, stored data surviving a limit change of source, thousands of switches, concurrent conservation of records) are in `interviewer/`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/110-data-source-layer test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Concurrently`, `Candidate`): `java/testkit`.
