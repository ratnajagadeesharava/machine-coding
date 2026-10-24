# 121 · Concurrent Bloom Filter

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 60 min | Level 2 — the filter entry point and the tests are provided; the filter design behind them is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A signup form wants to say "that username is probably taken" without a database round trip, a crawler wants to skip
URLs it has already visited without keeping every URL in memory, a cache wants to avoid looking up keys that cannot
exist. All three can live with an occasional "yes" that is wrong, as long as a "no" is **always** right. You are
building a **Bloom filter** for strings: a small, fixed-size structure that remembers *roughly* which items it has
seen. It never forgets an item that was added (no false negatives), it sometimes claims to have seen an item that was
never added (a false positive), and the chance of that is something you choose when you create it. Many threads add
and ask at the same time.

## Problem

Build a filter that is created from two numbers: how many distinct items you expect (`n`) and the false positive
rate you can tolerate at that size (`p`). The filter derives its own size from them, using exactly the formulas below
so that the numbers are the same for every implementation. Callers then `add` strings, ask `mightContain`, ask for an
`approximateSize` (how many distinct items it believes it holds), and can `clear` it. Everything is safe to call from
many threads at once.

## Requirements

1. **Sizing.** `create(expectedItems n, falsePositiveRate p)` requires `n >= 1` and `0 < p < 1` (an out-of-range or
   NaN value is an error). The filter has
   - `m = ceil( -n * ln(p) / (ln 2)^2 )` bits (`bitCount()`), and
   - `k = max(1, round( m / n * ln 2 ))` bit positions per item (`hashCount()`), where `round` is to the nearest integer
     (halves up) and `m` is the already-rounded integer above.

   `ln` is the natural logarithm; compute in double precision.
2. **Items.** An item is a string. `null` or blank (empty after trimming whitespace) is an error in every operation
   that takes an item. Items are **case-sensitive** and used exactly as given: `"Apple"`, `"apple"` and `" apple "`
   are three different items. Any other text is valid (non-ASCII, control characters, very long).
3. **`add(item)`** puts the item in the filter and returns `true` if the filter **changed**, that is, if this call
   flipped at least one bit from 0 to 1, and `false` otherwise. Adding the same item again therefore returns
   `false`. A new item can also return `false` if all of its `k` positions were already set by other items; callers
   cannot tell the two cases apart and must not rely on `add` to count distinct items.
4. **`mightContain(item)`** returns `false` only if the item has definitely not been added since the last `clear`;
   `true` means "added, or a false positive". It never changes the filter.
5. **No false negatives, ever.** Once `add(x)` has returned, `mightContain(x)` is `true` for every thread, until
   `clear`. This holds while other threads add other items (or the same one) at the same moment.
6. **False positive rate.** When the filter holds `n` distinct items, at most `3 * p` of probes of items that were
   never added report `true` (measured over 100,000 probes).
7. **Deterministic.** Which bits an item sets depends only on the item text, `m` and `k`. It does not depend on a
   random seed chosen per process, on insertion order, on the thread, or on the filter instance. Two filters with the
   same parameters that were given the same set of items, in any order, by any number of threads, have exactly the
   same set bits (`setBits()`), the same `approximateSize()` and answer every `mightContain` identically.
8. **`setBits()`** is `X`, the number of bits currently set to 1. It never decreases except through `clear`, and it
   never changes on a duplicate `add` or on `mightContain`. One `add` sets between 1 and `k` bits.
9. **`approximateSize()`** estimates the number of distinct items added:
   `round( -(m / k) * ln( 1 - X / m ) )`, with `X = setBits()` (nearest integer, halves up; ±1 from floating point is
   tolerated). It is `0` when `X = 0`, and `Long.MAX_VALUE` when every bit is set (`X = m`), because the estimate is
   unbounded then.
10. **`addAll(items)`** adds the items in iteration order and returns how many of those adds returned `true`. All
    items are checked first: if the collection is `null` or any element is `null`/blank, it is an error and nothing
    is added.
11. **`clear()`** forgets everything: `setBits()` is 0 afterwards and the filter is as good as new (same `bitCount()`
    and `hashCount()`). Clearing an empty filter is fine.
12. Two filters never share state.

## Constraints

- Many threads call `add`, `addAll`, `mightContain` and the accessors concurrently. No bit set by a returned `add`
  may ever be lost.
- The bit array is fixed at creation; the filter never grows, and memory use does not depend on how many items were
  added.
- Tests never sleep; they use latches and barriers.

## Examples

```
create(1000, 0.01)
  m = ceil(-1000 * ln(0.01) / (ln 2)^2) = ceil(9585.058...) = 9586
  k = max(1, round(9586 / 1000 * ln 2)) = round(6.6445...) = 7
  bitCount() = 9586   hashCount() = 7   setBits() = 0   approximateSize() = 0
```

```
f = create(1000, 1e-9)                       // a very roomy filter: m = 43133, k = 30
f.add("apple")        -> true                // bits flipped
f.add("apple")        -> false               // nothing left to flip
f.mightContain("apple") -> true
f.mightContain("Apple") -> false             // case-sensitive (and, in this roomy filter, not a false positive)
f.add("  ")           -> error               // blank
f.addAll(["a","b","a"]) -> 2                 // the second "a" changed nothing
```

```
create(1000, 0.01), one item whose 7 positions are all different:
  X = 7      approximateSize = round(-(9586/7) * ln(1 - 7/9586))    = round(1.0004) = 1
  X = 4000   approximateSize = round(-(9586/7) * ln(1 - 4000/9586)) = round(739.55)  = 740
```

```
8 threads each add 12,500 different items to create(100000, 0.01), then every one of the 100,000 items is probed
-> every probe is true; the filter's setBits() equals that of a filter that was given the same items one by one.
```

### Worked traces

```
create(10, 0.9)   -> m = ceil(2.19) = 3, k = max(1, round(0.208)) = 1
create(1, 0.5)    -> m = 2, k = 1
create(100, 0.001) -> m = 1438, k = 10
create(0, 0.01) / create(10, 0.0) / create(10, 1.0) / create(10, NaN) -> error
16 threads add the same new item at once -> between 1 and k of them see true (each of them flipped a bit no other thread flipped)
after clear(): setBits() = 0, approximateSize() = 0, mightContain(anything) = false
```

### Edge cases

- `p` very close to 1 can make `m / n * ln 2` smaller than 0.5: `k` is still at least 1.
- A tiny filter (`m = 2`) saturates after a handful of adds: every probe is then `true` and `approximateSize()` is the
  maximum value.
- Rejected items (and rejected `addAll` batches) change nothing.
- `add` returning `false` for an item that was never added before is legal (all its bits happened to be set).
- Items with a non-BMP character, a NUL character, or 200,000 characters are ordinary items.

## Glossary

- **Bloom filter**: a fixed-size bit array plus a rule that maps each item to some bit positions.
- **Bit position**: an index in `[0, m)`; adding an item sets its `k` positions to 1, probing checks that all `k` are 1.
- **False positive**: `mightContain` is `true` for an item that was never added.
- **False negative**: `mightContain` is `false` for an item that was added. Not allowed.
- **Expected items (`n`)** and **false positive rate (`p`)**: the design point the filter is sized for.
- **Fill / set bits (`X`)**: how many of the `m` bits are 1.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for an invalid `expectedItems`/`falsePositiveRate` in the factory, for a `null` or blank item in `add`/`mightContain`, and for a `null` collection or a `null`/blank element in `addAll`.
- `bitCount()`, `hashCount()` and `setBits()` expose `m`, `k` and `X`; `approximateSize()` is computed from them as in requirement 9.
- Nothing is promised about which bit positions an item gets, only that they are deterministic (requirement 7).
- Tests that measure the false positive rate use items such as `item-<i>` (added) and `probe-<i>` (never added).

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/contract` - see the source). It is **not** your domain model — it is the doorway. Add your own types next to it and wire them up.

```java
public interface ConcurrentBloomFilter {
    boolean add(String item);
    int addAll(Collection<String> items);
    boolean mightContain(String item);
    long approximateSize();
    long setBits();
    long bitCount();
    int hashCount();
    void clear();
}
```

Your wiring point (`solution/Solution.java`):

```java
public static ConcurrentBloomFilter create(int expectedItems, double falsePositiveRate)
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. More scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- The sizing formulas for several parameter sets, rounding of `k`, `k >= 1`, invalid parameters.
- `add` return values, duplicates, case sensitivity, non-ASCII items, blank items, `addAll` counting and all-or-nothing validation.
- `approximateSize` against the formula, against the true distinct count, for a single item.
- `clear`, determinism across instances and insertion orders.
- No false negatives for 100,000 items; false positive rate within 3x for two configurations.
- Threads adding disjoint ranges and the same items (final bits equal a sequential run), a reader that checks every published item while a writer keeps adding, probes of stable items during heavy writes.
- `BehaviorTest` (36 tests) and `interviewer/hidden-tests` (8 scenarios).

## Running

```bash
cd java/problems/121-concurrent-bloom-filter
mvn test
STRICT=1 mvn test                 # fail instead of skip while unimplemented
mvn test -Phidden                 # interviewer scenarios
```

Until `Solution.create` is implemented every test is skipped, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.
