# Follow-ups — 127 Browser History Manager

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Reopen closed tabs
`reopenLastClosedTab()` brings back the most recently closed tab with its whole back/forward trail and its position in it, under a new id or the old one (decide, and say why). Closed tabs are remembered up to a limit of 10; reopening twice restores the two most recent closes in order. What state must a closed tab keep, and what happens to its id promise ("never reused")?

### V3 — Bookmarks with folders
Users bookmark a url under a folder path such as `Work/Reading`. Folders nest, the same url may live in several folders, deleting a folder deletes its contents, and `listBookmarks(folder)` / `searchBookmarks(text)` exist. How does the folder tree relate to the history log (if at all), and which operations must stay O(depth)?

### V4 — Incognito tabs
`openTab(incognito)` creates a tab whose visits are never written to the global history (but whose back/forward trail works normally). Closing the last incognito tab leaves no trace anywhere. Where does the decision live: in the tab, in the recording step, or in a policy? What does `search` return while an incognito tab is open?

### V5 — Session persistence
Snapshot the whole browser (tabs with trails and positions, global history with timestamps) into a string, and restore it into a fresh instance after a "crash". Define the format and version it. What happens to tab ids and the id counter, and to history entries beyond a smaller capacity at restore time?

### V6 — Frecency ranking
Replace `mostVisited` with a frecency score: each visit contributes a weight that decays with age (for example 100 for the last hour, 70 for the last day, 40 for the last week, 10 older) and the score is the sum. Ranking must be stable and cheap to recompute as time passes (the clock is injected). Which parts of your history structure can answer "top k by score" without scanning everything, and how do you keep it correct under delete and capacity eviction?
