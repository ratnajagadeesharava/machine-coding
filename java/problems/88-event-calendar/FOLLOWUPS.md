# Follow-ups — 88 Shared Event Calendar

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Reschedule
Add `move(user, eventId, newStart, newEnd)` that only the organizer can call. It must be atomic: if the new time conflicts for anybody, the event stays where it was. What do you lock, and can a reader ever see the event in both places or in neither?

### V3 — Accept / tentative
Invitations start as pending and only an acceptance blocks the attendee's time. Pending invitations may overlap each other, an acceptance that would overlap an accepted event is refused. How does your conflict index change, and what does `agenda` show?

### V4 — Working hours and rooms
Each user has working hours and meetings may also reserve a meeting room (a resource that behaves like a participant). `commonFreeSlots` must respect working hours and optionally find a room. How do you generalise 'participant' without special cases leaking everywhere?

### V5 — Recurring series as a first-class thing
`cancel` should be able to cancel a single occurrence, 'this and following', or the whole series. Do you store occurrences or a rule? What does the conflict check do for a rule with no end date?

### V6 — Scale-out
Millions of users, shard by user. A meeting with participants on different shards needs a cross-shard atomic booking. Compare two-phase commit, a per-meeting coordinator with reservations that expire, and optimistic booking with compensation.
