# Follow-ups — 116 In-Memory Search Engine

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Phrase search
A query in double quotes, `"quick brown"`, matches only documents where the terms appear next to each other, in that order. It can be combined with plain terms (`"quick brown" fox`). What extra information must the index keep, what does it cost in memory, and how does deleting or updating a document touch it? How do you score a phrase match?

### V3 — Fields and boosts
Documents become `title` + `body`. A match in the title counts three times as much as a match in the body, and a query can be restricted to one field (`title:fox`). Which of tf, df and the document length are per field? Does the tokeniser or the scorer change, and what do existing callers see?

### V4 — Fuzzy match and snippets
A term followed by `~` also matches terms within edit distance 1 (`quik~` finds `quick`), and each result can carry a snippet: the best window of about 10 tokens around the matches with the matched words marked. How do you find the fuzzy candidates without comparing against every term in the vocabulary? Where does the original text live, and what does an update cost now?

### V5 — Scale and freshness
The collection grows to 50 million documents and does not fit on one machine, and writes arrive faster than readers should be blocked. How do you shard it, and what changes about idf (global or per shard)? How do you make a new document searchable within a second without rebuilding the index, and how are deletes merged in later? What do you cache, and how do you invalidate it?
