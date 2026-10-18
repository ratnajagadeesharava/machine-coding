# Follow-ups — 12 Travel Itinerary Planner

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Multi-city routing
Activities carry cities; the planner proposes the cheapest or fastest way between consecutive activities in different cities using a graph of transport links (flights, trains) with schedules.

### V3 — Suggestions
"Where can I fit a 90-minute museum visit on day 2?" — list feasible time slots, ranked by wasted travel time.

### V4 — Collaboration
Several people edit the same trip concurrently; conflicting edits are detected and merged or rejected. Undo/redo per user.

### V5 — Optimization
Given unordered must-do activities with durations and opening hours, produce a feasible schedule that minimizes travel.
