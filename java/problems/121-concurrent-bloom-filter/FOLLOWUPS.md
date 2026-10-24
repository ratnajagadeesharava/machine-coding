# Follow-ups — 121 Concurrent Bloom Filter

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Union of two filters
`mergeFrom(other)` makes this filter contain everything either filter contains. It is only allowed between filters
created with identical parameters (anything else is an error, and nothing changes). What does "identical" have to
include so that the result is meaningful? What happens to `approximateSize()`, and is the merge safe while both
filters are being written to?

### V3 — Counting filter with removal
Add `remove(item)`. A plain bit cannot be un-set safely, so what do you store instead, how wide is each counter, and what
happens when a counter would overflow or underflow? What does `remove` of an item that was never added do to the
no-false-negatives guarantee, and what can you promise to the caller?

### V4 — Scalable filter
The caller does not know `n`. When the filter reaches its design capacity, start a new, larger and tighter layer so the
overall false positive rate stays under `p`. How do you tighten each layer, how does `mightContain` consult the layers,
how does `add` decide which layer to write to, and how does this interact with concurrent adds at the moment a layer fills up?

### V5 — Serialise to bytes
`toBytes()` and `fromBytes(...)` so a filter can be saved and loaded by another process (or another language). What
exactly has to be in the byte format so that the loaded filter answers every query the same way, and what does a
reader do with a version or length it does not understand? Which part of your hashing becomes a compatibility promise?

### V6 — Contention
Sixty-four threads hammer one filter. Compare one atomic word array with compare-and-set loops, a lock per stripe of
words, and a single lock. Where does each lose, how would you measure it, and what changes if most operations are probes?
