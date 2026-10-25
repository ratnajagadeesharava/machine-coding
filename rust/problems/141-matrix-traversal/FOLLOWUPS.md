# Follow-ups — 141 Matrix Traversal Strategies

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Skip predicate
`start(strategy, predicate)` returns a cursor that silently skips values (or cells) for which a caller-supplied predicate is true, for example negative numbers or cells on the diagonal. What do `remaining` and `pathSum` mean now, and can `remaining` stay cheap?

### V3 — Bidirectional cursors
Add `previous(cursor)` and `seek(cursor, index)`. Which of your strategies can move backwards in O(1)? What does a registered strategy need to provide for this to keep working, and what happens at the two ends?

### V4 — Sparse matrices
The source now has mostly zeros: it can enumerate its non-zero cells. Traversals of the zeros are pointless in some views. Which operations can use the sparsity, and how do you keep the visiting order identical to the dense one?

### V5 — Parallel chunks
Split a traversal into N contiguous chunks (index ranges) that N workers read in parallel and whose results are glued back in order. Where is the chunk boundary computed, what must be true about a custom strategy for this to be safe, and how do you verify that the chunks cover the sequence exactly?

### V6 — N dimensions
Generalise to a 3-D (or N-D) array with row-major, snake and layered-spiral orders. What in your design was specific to two dimensions, and what carried over unchanged?
