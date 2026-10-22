# Follow-ups — 109 Document Line Search

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Phrase search
A query in double quotes (`"quick brown" fox`) must match the words in that exact order, next to each other, in addition to the plain words. What extra information does your index need per word, and how do you verify a phrase after you have narrowed the candidate lines?

### V3 — Prefix search
A trailing `*` (`cat*`) matches every word that starts with `cat`. Keep the whole-word behaviour for terms without the star. How do you find all words with a prefix without scanning every word, and how do you combine the postings of many matching words without blowing up the cost?

### V4 — Add a line
`addLine(text)` appends a line and returns its new id (one more than the largest id ever assigned, even if the last line was deleted). Concurrent searches must keep working. What happens to your immutable or sorted structures, and what do you pay per insert?

### V5 — Ranking
Return lines ordered by relevance instead of id: lines that contain the query words more often, or where the words are close together, come first; ties by id. Where is the score computed (at index time or at query time), and how do you return only the top K without scoring every candidate?

### V6 — Memory and persistence
The document has 50 million lines and does not fit in memory as `String` sets. How do you shrink the index (word ids, delta-encoded postings), and how do you make deletions durable and cheap (tombstones, compaction) without renumbering ids?
