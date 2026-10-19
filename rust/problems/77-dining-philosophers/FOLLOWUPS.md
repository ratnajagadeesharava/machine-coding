# Follow-ups — 77 Dining Philosophers

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Fairness / no starvation guarantee
Prove (or enforce) that a philosopher whose neighbours are faster still eats within a bounded number of other meals. Which solutions are only 'probably fine'?

### V3 — Arbitrator vs ordering vs state machine
Implement a second solution (waiter, Chandy-Misra, or hungry/eating/thinking state with a condition). Compare concurrency, fairness and complexity.

### V4 — Timeouts
`dine(p, timeoutMs, eat)`: give up without holding any fork. What guarantees that a timed-out philosopher does not leave a fork locked?

### V5 — Thousand tables
Run 1,000 tables x 5 philosophers in one process. What dominates: lock cost, thread cost? How would a thread pool + async fork acquisition change the design?
