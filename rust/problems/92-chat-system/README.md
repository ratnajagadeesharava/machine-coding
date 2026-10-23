# 92 · Chat System (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

You are building the messaging core of a chat app such as WhatsApp or Slack, without any network or UI: just the in-memory
service behind it. People send direct messages to one other person or post into group chats. If the recipient is online the
message arrives at once; if not, it waits and is delivered the moment they connect. The sender can ask for the status of a
message (sent, delivered, read), and for a group that status reflects *all* the people the message was addressed to. Groups
have an admin who manages membership, and someone who joins later must never see what was said before they joined. Many
threads use the service at once, yet everybody in a chat must see the same messages in the same order.

## Problem

Design the messaging core of a chat application. Users can talk one-to-one or in groups. A message is delivered right
away when the recipient is online and waits for them otherwise. The sender sees a status that progresses from sent to delivered
to read, and for a group this means all current recipients. Groups have an admin, people can join and leave, and a person who joins
later must not see what was said before. Everybody in a conversation must see the messages in the same order, even when many
people type at the same time.

## Requirements

1. `register(user)` creates a user. Users start offline. `connect(user)` goes online and everything waiting for the user is delivered now; `disconnect(user)` goes offline. Unknown users are an error.
2. `sendDirect(from, to, text)` returns a message id. It is delivered immediately if the recipient is online, otherwise it waits until the next `connect`. A direct chat has id `dm:<smaller>:<larger>` (names compared as strings).
3. `createGroup(groupId, admin, members)` makes the creator the admin and a member automatically and adds the others. A group chat has id `group:<groupId>`.
4. `addMember(actor, group, user)`: only the admin. `removeMember(actor, group, user)`: the admin may remove anybody, any member may remove themselves; the admin cannot leave.
5. `sendGroup(from, group, text)`: the sender must currently be a member. The message is addressed to every *other* current member at that moment.
6. `history(user, chatId)` returns the messages the user took part in (sent, or was a recipient of when it was sent) in send order, each `messageId|sender|text`.
7. `read(user, chatId)` marks every message addressed to the user in that chat as read (and delivered). `unreadCount(user, chatId)` counts the messages addressed to the user not yet read.
8. `statusOf(messageId)` is the status from the sender's point of view: `SENT` (some recipient has not got it yet), `DELIVERED` (all recipients have it) or `READ` (all have read it). A message with no recipients is `READ`; recipients who have since left the group are ignored.
9. Message statuses are only ever `SENT`, `DELIVERED` or `READ` (exact upper-case strings). "Delivered" means the recipient's device got the message (they were online when it was sent, or connected afterwards); "read" means they called `read` for that chat.
10. `create_group` with a group id that already exists is an error (`ChatError::AlreadyExists`). The admin is a member automatically and receives the group's messages like any other member except their own.

## Constraints

- Used from many threads: concurrent senders, readers and connect/disconnect toggles.
- Within one chat every participant sees one single global order of messages (the same list for all members), and each sender's own messages appear in the order they were sent.
- Every message is delivered exactly once per recipient, even when the recipient connects and disconnects repeatedly while messages arrive (hidden/behaviour test: 800 messages, one flapping recipient).
- A group of 200 members with 10 concurrent senders posting 1,000 messages must stay consistent (hidden test).
- Message ids are unique; their format is yours (the tests only split `history` entries at the first two `|`, so the text may contain `|`).

## Glossary

- **Recipient**: someone a message is addressed to: the other person in a direct chat, or every *other* member of the group at the moment the message is sent. The sender is never a recipient of their own message.
- **Online / offline**: a user is online between `connect` and `disconnect`. Registering does not connect.
- **Delivered**: the recipient has the message (sent while they were online, or they connected afterwards). Reading also counts as delivered.
- **Chat id**: the string naming a conversation: `dm:<smaller>:<larger>` for direct chats, `group:<group_id>` for groups. It is what `history`, `read` and `unread_count` take.
- **Admin**: the creator of a group; the only person who may add members or remove others, and who can never leave.

## Examples

**Example 1: group status aggregates over every recipient** (the test `should_aggregate_group_status_over_all_recipients`)

```
register ann, bob, cy
create_group("g", ann, [bob, cy])            // chat id "group:g"; ann = admin
connect(bob)                                // cy stays offline
m = send_group(ann, "g", "hello all")        // addressed to bob and cy
status_of(m)  -> SENT                        // cy has not got it yet
connect(cy)
status_of(m)  -> DELIVERED                   // both have it
read(bob, "group:g")
status_of(m)  -> DELIVERED                   // cy has not read it
read(cy, "group:g")
status_of(m)  -> READ                        // everybody addressed has read it
unread_count(ann, "group:g") -> 0            // the sender is not a recipient
```

**Example 2: direct chat, offline recipient**

```
register ann, bob;  (bob is offline)
m = send_direct(ann, bob, "hi")              // chat id "dm:ann:bob" (same id for bob -> ann)
status_of(m) -> SENT;   unread_count(bob, "dm:ann:bob") -> 1
connect(bob)
status_of(m) -> DELIVERED
read(bob, "dm:ann:bob")
status_of(m) -> READ;   unread_count(bob, "dm:ann:bob") -> 0
history(bob, "dm:ann:bob") -> ["<id>|ann|hi"]
```

**Example 3: joining and leaving a group** (the tests `should_show_new_members_only_messages_sent_after_they_joined` and `should_stop_receiving_after_leaving_but_keep_what_was_already_there`)

```
create_group("g", ann, [bob, cy]);  send_group(ann, "g", "one")
remove_member(cy, "g", cy)                   // cy leaves (any member may remove themselves)
send_group(ann, "g", "two")
history(bob, "group:g") -> texts: one, two
history(cy,  "group:g") -> texts: one       // keeps what she had, gets nothing new
unread_count(cy, "group:g") -> 1             // "one" is still unread
send_group(cy, "g", "x") -> Err(NotAllowed)   // no longer a member
add_member(ann, "g", dee); send_group(bob, "g", "after")
history(dee, "group:g") -> texts: after     // nothing from before she joined
```

**Example 4: who may do what**

```
add_member(bob, "g", cy)       by a non-admin    -> Err(NotAllowed)
remove_member(bob, "g", ann)   non-admin removes someone else -> Err(NotAllowed)
remove_member(ann, "g", ann)   the admin leaving  -> Err(NotAllowed)
send_group(cy, "g", "x")       cy is not a member -> Err(NotAllowed)
send_group(ann, "nope", "x")   unknown group      -> Err(Unknown)
```

## Edge cases

- A message sent to a group where the sender is the only member has no recipients: its status is `READ` straight away.
- `history` of a user who has nothing to do with the chat (never sent, never a recipient) is an empty list; so is `history` of a chat with no messages.
- A history entry is `messageId|sender|text`; the text may itself contain `|`, so the first two `|` are the separators.
- Message ids are unique across the whole service; their format is yours.
- A message sent while the recipient is offline stays `SENT` for the sender; disconnecting after having received it never "un-delivers" it.
- If a recipient leaves the group before reading, the status is computed from the remaining recipients only (see the behavioural contract below).
- Direct chat ids are symmetric: `send_direct(ann, bob, ..)` and `send_direct(bob, ann, ..)` land in the same chat `dm:ann:bob`; the comparison is plain string order (`"Bob" < "ann"`).

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `ChatApi`; `IllegalArgumentException` for unknown users, groups, messages; `SecurityException` for non-admin member management, a non-member sending to a group, the admin leaving. Rust: `ChatService` trait with `ChatError::{Unknown, NotAllowed, AlreadyExists}`; `connect`, `disconnect`, `read`, `history`, `unread_count`, `status_of` return `Chat<..>`.
- Direct chat id: `"dm:" + smaller + ":" + larger` by string comparison, so ann->bob and bob->ann share `dm:ann:bob`. `history` of a user who is not part of the chat is an empty list.
- Delivery state is per recipient: `delivered` is set on connect/online send or on `read`; `read` implies delivered. A `read` call marks everything addressed to the user in that chat as read in one go.
- The sender is never a recipient of their own message: `unreadCount` for the sender is 0 and a one-recipient message is `DELIVERED` once that recipient got it.
- A group message is addressed to the members at send time. A member added later does not see earlier messages in `history`; a member who left keeps what they already received (and their `unreadCount` for those), but gets nothing new and cannot send.
- Status ignores recipients who left the group: once the remaining members have read the message it is `READ`, even if the leaver never connected.
- `history` contains the messages the user sent and those addressed to them, sorted by the chat's send order. Two histories of different members of one group agree on the common messages.
- Unknown users in `sendDirect` (sender or receiver), `connect` of an unknown user and `statusOf` of an unknown id are errors. A group send to an unknown group is an error as well.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Chat ids: a direct chat between two users has the id `"dm:" + smaller_name + ":" + larger_name` (names compared
//! as strings); a group chat has the id `"group:" + group_id`.

#[derive(Debug, PartialEq, Eq)]
pub enum ChatError {
    /// unknown user, group or message
    Unknown,
    /// the actor is not allowed to do this (non-admin managing members, non-member sending, admin leaving)
    NotAllowed,
    /// group id already used
    AlreadyExists,
}

pub type Chat<T> = Result<T, ChatError>;

pub trait ChatService: Send + Sync {
    fn register(&self, user: &str);

    /// Goes online. Everything waiting for this user is delivered now.
    fn connect(&self, user: &str) -> Chat<()>;

    /// Goes offline; messages sent from now on wait until the next connect.
    fn disconnect(&self, user: &str) -> Chat<()>;

    /// Sends a direct message; returns the message id. It is delivered immediately if the recipient is online.
    fn send_direct(&self, from: &str, to: &str, text: &str) -> Chat<String>;

    /// The creator becomes the group's admin and is a member automatically; the other members are added.
    fn create_group(&self, group_id: &str, admin: &str, members: &[&str]) -> Chat<()>;

    /// Only the admin.
    fn add_member(&self, actor: &str, group_id: &str, user: &str) -> Chat<()>;

    /// The admin may remove anybody; any member may remove themselves (leave). The admin cannot leave.
    fn remove_member(&self, actor: &str, group_id: &str, user: &str) -> Chat<()>;

    /// Sender must currently be a member. Returns the message id.
    fn send_group(&self, from: &str, group_id: &str, text: &str) -> Chat<String>;

    /// The messages the user took part in (sent, or was a recipient of when they were sent) in this chat, in the
    /// order they were sent, formatted "messageId|sender|text".
    fn history(&self, user: &str, chat_id: &str) -> Chat<Vec<String>>;

    /// Marks every message addressed to the user in the chat as read (and delivered).
    fn read(&self, user: &str, chat_id: &str) -> Chat<()>;

    /// Messages addressed to the user in the chat that they have not read yet.
    fn unread_count(&self, user: &str, chat_id: &str) -> Chat<usize>;

    /// Status from the sender's point of view: "SENT" (somebody it is addressed to has not got it yet), "DELIVERED"
    /// (all recipients have it) or "READ" (all recipients have read it). A message nobody receives (a group of one)
    /// is "READ". Recipients who have left the group since are ignored.
    fn status_of(&self, message_id: &str) -> Chat<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn ChatService>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Offline recipient: SENT until connect, DELIVERED after; online recipient: DELIVERED immediately, unread 1, then READ after `read` with unread 0; disconnect stops delivery.
- Direct history in send order for both sides, separate chats separate, uninvolved user sees nothing.
- Group aggregate status with one online and one offline member through SENT, DELIVERED, READ.
- The sender is not a recipient (`unreadCount` 0, status DELIVERED after the single recipient got it).
- Permissions: non-member send, non-admin add, non-admin remove, admin leaving, unknown group; then a successful add and send.
- New members only see later messages; a leaving member keeps the earlier message, gets no later ones, cannot send, and still counts 1 unread.
- Status after a member leaves: ignored for aggregate (SENT becomes READ once the others read).
- Unknown users and message ids.
- Concurrency: 8 senders x 100 group messages (800 unique messages, identical order for all 4 members, per-sender order preserved); one thread flapping connect/disconnect while 4 senders send 800 direct messages (exactly 800 delivered once, unread 800); 4 senders over 100 runs (identical 80-message histories).
- More edge cases (200-member group, unread counts while reading and sending) are in `interviewer/`.

## Running

```bash
cargo test -p p92_chat_system
STRICT=1 cargo test -p p92_chat_system   # fail instead of skip while unimplemented
cargo test -p p92_chat_system --features hidden   # interviewer scenarios
cargo play run 92              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.
