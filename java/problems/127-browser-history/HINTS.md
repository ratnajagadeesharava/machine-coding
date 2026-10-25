# Hints — 127 Browser History Manager (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Draw one tab as a line of pages with a marker on the page being shown. What does Back do to the marker, what does a new visit do to the pages after the marker, and what does the global log need that the tab does not?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep the two kinds of state apart: each tab owns its trail (pages and a current position), and the manager owns a single log of visit entries in recording order. The log is cut at the oldest end when it is full and filtered in the middle by delete and clear, so choose a container for which both are cheap enough. For the ranking queries, decide whether to compute them from the log on demand or to maintain a per-url summary, and what each choice costs when entries are deleted or evicted.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Tab { pages, position }` in a map keyed by an ever-increasing id; `Entry { time, url, tabId }` in a queue with the newest at one end; a single lock protects both so a visit (trail update, append, eviction) is one atomic step. This is the memento/command territory only if you later need undo/reopen; do not add it now. `search` walks the log from the newest end and keeps the first sighting of each lower-cased match; `mostVisited` groups by url (count, max time) and sorts with a three-key comparator. If the log grows large and ranking becomes hot, a per-url counter map updated on append/evict/delete avoids the scan, at the price of keeping it consistent.

**Java:** an `ArrayList<String>` plus an index is enough for a trail, and `ArrayDeque<Entry>` gives O(1) eviction at the head; remember that `removeIf` and `ArrayList.remove(0)` are linear. One `ReentrantLock`/`synchronized` around the whole manager is a legitimate answer for this size; say why a finer lock would not help.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
