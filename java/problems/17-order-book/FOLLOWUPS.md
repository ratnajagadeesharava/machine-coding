# Follow-ups — 17 Stock Exchange Order Book

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Market orders and time-in-force
Market orders (no price, take whatever is there), immediate-or-cancel and fill-or-kill. What does a market order do on an empty side?

### V3 — Modify and iceberg
Amend quantity (does it keep time priority?) and price (it must lose it). Iceberg orders showing only a slice.

### V4 — Many instruments and depth snapshots
One book per symbol; top-N depth levels per side; trade subscribers.

### V5 — Stop orders and circuit breakers
Orders triggered by the last trade price; halting when the price moves too far too fast.
