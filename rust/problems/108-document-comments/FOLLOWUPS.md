# Follow-ups — 108 Document Comment Service

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Reply threads
A comment can be a reply to an earlier comment of the same document, and replies can have replies. Reading a document returns the threads (a comment followed by its replies, depth first, each level in insertion order). What happens when the parent id is unknown, belongs to another document, or is itself a reply? Which of your structures has to change, and which stays?

### V3 — Edit and delete
The author of a comment (and only the author) can edit its text or delete it. A deleted comment with replies must not break the thread. Do ids ever get reused? What does a reader who fetched the list before the delete see, and how would you show "edited"?

### V4 — Filter by reviewer
`commentsBy(documentId, reviewerId)` and "all comments by this reviewer across every document". Keep reads fast with 100,000 comments. Which index do you add, who keeps it consistent with the main list, and what does it cost on every write?

### V5 — Resolve state
Each comment (or thread) can be resolved and reopened; readers can ask for open comments only, and a document reports how many are unresolved. Who may resolve, and what happens to a resolved thread when somebody replies to it? Where does the counter live so that it stays right under concurrent updates?
