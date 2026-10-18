# Follow-ups — 02 Elevator System

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Priority and capacity
Express/VIP cars, freight elevators that skip some floors, a maximum load per car. How do these change dispatch and stop selection?

### V3 — Smarter dispatch
Minimize average wait time: pre-positioning idle cars, grouping hall calls, destination-dispatch (passengers enter their *destination* at the hall). Make the dispatch policy swappable and *measure* it with your own simulation.

### V4 — Faults and maintenance
An elevator can go out of service (its pending requests must be redistributed), or be locked to certain floors by key.

### V5 — Real time
Doors take 3 ticks to cycle, movement takes 2 ticks per floor, and elevators run on their own threads. What is the unit of coordination now?
