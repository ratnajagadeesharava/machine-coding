# Follow-ups — 125 Music Streaming and Search

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Shuffle
`shuffle(userId, playlistId, seed)` returns the playlist's songs in a pseudo-random order that is fully determined by the seed (the same seed gives the same order, every run, on every machine). Which random generator do you use, and how do you keep the result stable when the playlist is edited between calls? How would a "play shuffled" feature feed the queue?

### V3 — Collaborative playlists
The owner can invite other users as editors. Editors can add and remove songs, only the owner can delete or invite, and the playlist shows who added each song. Where does ownership/permission live, and what changes in the error behaviour of the ownership rule from the base problem?

### V4 — Follow artists
Users follow artists. A new `feed(userId, k)` lists unplayed songs of followed artists, and recommendations also boost followed artists. How do you avoid recomputing artist counts for every request? What is the cost of a `follow` on the read paths?

### V5 — Premium-only songs and offline downloads
Some songs are premium-only; free users cannot play, queue or search-hit them (decide which), premium users can. Premium users may download up to N songs for offline play; downloaded songs stay playable for them, and a downgrade removes access but keeps the files marked. How does the entitlement check fit the play path without scattering `if premium` checks, and how do you keep the play counters correct for offline plays that are reported later?

### V6 — Play events
Other components (a billing/royalty counter, a "trending now" ranking, an analytics export) want to react to every play without `play` knowing about them. How do you add them, what happens when one of them fails or is slow, and what ordering guarantees do they get?
