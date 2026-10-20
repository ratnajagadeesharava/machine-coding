# Interviewer notes — 92 Chat System

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Message sent exactly while the recipient is connecting/disconnecting: delivered exactly once, never lost.
- Direct chat id with names in reverse order (`bob` sends to `ann`).
- A group message when the sender is the only member: no recipients, status READ.
- Member joins after message 1, before message 2: sees only message 2.
- Member leaves between SENT and READ: aggregate status computed over remaining members.
- Admin tries to leave, a member removes someone else, a non-member removes themselves.
- `read` on a chat with nothing unread; `read` of an offline user (marks as delivered too).
- 200-member group with 10 senders: unread counts equal total messages not sent by the member.

## Likely interviewer questions
- Where does the total order of a chat come from? What is the critical section when a message is sent?
- How do you avoid a race between 'is the recipient online?' and `connect`?
- What do you store per message and what per recipient? Why derive status instead of storing it?
- How does a newly added member not see old messages? And a member who left?
- What does `unreadCount` mean for a user who left the group?

## Invariants
- Every message has a unique id and a unique position in its chat's order.
- A message is delivered to each of its recipients at most once and, once they connect, exactly once.
- read implies delivered; once read it never goes back to unread.
- The recipient set of a message is fixed when it is sent and never contains the sender.

## Common mistakes
- Deciding delivery on an `online` flag read outside the lock, so a message sent during `connect` waits forever.
- Using the current member list to compute history or status (old messages appear for new members).
- Counting the sender as a recipient (unread 1 for the sender; status stays SENT).
- Storing a single `read` flag on the message instead of per recipient.
- Sorting by timestamps instead of using a sequence number; equal timestamps give different orders per member.
- Direct chat id depending on who sent first.

## Over-engineering traps
- A complete pub-sub/WebSocket gateway abstraction; the contract is synchronous.
- Separate classes for DirectChat, GroupChat, Channel with deep inheritance when a chat with a member policy suffices.
- A message-queue broker simulation for offline delivery - a per-user pending list is enough.

## Design and scaling discussion
- Fan-out on write gives O(1) reads and O(n) sends; fan-out on read keeps one log per chat and makes unread counts cursor arithmetic. Large groups favour the log with cursors.
- Ordering: a single sequencer per chat (partitioned by chat id) gives total order; a distributed system uses a per-chat leader or a consensus-backed sequence.
- Delivery guarantees: at-least-once from server to device with client-side dedup by message id yields effectively exactly-once.
- Storage: messages in an append-only log (Cassandra/Kafka-like) keyed by chat, user inbox as cursors; receipts are compacted by batching.

## Trade-off questions to ask when the candidate is done
- Fan-out on write versus on read; which for a 200-member group and which for a 2-person chat?
- One lock for the service versus per chat; what must still be global (message ids, users)?
- Computing status on demand versus maintaining counters per message; what about groups of 100k?
- What should 'delivered' mean for a user with several devices?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
