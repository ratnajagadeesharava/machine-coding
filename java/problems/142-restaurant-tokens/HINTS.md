# Hints — 142 Restaurant Customer Tokens (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Write down the life of one token as a list of states and the operations that move it between them. Then write down, separately, the single rule that decides which waiting token comes first. Do your counters, position and snapshot all consult the same rule?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

The serving order is a priority queue whose key is not fixed: a token's rank depends on its priority class, its issue number, whether it was recalled, and an "aging" counter that changes when *other* tokens are called. Think about what is cheap to maintain (a per-token counter, an ordered structure) and what is cheap to recompute (sorting the few hundred waiting tokens on demand). State-wise, keep one owner for every token and make illegal transitions impossible to express.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: a `Token { number, priority, type, state, calledAt, noShowAt, passedOver, recalled, frontKey }` and a system object that owns all tokens behind one lock. The serving order is a comparator: promoted first (by number), then priority descending, then `frontKey` (a negative, decreasing value for recalled tokens, the number otherwise). `callNext` scans the order for the first supported type, marks it `CALLED`, and increments `passedOver` of the waiting tokens with a smaller number. Status transitions are a small state machine (State pattern or an enum with allowed-next sets). Averages need only a running sum and count.

**Java:** a single `synchronized`/`ReentrantLock` around operations is enough here; sort a `List<Token>` of waiting tokens with a `Comparator` for snapshot/position/call, or keep a `TreeSet` that you re-key when a counter changes (remove, mutate, re-insert).

**Rust:** `Mutex<State>` inside the system struct; tokens in a `Vec<Token>` or `HashMap<u64, Token>`; sort indices with `sort_by` and a comparator chain (`then`).

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
