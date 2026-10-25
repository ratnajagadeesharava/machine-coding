# Follow-ups — 131 Shortest Distance and Nearest Landmarks

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Coordinates and a faster point-to-point query
Every location now has `(x, y)` coordinates and a road is never shorter than the straight-line distance between its ends. Answer `shortestDistance` while visiting far fewer locations on a big map, and prove the answer is still exact. What do you need to know about the data to be allowed to do this, and what do you do when a road violates it?

### V3 — Roads closed at certain times
A road can be closed during daily time windows (for example 22:00-06:00), and `shortestDistance(a, b, atMinuteOfDay)` must route only over roads open at that moment. Which parts of your model change, how is time injected so tests stay deterministic, and does a route that spans several hours use the clock at departure only?

### V4 — Multi-stop routes
`routeVia(start, List<stops>)` visits every stop (in any order) and returns the stops in a good visiting order with the total distance, using a simple greedy rule (always go to the nearest unvisited stop). How many shortest-path computations does that take for k stops, and what would you change to make it cheaper? Define tie-breaks and what happens when a stop is unreachable.

### V5 — Caching with invalidation
A popular `(a, b)` pair is asked thousands of times a minute while roads change rarely. Add a cache that never returns a stale answer. What exactly does an update invalidate, can you invalidate less than everything, and how do concurrent queries interact with an invalidation in flight?

### V6 — One-way roads
Roads can be one-way or two-way, `removeRoad(a, b)` now needs a precise meaning for a mixed set of roads, and `shortestDistance(a, b)` is no longer symmetric. What happens to your path tie-break and to "reachable" in `nearestLandmarks`? Which of your data structures assumed symmetry?
