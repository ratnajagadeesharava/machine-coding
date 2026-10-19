# Follow-ups — 79 Parallel Merge Sort

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Parallel merge
The final merge is sequential and becomes the bottleneck (Amdahl). Split the merge itself using binary search for the split point.

### V3 — Generic and stable
Sort `T` with a comparator and keep it stable. What changes in the merge and in the base case?

### V4 — Work stealing vs a fixed pool
Compare your implementation with a work-stealing fork-join pool. Why can a plain fixed thread pool deadlock on recursive tasks?

### V5 — External sort
The data does not fit in memory: sort chunks in parallel, spill, k-way merge. Where does the parallelism go?
