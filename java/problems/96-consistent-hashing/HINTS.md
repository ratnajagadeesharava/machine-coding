# Hints — 96 Consistent Hash Ring (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

If you picture all hash values on a circle, what is a node's claim on the circle, and which node does a key belong to?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Place many points per node on the circle (virtual nodes) so each node owns many small arcs, which evens out the load. Keep the points in sorted order and find a key's owner with a binary search for the first point at or after the key's hash, wrapping around at the end. Use a hash with good dispersion whose value you control, and derive the points from the node's name and index only.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Store the ring as a sorted array of (hash, node) points. `nodeFor` is one binary search; `nodesFor` keeps walking clockwise, skipping points whose node is already in the result, until it has n distinct nodes or has gone round once. Weights just multiply the number of points. For concurrency, never mutate the array readers are using: build the new ring on the side and swap it in (or guard with a read-write lock). Decide what you do about the rare hash collision between two points.

**Java:** Do not use `String.hashCode()` or `Object.hashCode()` for ring positions; implement or import a real mixing hash (murmur3, FNV-1a with a final mix) over UTF-8 bytes. `TreeMap.ceilingEntry` or a sorted `long[]` plus `Arrays.binarySearch` both work.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
