# Follow-ups — 86 Document Versioning with Merge

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Branches
Allow a document to fork into named branches and to merge a branch back. A version now has parents, so history is a DAG. How do you find the common ancestor, and what changes in `history` and `latest`?

### V3 — Compact storage
Store each version as a delta against the previous one with a full snapshot every N versions. How do you read version 4,973 quickly, and how does the delta format interact with merge?

### V4 — Conflict resolution
Instead of rejecting, return the conflict regions to the author, who resolves them and resubmits. What does a conflict marker look like in the model, and how is a resolution validated against the version it was created for?

### V5 — Permissions and reviews
Edits by some users must be approved before they become `latest`. Where does an unapproved edit live, and what is its base when the document moves while it waits?

### V6 — Many documents
Store millions of documents and take locks per document only. How do you keep version numbering per document gapless without a global counter, and what does optimistic concurrency look like in a database (`UPDATE ... WHERE version = ?`)?
