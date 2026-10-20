# Follow-ups — 92 Chat System

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Message pagination and search
`history(user, chat, afterMessageId, limit)` and keyword search over a user's messages. What index do you keep, and how does it behave for a member who joined mid-history?

### V3 — Edit, delete and reactions
Senders can edit or delete (for everyone) a message; recipients can react. History must show the latest text, status must not regress. How do you model message versions and keep the order stable?

### V4 — Typing indicators and presence
Add last-seen and 'typing...' signals that are not persisted. Where do you keep ephemeral state, how do you expire it, and why should it not share the lock with message ordering?

### V5 — Multi-device
A user is online on several devices; each device has its own delivery cursor and a message is 'delivered' when the first device gets it but 'read' when any reads it. Redesign the per-recipient state around cursors instead of flags.

### V6 — Scale-out
Millions of users, groups with 100k members. Compare fan-out on write (copy into each inbox) versus fan-out on read (one log per chat with per-user cursors). Where does ordering come from in a sharded system?
