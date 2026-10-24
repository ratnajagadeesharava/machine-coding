# Hints — 121 Concurrent Bloom Filter

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Split the problem in two: (a) turn an item into `k` positions in `[0, m)`, deterministically; (b) store bits so that
setting a bit from many threads can never lose another thread's bit. Which operation tells you whether *you* flipped a bit?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Store the bits packed into 64-bit words. An update to a word must be a single atomic read-modify-write that returns the
previous value; the previous value tells you whether the bit was already set. Because bits are only ever turned on
(until `clear`), no lock is needed for `add` or `mightContain`. For the positions, hash the item once or twice into
64-bit values (a fixed algorithm over the UTF-8 bytes, not the language's randomised or identity hash) and derive all `k`
positions from them.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `create` computes `m` and `k` with the documented formulas and allocates `ceil(m/64)` atomic words. A position
`i` lives in word `i / 64`, bit `i % 64`. `add` validates, computes the `k` positions, and for each does an atomic OR
on its word, remembering whether the old word already had the bit; the result is "any bit was newly set". `mightContain`
validates, computes the positions, and returns false at the first unset bit. `setBits` sums the population count of
every word; `approximateSize` applies the formula to that reading and handles `X = 0` and `X = m`. `addAll` validates
everything, then adds. `clear` zeroes the words.

**Rust:** `Vec<AtomicU64>` and `AtomicU64::fetch_or` (it returns the previous value) is the atomic OR.
Deriving `k` positions as `h1 + i * h2` from two 64-bit hashes ("double hashing") is enough; keep `h2` odd and reduce
modulo `m` as an unsigned value.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
