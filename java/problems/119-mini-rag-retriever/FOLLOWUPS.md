# Follow-ups — 119 Mini RAG Retrieval Component

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Metadata filters
Each document is ingested with metadata (`lang=en`, `team=billing`, `year=2024`) and a query can restrict retrieval to chunks whose metadata match. Where does the filter run: before the nearest-neighbour search, after it, or inside the store? What happens to `k` when the filter removes most of the top results, and what does the vector store contract need to change?

### V3 — Re-ranking
Retrieve the top 50 by vector similarity, then re-order them with a second, more expensive scorer (a cross-encoder, a recency boost, a per-source weight) and keep the best `k`. How does the second stage plug in without the first stage knowing about it, and how do you keep the tie-breaking rule deterministic?

### V4 — Hybrid keyword + vector scoring
Combine the cosine score with a keyword score (term frequency / BM25-like) using a weight, or fuse the two ranked lists (reciprocal rank fusion). Where is the keyword index kept so that it stays in step with ingest, replace and delete? What does a query with a rare exact term (an error code) need that embeddings alone do not give?

### V5 — Streaming ingestion
Documents arrive as a stream of fragments (`append(docId, fragment)`) and become searchable as they arrive, in near real time. How do you chunk across fragment boundaries, what does re-ingesting a growing document cost, and what can a concurrent reader see while a document is half-streamed? Add a way to commit a document so that readers only ever see complete versions.
