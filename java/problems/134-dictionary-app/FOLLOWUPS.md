# Follow-ups — 134 Dictionary App

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Pronunciations and examples
Each meaning can carry a pronunciation (IPA string) and a list of example sentences, and `lookup` should be able to show them. What changes in your model and in the duplicate rule? Does `search` look into examples, and if so, how do callers choose?

### V3 — Bulk import
Load 200,000 entries from a text file (`word<TAB>POS<TAB>meaning` per line) in one call. Bad lines must be reported, not abort the import, and readers must not see a half-imported file. How do you keep readers unblocked while you load, and what does "atomic" mean for a partial failure?

### V4 — Fuzzy ranking by frequency
`suggest` should return the best candidates first: smaller edit distance, then higher lookup frequency, then alphabetical, and it should find words at distance up to 2 without scanning the whole dictionary. How do you count lookups without making every read a write bottleneck, and how do you find near words quickly?

### V5 — Favourites per user
Users keep a personal list of favourite words, in the order they were saved. What happens to a favourite when the word is removed from the dictionary, and who owns that cleanup? Where does a per-user feature live so that the dictionary does not learn about users?

### V6 — Persistence format
Write the dictionary to disk and read it back, surviving a crash in the middle of a save. Choose a format, decide how to version it, and say what happens to words in a file that the current rules would reject.
