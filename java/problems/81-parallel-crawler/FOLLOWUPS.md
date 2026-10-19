# Follow-ups — 81 Parallel Web Crawler

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Limits
Stop after `maxPages` pages or `maxDepth` levels, and only follow links on the same host. Which thread decides to stop and how do the others learn about it?

### V3 — Politeness
At most 2 concurrent requests per host and a minimum delay between requests to the same host. How does a per-host queue change the termination logic?

### V4 — Cancellation and timeouts
`crawl` can be cancelled or has a deadline; slow fetches are abandoned. What state do you leave behind?

### V5 — Streaming results / back-pressure
Return results as a stream while the crawl runs, with a bounded queue so a slow consumer slows the crawl down.
