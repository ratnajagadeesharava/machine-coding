# Interviewer notes — 121 Concurrent Bloom Filter

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, concurrency bug, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Decisions fixed by the contract (candidates often ask)
- Sizing is spelled out so tests are exact: `m = ceil(-n ln p / (ln 2)^2)`, `k = max(1, round(m/n * ln 2))` using the integer `m`. Using the real-valued `m` for `k` gives the same answer for every parameter set the tests use, but is not what the README says.
- `add` returns "the filter changed", not "the item is new". A new item can legitimately return `false`; the tests only assert `true` for new items in a roomy filter (`p = 1e-9`), where a collision is practically impossible.
- Items are not trimmed, not lower-cased, and have no length limit; blank means empty after trim and is rejected everywhere.
- `approximateSize` is `Long.MAX_VALUE` when every bit is set; `0` when none is. The ±1 tolerance in the test is for floating-point evaluation order only.
- Determinism is a behaviour: the tests compare two instances (different insertion order, concurrent vs sequential) bit count for bit count. They do not compare to a reference hash; any good hash passes.

## Hidden edge cases (see `interviewer/`)
- A saturated filter (`m = 2`): all probes true, `add` false, estimate is the maximum value.
- 16 threads add the same shuffled 6,000 items in different orders: final `setBits()` equals the sequential run.
- False positive rate for `(2000, 1%)`, `(5000, 5%)`, `(1000, 10%)`; size estimate within 5% at five fill levels; estimate never decreases while another thread reads it during adds.
- A 200,000-character item that differs only in the last character; NUL and newline inside items.
- In a roomy filter the number of `true` returns across threads equals the number of distinct items (no thread double-claims a flip for different items).
- Adds after `clear` while threads run.

## Likely interviewer questions
- Why can `add` and `mightContain` be lock-free here? What property of the data makes it safe (bits only go 0 to 1)?
- A reader runs between the first and last position of a writer's `add`. What can it see, and is that ever a false negative for an item whose `add` has *returned*?
- Why is `bits[w] |= mask` on a plain array wrong even when each thread writes different items?
- How do you get "did I flip it" without a second read? (the old value returned by the atomic OR)
- `setBits()` while others add: is the result a consistent snapshot? Does it need to be (it is monotone, so the estimate only needs to be a value that existed)?
- `clear()` concurrent with `add`: what are the possible outcomes, and what do you promise?
- What does the hash need to satisfy for the false positive bound to hold (independence of the k positions, uniformity)? What happens with `h1 + i*h2` when `h2` is even or `m` is a power of two?

## Invariants
- A bit is only ever set, except by `clear`; therefore a returned `add` can never be undone by another add.
- `m` and `k` are fixed at construction; `k >= 1`; `bitCount()` is exactly the documented value.
- Validation happens before any bit is touched; a rejected `addAll` leaves no partial additions.
- `add` returns `true` only if this call itself flipped a bit.
- The positions of an item are a pure function of (item text, m, k).

## Common mistakes
- Plain `long[]` with `|=`: lost updates; caught by the disjoint-ranges and sequential-equivalence tests.
- `volatile long[]` thinking it makes elements volatile.
- `String.hashCode()` / Rust `DefaultHasher::new()` with `RandomState` / `HashMap`'s hasher: low-quality or randomised per process; the first fails the false positive bound, the second makes two instances disagree.
- `(int) Math.ceil(...)` on a large `m`, `int` bit index overflow, `%` on a negative hash (negative index), `Math.abs(Integer.MIN_VALUE)`.
- `k = (int) (m/n*ln2)` (truncation) instead of rounding; `k = 0` for loose `p`.
- Integer division `m / n` before multiplying by `ln 2`.
- `approximateSize` dividing integers (`X / m` is 0), or `ln(0)` when saturated.
- Checking blank with `isEmpty()` instead of after trim; lower-casing the item.
- Returning "new item" from `add` by calling `mightContain` first (check-then-act race: two threads both say `true`... or both say `false`).
- `addAll` that stops half way on the first blank item.
- A global `synchronized` / `ReentrantLock` around everything: correct, slow, and the point of the exercise is missed (discuss, do not fail on correctness).

## Over-engineering traps
- A `HashFunction` strategy hierarchy, a pluggable `BitStore` interface, builders and factories before the single implementation works.
- Separate classes for sizing, hashing, storage and the facade, each with an interface, for 100 lines of logic.
- Reader/writer locks around a structure that does not need any.

## Design and scaling discussion
- Structures an interviewer hopes to hear: a packed bit array in `AtomicLongArray` / `Vec<AtomicU64>`; atomic OR (or CAS loop) per word; double hashing (Kirsch and Mitzenmacher) to derive `k` positions from two 64-bit hashes; a stable, seed-free hash such as a 64-bit FNV/murmur variant over UTF-8 bytes.
- Lock-free vs striped locks: atomic words give no lock at all, but `k` independent atomic operations per add are not one atomic step (an item is partly visible while being added); that is fine because no promise is made until `add` returns.
- False sharing and cache misses: `k` random positions means `k` cache misses; a blocked Bloom filter (all `k` bits in one 64-byte block) trades a slightly worse rate for one miss.
- Sizing math: optimum `k = (m/n) ln 2`; the rate with rounded `k` is slightly different from `p`, which is why the test allows 3x.
- Memory: about 9.6 bits per item at 1%; 14.4 at 0.1%.
- Persistence, merging and scaling: see the follow-ups; merging is a bitwise OR, so it is also a good place to discuss atomicity of bulk operations.

## Trade-off questions to ask when the candidate is done
- Would you give `size()` an exact answer? What would it cost (a counter of "changed" adds is wrong, why)?
- How would you test the concurrency without sleeping? (barrier start, compare to a sequential reference, publish-then-probe)
- What breaks if you change the hash function between releases and keep persisted filters (V5)?
- How would you size a filter when `n` is unknown (V4)?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
