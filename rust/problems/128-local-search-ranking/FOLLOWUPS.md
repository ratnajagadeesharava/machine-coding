# Follow-ups — 128 Local Search with Ranking

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Make 100,000 places and thousands of searches per second comfortable
A user in a city of 50,000 places only cares about the ones within a few hundred grid steps. Make a search cost roughly proportional to the places near the user rather than to the whole catalogue, and keep updates and removals correct (a place that moves must leave its old neighbourhood). How do you prove the faster version returns exactly what the simple one did, and what happens with a huge radius or a very sparse map?

### V3 — Personalised boosts and sponsored results
A user has favourite categories (+30 points) and a list of places they have visited (-20 points); advertisers can buy a sponsored slot so that at most one sponsored place appears at position 1 if it passes all filters. Which part of the score belongs to the place, which to the user, and which to the campaign? How do you keep the ranking explainable ("why is this first?") and the tie-breaks deterministic?

### V4 — Pagination with stable cursors
Return results in pages of `n` with an opaque cursor for the next page. Places can be added, updated and removed between two page requests. Define what "stable" means (no duplicates, no skipped place that was present throughout), what the cursor contains, and what happens when the place the cursor points at has been removed or moved.

### V5 — Opening hours
Each place has weekly opening hours (several intervals per day, some past midnight) and a search can ask for places "open now" or "open at 21:30 on Friday". Time must come from an injected clock. How do hours interact with ranking (closing soon?) and with the spatial and text filters, and how do you test it without sleeping?

### V6 — Real coordinates
Replace the integer grid with latitude/longitude and a distance in metres. What changes in the distance computation, the radius test, the score (which must stay an integer) and any spatial index around the poles and the date line?
