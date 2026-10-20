# Hints — 92 Chat System (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

For one message and one recipient, what are the facts you need to remember to answer both 'delivered?' and 'read?' - and who owns them?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Separate the immutable message (id, chat, sender, text, recipients at send time, sequence) from the mutable per-recipient state (delivered, read). A chat is an ordered log, and what a user sees is derived from the log plus the user's membership at send time. Ordering inside one chat needs a single point of sequencing.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Keep a global or per-chat sequence counter incremented inside a critical section together with appending the message and snapshotting the recipient set. Per user keep an inbox: pending (undelivered) messages and the per-chat unread set. `connect` moves pending to delivered; `read` marks the chat's unread messages. Status is computed on demand by scanning the message's recipients still relevant (members now), not stored on the message.

**Java:** One lock per chat (or one lock for the whole service first) so that sequencing, fan-out and the online check are atomic; keep `online` as a `volatile` or inside the lock to avoid losing a message between 'recipient is offline' and 'recipient connects'.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
