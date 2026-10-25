# Follow-ups — 142 Restaurant Customer Tokens

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Table assignment by party size
The restaurant has tables of 2, 4, 6 and 12 seats. When a counter calls a dine-in token the system must also assign the smallest free table that fits the party, and a party that fits nothing right now must not block smaller parties behind it for ever. Which part of the serving order changes, how does this interact with the anti-starvation rule, and what happens to the table when the token becomes a no-show?

### V3 — Reservations
Customers can reserve a time slot in advance. A reservation holder who arrives within 10 minutes of the slot is issued a token that outranks walk-in DINE_IN customers but not VIPs. How do you model the extra priority source without a special case in every comparison, and how do you keep `positionInQueue` honest when a reservation's slot is still in the future?

### V4 — SMS notifications
Customers can register a phone number and get a message when they are 3 positions from the front, when called, and when their no-show is about to expire. Where does the notification logic live so the queue does not know about SMS, how do you avoid sending twice for the same event, and what happens when the position jumps by more than one (a promotion, a cancellation)?

### V5 — Multiple branches
Several branches run their own queues and day sequences, but VIP status and the average service time are tracked per customer chain-wide, and a customer may transfer a waiting token to another branch. What is shared, what is per branch, how are token ids kept unique across branches, and how do you move a token without losing its place or double-serving it?

### V6 — Priority aging curves
Replace the fixed "5 pass-overs" rule by a configurable aging curve: a token's effective priority grows with the time it has waited (linear, stepped, capped) and VIP can never be overtaken by aging alone. How do you keep `queueSnapshot()` and `positionInQueue` consistent when order depends on the clock, and what does the cost of `callNext` become with 10,000 waiting tokens?
