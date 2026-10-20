# Follow-ups — 91 Online Auction

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Proxy (maximum) bidding
A bidder gives a maximum and the system bids the minimum increment on their behalf up to it, like eBay. The visible highest bid is now not the highest maximum. How does the history/notification model change and what do you store?

### V3 — Minimum increment and buy-it-now
Add a configurable minimum increment (a bid must beat the current by at least it) and a buy-it-now price that ends the auction immediately. How do you make the ending atomic with concurrent bids?

### V4 — Real-time subscriptions
Watchers want push updates when the highest bid changes. Add subscribe/unsubscribe with callbacks. Where are callbacks invoked (inside or outside the critical section), and what if a callback is slow or throws?

### V5 — Proactive closing
An auction that ends should settle (charge the winner, mark the item sold) at the end time even if nobody calls anything. Use a scheduler driven by the injected clock. How do you avoid settling twice and how does an anti-snipe extension reschedule it?

### V6 — Hot auction at scale
A celebrity item gets 50,000 bids per second. A single lock per auction is the bottleneck. Discuss sharded ingestion with a sequencer, optimistic compare-and-set on the top bid, and what you accept to lose in strictness.
