# Machine Coding + LLD Interview Playground

Senior-level (about 6 years of experience) practice for **machine-coding and low-level-design interviews**, in **Java 21** and **Rust**.
105 problems, each with requirements, a behavioural contract, runnable tests, hints, follow-ups and a scoring rubric.

> **Give me the requirements. Give me the tests. Give me the playground. Do NOT give me the architecture.**

You get what a senior engineer is handed at the start of a real interview:

```
Problem -> Requirements -> Constraints -> Examples -> Behavioural contract -> Test + build environment
```

You are responsible for everything in the middle:

```
Classes · Interfaces · Enums · Value objects · Services · Policies · State machines
Data structures · Ownership of state · Concurrency model · Implementation · Refactoring
```

There are no `ParkingLot.java` / `Piece.rs` skeletons with `TODO`s: `src/main` is empty except for one wiring file.

## Contents

- [Quick start](#quick-start)
- [How a problem works](#how-a-problem-works)
- [Learning tracks](#learning-tracks)
- [Track 1 - Foundations, principles and patterns (1-73)](#track-1---foundations-principles-and-patterns-1-73)
- [Track 2 - Concurrency](#concurrency)
- [Track 3 - Classic product systems](#classic-product-systems)
- [Track 4 - Distributed systems](#distributed-systems-in-process-simulations)
- [Full catalogue](#full-catalogue)
- [The `play` runner](#the-play-runner-rustlings-style)
- [Where to write your solutions](#where-to-write-your-solutions)

## Quick start

Requirements: JDK 21 + Maven (Java side) and a stable Rust toolchain (Rust side).

```bash
cargo play list                  # every problem and your progress (needs only the Rust toolchain)
cargo play next                  # first problem that is not passing yet
cargo play info 82               # print the README of problem 82
cargo play run 82                # run its tests (Rust);  add  -l java  for the Java side
cargo play run 82 --hidden       # also run the interviewer scenarios
```

Or directly: `cd java/problems/82-url-shortener && mvn test` / `cd rust/problems/82-url-shortener && cargo test`.
Pick a problem, read its `README.md`, start the timer from `INTERVIEW.md`, and **stare at the requirements before typing**.

> **Do not write solutions on `main`.** Create a branch for your solutions (`git checkout -b my-solutions`), see [below](#where-to-write-your-solutions).

## How a problem works

```
<problem>/
├── README.md              requirements, constraints, examples, behavioral contract, test entry point
├── INTERVIEW.md           time limit, phases, rules, self-review questions
├── FOLLOWUPS.md           V2…V5 requirements — open one at a time
├── RUBRIC.md              how to score yourself / a candidate
├── INTERVIEWER_NOTES.md   hidden edge cases, traps, discussion guide   ← don't read while practicing
├── interviewer/           extra scenarios (not in the default test run)
├── src/ (and tests/)      your design + the behavioral tests
└── pom.xml | Cargo.toml
```

### Architecture-agnostic tests

Tests talk to your system only through a **thin, primitive-typed entry point** (strings, numbers, lists,
`Optional`/`Option`). That entry point is a doorway, not a design:

* Java — `contract/*Api.java` (provided) and `solution/Solution.java` (you wire your own classes in)
* Rust — `src/contract.rs` (provided) and `create(...)` in `src/lib.rs` (you wire your own types in)

Choose any model you like behind it: inheritance, composition, enums, traits, data-oriented structs, no
interfaces at all. Test names describe behavior (`shouldReleaseSpotAfterVehicleExits`), never classes.

For some problems the entry point itself is the problem (e.g. `Cache<K,V>`, a blocking queue). Those are marked
**Level 2**; the shared test infrastructure (fake clock, thread coordination helpers) for concurrency-heavy ones is
**Level 3**. Everything else is **Level 1** — almost empty.

### Fresh checkout builds green

Until you wire your solution, behavioral tests are **skipped** (Java) / print `SKIPPED` and return (Rust), so
`mvn test` and `cargo test` pass on a clean repo. Use `STRICT=1` to make "not implemented" a failure:

```bash
STRICT=1 mvn test          # Java
STRICT=1 cargo test        # Rust  (PowerShell: $env:STRICT=1; cargo test)
```

### Hidden scenarios

Visible tests teach you the contract; they do not enumerate every edge case. Each problem also ships
`interviewer/` scenarios that use the same contract: `mvn test -Phidden` / `cargo test --features hidden`.

## Learning tracks

Each track is ordered by **what you need to know first**, not by difficulty (difficulty is shown for reference and goes up and down on purpose).
Tick the problems off as you go; you are done with a problem when the tests pass **and** you can explain your design and the follow-ups out loud.

| Track | Problems | Focus | Start with |
|---|---|---|---|
| [1. Foundations](#track-1---foundations-principles-and-patterns-1-73) | 01-73 | OOP, design principles, SOLID, design patterns, classic LLD designs (parking lot, elevator, chess ...) | [Classes and Objects](java/problems/33-classes-objects/README.md) |
| [2. Concurrency](#concurrency) | 74-81 (+ 13, 20, 22, 24, 27-31) | locks, deadlocks, wait/notify, thread pools, parallel algorithms | [Ordered Printer](java/problems/76-ordered-printer/README.md) |
| [3. Classic product systems](#classic-product-systems) | 82-95 (+ 11, 15) | URL shortener, ticketing, food delivery, file system, text editor, SQL engine ... | [URL Shortener](java/problems/82-url-shortener/README.md) |
| [4. Distributed systems](#distributed-systems-in-process-simulations) | 96-105 | consistent hashing, idempotency, leases, gossip, CRDTs, quorums, Raft, 2PC, sagas | [Consistent Hashing](java/problems/96-consistent-hashing/README.md) |

Suggested overall order: Track 1 first, then 2 and 3 in either order, then 4 (it assumes you are comfortable with concurrency and state machines).

## Track 1 - Foundations, principles and patterns (1-73)

The first 73 problems are sequenced so each builds on the ones before it: fundamentals, principles, patterns, open-ended design, and finally the concurrency problems
that also appear in Track 2. Work top to bottom. The *Step* column is the suggested sequence; `#` is the folder number (`cargo play run <#>`).
Problems 33-73 are small concept drills (exact API, stubs that make the tests compile, tests that check the *concept*); 1-32 are open designs.

#### Phase 1 - OOP fundamentals

Objects, contracts and how objects relate.

| Step | Problem | Difficulty | Time | Scaffold | # |
|---|---|---|---:|:---:|---:|
| Step 1 | [Classes and Objects](java/problems/33-classes-objects/README.md) | Easy | 30 min | L2 | 33 |
| Step 2 | [Interfaces](java/problems/34-interfaces/README.md) | Easy | 30 min | L2 | 34 |
| Step 3 | [Inheritance](java/problems/35-inheritance/README.md) | Easy | 30 min | L2 | 35 |
| Step 4 | [Polymorphism](java/problems/36-polymorphism/README.md) | Easy | 30 min | L2 | 36 |
| Step 5 | [Abstraction](java/problems/37-abstraction/README.md) | Medium | 30 min | L2 | 37 |
| Step 6 | [Encapsulation](java/problems/38-encapsulation/README.md) | Medium | 30 min | L2 | 38 |
| Step 7 | [Aggregation](java/problems/39-aggregation/README.md) | Easy | 30 min | L2 | 39 |
| Step 8 | [Composition](java/problems/40-composition/README.md) | Medium | 40 min | L2 | 40 |
| Step 9 | [Association](java/problems/41-association/README.md) | Medium | 40 min | L2 | 41 |

#### Phase 2 - Design principles

Habits that keep designs small and clean.

| Step | Problem | Difficulty | Time | Scaffold | # |
|---|---|---|---:|:---:|---:|
| Step 10 | [KISS](java/problems/43-kiss/README.md) | Easy | 20 min | L2 | 43 |
| Step 11 | [YAGNI](java/problems/44-yagni/README.md) | Easy | 25 min | L2 | 44 |
| Step 12 | [DRY](java/problems/42-dry/README.md) | Medium | 30 min | L2 | 42 |
| Step 13 | [Law of Demeter](java/problems/45-law-of-demeter/README.md) | Medium | 40 min | L2 | 45 |

#### Phase 3 - SOLID

Applies the habits to class design; finish with the combined exercise.

| Step | Problem | Difficulty | Time | Scaffold | # |
|---|---|---|---:|:---:|---:|
| Step 14 | [Single Responsibility](java/problems/46-srp/README.md) | Medium | 40 min | L2 | 46 |
| Step 15 | [Open/Closed](java/problems/47-ocp/README.md) | Medium | 35 min | L2 | 47 |
| Step 16 | [Liskov Substitution](java/problems/48-lsp/README.md) | Medium | 40 min | L2 | 48 |
| Step 17 | [Interface Segregation](java/problems/49-isp/README.md) | Medium | 35 min | L2 | 49 |
| Step 18 | [Dependency Inversion](java/problems/50-dip/README.md) | Medium | 35 min | L2 | 50 |
| Step 19 | [SOLID Summary](java/problems/51-solid-summary/README.md) | Hard | 75 min | L2 | 51 |

#### Phase 4 - Creational patterns

How objects get made.

| Step | Problem | Difficulty | Time | Scaffold | # |
|---|---|---|---:|:---:|---:|
| Step 20 | [Singleton](java/problems/52-singleton/README.md) | Medium | 30 min | L2 | 52 |
| Step 21 | [Factory Method](java/problems/53-factory-method/README.md) | Medium | 35 min | L2 | 53 |
| Step 22 | [Abstract Factory](java/problems/54-abstract-factory/README.md) | Medium | 45 min | L2 | 54 |
| Step 23 | [Builder](java/problems/55-builder/README.md) | Medium | 45 min | L2 | 55 |
| Step 24 | [Prototype](java/problems/56-prototype/README.md) | Medium | 40 min | L2 | 56 |

#### Phase 5 - Structural patterns

How objects are wired together. Bridge comes last since it builds on Adapter/Composite thinking.

| Step | Problem | Difficulty | Time | Scaffold | # |
|---|---|---|---:|:---:|---:|
| Step 25 | [Adapter](java/problems/57-adapter/README.md) | Medium | 35 min | L2 | 57 |
| Step 26 | [Facade](java/problems/58-facade/README.md) | Medium | 35 min | L2 | 58 |
| Step 27 | [Decorator](java/problems/59-decorator/README.md) | Medium | 40 min | L2 | 59 |
| Step 28 | [Composite](java/problems/60-composite/README.md) | Medium | 40 min | L2 | 60 |
| Step 29 | [Proxy](java/problems/61-proxy/README.md) | Medium | 45 min | L2 | 61 |
| Step 30 | [Flyweight](java/problems/63-flyweight/README.md) | Medium | 40 min | L2 | 63 |
| Step 31 | [Bridge](java/problems/62-bridge/README.md) | Hard | 50 min | L2 | 62 |

#### Phase 6 - Behavioural patterns

How objects collaborate. Start with the everyday ones (Strategy, Observer); Visitor last.

| Step | Problem | Difficulty | Time | Scaffold | # |
|---|---|---|---:|:---:|---:|
| Step 32 | [Strategy](java/problems/66-strategy/README.md) | Medium | 40 min | L2 | 66 |
| Step 33 | [Observer](java/problems/65-observer/README.md) | Medium | 40 min | L2 | 65 |
| Step 34 | [Template Method](java/problems/69-template-method/README.md) | Medium | 40 min | L2 | 69 |
| Step 35 | [Command](java/problems/67-command/README.md) | Hard | 50 min | L2 | 67 |
| Step 36 | [State](java/problems/68-state/README.md) | Hard | 60 min | L2 | 68 |
| Step 37 | [Chain of Responsibility](java/problems/73-chain-of-responsibility/README.md) | Medium | 45 min | L2 | 73 |
| Step 38 | [Iterator](java/problems/64-iterator/README.md) | Medium | 45 min | L2 | 64 |
| Step 39 | [Memento](java/problems/72-memento/README.md) | Hard | 50 min | L2 | 72 |
| Step 40 | [Mediator](java/problems/71-mediator/README.md) | Hard | 60 min | L2 | 71 |
| Step 41 | [Visitor](java/problems/70-visitor/README.md) | Hard | 50 min | L2 | 70 |

#### Phase 7 - Applied design (state machines, data structures)

Open-ended problems that reuse the patterns above. Small games and caches first, then multi-entity systems.

| Step | Problem | Difficulty | Time | Scaffold | # |
|---|---|---|---:|:---:|---:|
| Step 42 | [Tic-Tac-Toe](java/problems/08-tic-tac-toe/README.md) | Medium | 45 min | L1 | 08 |
| Step 43 | [Snake and Ladder](java/problems/09-snake-and-ladder/README.md) | Medium | 45 min | L1 | 09 |
| Step 44 | [LRU Cache](java/problems/18-lru-cache/README.md) | Medium | 45 min | L2 | 18 |
| Step 45 | [Library Management](java/problems/03-library/README.md) | Medium | 60 min | L1 | 03 |
| Step 46 | [Coffee Machine](java/problems/04-coffee-machine/README.md) | Medium | 60 min | L1 | 04 |
| Step 47 | [Vending Machine](java/problems/05-vending-machine/README.md) | Medium | 60 min | L1 | 05 |
| Step 48 | [ATM](java/problems/06-atm/README.md) | Medium | 60 min | L1 | 06 |
| Step 49 | [Splitwise](java/problems/15-splitwise/README.md) | Medium | 60 min | L1 | 15 |
| Step 50 | [Parking Lot](java/problems/01-parking-lot/README.md) | Medium-Hard | 90 min | L1 | 01 |
| Step 51 | [LFU Cache](java/problems/19-lfu-cache/README.md) | Medium-Hard | 60 min | L2 | 19 |
| Step 52 | [Metrics Collector](java/problems/26-metrics/README.md) | Medium-Hard | 60 min | L1 | 26 |
| Step 53 | [Logging Framework](java/problems/25-logging-framework/README.md) | Medium-Hard | 90 min | L2 | 25 |
| Step 54 | [In-Memory Key-Value Store](java/problems/21-key-value-store/README.md) | Medium-Hard | 90 min | L1 | 21 |
| Step 55 | [Pub-Sub Message Broker](java/problems/22-pub-sub/README.md) | Medium-Hard | 90 min | L1 | 22 |
| Step 56 | [Notification System](java/problems/23-notification-system/README.md) | Medium-Hard | 90 min | L2 | 23 |
| Step 57 | [Digital Wallet](java/problems/16-wallet/README.md) | Medium-Hard | 90 min | L1 | 16 |
| Step 58 | [Bus / Train Seat Booking](java/problems/13-seat-booking/README.md) | Medium-Hard | 90 min | L1 | 13 |
| Step 59 | [Ride Sharing](java/problems/14-ride-sharing/README.md) | Medium-Hard | 90 min | L1 | 14 |
| Step 60 | [Travel Itinerary Planner](java/problems/12-travel-itinerary/README.md) | Medium-Hard | 90 min | L1 | 12 |
| Step 61 | [Elevator System](java/problems/02-elevator/README.md) | Hard | 90 min | L1 | 02 |
| Step 62 | [Flight Booking](java/problems/10-flight-booking/README.md) | Hard | 90 min | L1 | 10 |
| Step 63 | [Hotel Booking](java/problems/11-hotel-booking/README.md) | Hard | 90 min | L1 | 11 |
| Step 64 | [Stock Exchange Order Book](java/problems/17-order-book/README.md) | Hard | 120 min | L1 | 17 |
| Step 65 | [Board Game Engine](java/problems/32-board-game-engine/README.md) | Hard | 120 min | L1 | 32 |
| Step 66 | [Chess](java/problems/07-chess/README.md) | Senior | 120 min | L1 | 07 |

## Concurrency

Multithreading and parallel-programming practice at SDE-3 level (Java and Rust). **Work top to bottom** - this is the order of
learning, not of difficulty: each step uses an idea the earlier ones introduced. Difficulty is shown for reference only.
Problems marked **new** (74-81) were added for this track; the others already live in the foundations track above and are reused here.
Tick the box when the tests pass **and** you can explain your locking choices and the follow-ups out loud.

### Stage 1 - Waiting and waking

*One thread waits until another says go. Learn the condition-variable protocol before anything else.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 1 | [Ordered Printer (Turn Taking)](java/problems/76-ordered-printer/README.md) **new** | Medium | 45 min | wait/notify, `Condvar`, condition loops, lost wake-ups, hand-off | [java](java/problems/76-ordered-printer/README.md) · [rust](rust/problems/76-ordered-printer/README.md) | ☐ |
| Step 2 | [Bounded Blocking Buffer](java/problems/28-blocking-queue/README.md) | Hard | 90 min | producer-consumer, two conditions, close/drain semantics, timeouts | [java](java/problems/28-blocking-queue/README.md) · [rust](rust/problems/28-blocking-queue/README.md) | ☐ |

### Stage 2 - Locks and deadlocks

*Several locks, several objects. Learn lock ordering and what a deadlock needs.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 3 | [Bank Transfers Without Deadlock](java/problems/75-bank-transfer/README.md) **new** | Medium | 45 min | mutual exclusion, lock ordering, deadlock, atomic multi-object updates, consistent snapshots | [java](java/problems/75-bank-transfer/README.md) · [rust](rust/problems/75-bank-transfer/README.md) | ☐ |
| Step 4 | [Dining Philosophers](java/problems/77-dining-philosophers/README.md) **new** | Medium-Hard | 60 min | deadlock conditions, resource ordering, starvation, scalable fairness | [java](java/problems/77-dining-philosophers/README.md) · [rust](rust/problems/77-dining-philosophers/README.md) | ☐ |

### Stage 3 - Shared state and resources

*Read-write locks, bounded resources, timeouts, shutdown.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 5 | [Concurrent LRU Cache (Read-Write Lock)](java/problems/74-rw-lock-lru-cache/README.md) **new** | Medium-Hard | 60 min | read-write locks, lock upgrade pitfalls, shared mutable recency order, visibility, invariants across two data structures | [java](java/problems/74-rw-lock-lru-cache/README.md) · [rust](rust/problems/74-rw-lock-lru-cache/README.md) | ☐ |
| Step 6 | [Bounded Connection Pool](java/problems/78-connection-pool/README.md) **new** | Medium-Hard | 60 min | semaphores, permits vs identity, timeouts, close semantics, resource leaks | [java](java/problems/78-connection-pool/README.md) · [rust](rust/problems/78-connection-pool/README.md) | ☐ |
| Step 7 | [Rate Limiter](java/problems/20-rate-limiter/README.md) | Medium-Hard | 60 min | token bucket, injectable clock, atomics vs lock | [java](java/problems/20-rate-limiter/README.md) · [rust](rust/problems/20-rate-limiter/README.md) | ☐ |
| Step 8 | [Worker Pool](java/problems/29-worker-pool/README.md) | Hard | 90 min | thread pool, task queue, graceful shutdown | [java](java/problems/29-worker-pool/README.md) · [rust](rust/problems/29-worker-pool/README.md) | ☐ |

### Stage 4 - Splitting work across threads

*Divide, run in parallel, combine. Learn bounded parallelism and termination.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 9 | [Parallel Merge Sort](java/problems/79-parallel-merge-sort/README.md) **new** | Medium-Hard | 60 min | fork-join, work splitting, thresholds, bounded parallelism, disjoint ranges | [java](java/problems/79-parallel-merge-sort/README.md) · [rust](rust/problems/79-parallel-merge-sort/README.md) | ☐ |
| Step 10 | [Parallel Word Count](java/problems/80-parallel-word-count/README.md) **new** | Medium | 45 min | partition and merge, thread-local aggregation, contention on shared maps, split boundaries | [java](java/problems/80-parallel-word-count/README.md) · [rust](rust/problems/80-parallel-word-count/README.md) | ☐ |
| Step 11 | [Parallel Web Crawler](java/problems/81-parallel-crawler/README.md) **new** | Hard | 90 min | shared visited set, work queue, termination detection, bounded workers, error isolation | [java](java/problems/81-parallel-crawler/README.md) · [rust](rust/problems/81-parallel-crawler/README.md) | ☐ |

### Stage 5 - Whole systems

*Everything together: caches that load, races on bookings, schedulers and event buses.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 12 | [Concurrent Cache with Loaders](java/problems/30-concurrent-cache/README.md) | Hard | 90 min | per-key in-flight loads, thundering herd, failure handling | [java](java/problems/30-concurrent-cache/README.md) · [rust](rust/problems/30-concurrent-cache/README.md) | ☐ |
| Step 13 | [Bus / Train Seat Booking](java/problems/13-seat-booking/README.md) | Medium-Hard | 90 min | check-then-act races, double booking, holds and expiry | [java](java/problems/13-seat-booking/README.md) · [rust](rust/problems/13-seat-booking/README.md) | ☐ |
| Step 14 | [Circuit Breaker](java/problems/27-circuit-breaker/README.md) | Medium-Hard | 60 min | state machine under concurrency, time windows | [java](java/problems/27-circuit-breaker/README.md) · [rust](rust/problems/27-circuit-breaker/README.md) | ☐ |
| Step 15 | [Task Scheduler](java/problems/24-task-scheduler/README.md) | Hard | 90 min | delayed and recurring tasks, delay queue, cancellation | [java](java/problems/24-task-scheduler/README.md) · [rust](rust/problems/24-task-scheduler/README.md) | ☐ |
| Step 16 | [Pub-Sub Message Broker](java/problems/22-pub-sub/README.md) | Medium-Hard | 90 min | topics, subscribers, delivery ordering | [java](java/problems/22-pub-sub/README.md) · [rust](rust/problems/22-pub-sub/README.md) | ☐ |
| Step 17 | [In-Process Event Bus](java/problems/31-event-bus/README.md) | Hard | 120 min | multiple subscribers, back-pressure, failure isolation, async delivery | [java](java/problems/31-event-bus/README.md) · [rust](rust/problems/31-event-bus/README.md) | ☐ |

### How these tests work

- They are black-box: they only use the thin contract in each problem, so any class design works.
- Many threads hit one object; deadlock timeouts turn a hang into a failure (Java: `Concurrently`; Rust: `run_concurrently` aborts with `TIMEOUT` after 60 s).
- The new problems repeat a randomized workload **100 times** to catch rare interleavings. To hunt a flaky pass: `for i in $(seq 20); do cargo play run 77 || break; done`.
- Each new problem has a `HINTS.md` with three collapsed levels (small nudge, which tool, shape of the solution) - open one level at a time.
- Optional: Rust solutions can also be model-checked with [`loom`](https://github.com/tokio-rs/loom); it is not needed for the provided tests.

## Classic product systems

Interview staples from product companies (URL shortener, ticketing, food delivery, expense groups, file systems, text editors, a tiny SQL engine, ...). **Work top to bottom** - this is the order of learning, not of difficulty. Problems marked **new** (82-95) were added for this track; the others already live in the foundations track above and are reused here. Tick the box when the tests pass **and** you can explain your design and the follow-ups out loud.

These are plain single-threaded designs unless a problem says otherwise (time comes from an injected fake clock, so nothing sleeps). Each has `README.md`, `HINTS.md` (three collapsed levels), `INTERVIEW.md`, `FOLLOWUPS.md`, `RUBRIC.md` and `INTERVIEWER_NOTES.md`, in both Java and Rust.

### Stage 1 - Mapping and rules

*Keys, ids and rule evaluation: the vocabulary every later product problem reuses.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 1 | [URL Shortener](java/problems/82-url-shortener/README.md) **new** | Medium | 45 min | id generation, bidirectional maps, uniqueness under contention, alias reservation, tombstones, atomic counters | [java](java/problems/82-url-shortener/README.md) · [rust](rust/problems/82-url-shortener/README.md) | ☐ |
| Step 2 | [Feature Flag Service](java/problems/83-feature-flags/README.md) **new** | Medium | 45 min | rule evaluation order, deterministic hashing, stable percentage rollout, kill switch, read-mostly concurrency, copy-on-write | [java](java/problems/83-feature-flags/README.md) · [rust](rust/problems/83-feature-flags/README.md) | ☐ |

### Stage 2 - Trees, edits and history

*State that is a tree or a sequence, and how to undo, version and merge it.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 3 | [In-Memory File System with Permissions](java/problems/84-file-system-permissions/README.md) **new** | Medium-Hard | 75 min | tree model, path parsing, unix permission bits, owner/group/other classes, traversal checks, move invariants, locking a tree | [java](java/problems/84-file-system-permissions/README.md) · [rust](rust/problems/84-file-system-permissions/README.md) | ☐ |
| Step 4 | [Text Editor with Undo/Redo](java/problems/85-text-editor-undo/README.md) **new** | Medium | 60 min | command pattern vs snapshots, undo/redo stacks, operation grouping (coalescing), cursor and selection model, state restoration | [java](java/problems/85-text-editor-undo/README.md) · [rust](rust/problems/85-text-editor-undo/README.md) | ☐ |
| Step 5 | [Document Versioning with Merge](java/problems/86-document-versioning/README.md) **new** | Hard | 75 min | append-only history, optimistic concurrency, line diff (LCS), three-way merge, conflict detection, compare-and-append under a lock | [java](java/problems/86-document-versioning/README.md) · [rust](rust/problems/86-document-versioning/README.md) | ☐ |

### Stage 3 - Money and time

*Balances that must add up, and calendars/schedules driven by an injected clock.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 6 | [Splitwise](java/problems/15-splitwise/README.md) | Medium | 60 min | balances, split strategies | [java](java/problems/15-splitwise/README.md) · [rust](rust/problems/15-splitwise/README.md) | ☐ |
| Step 7 | [Expense Groups (Splitwise)](java/problems/87-expense-groups/README.md) **new** | Hard | 75 min | ledger and net balances, fair integer splitting, atomic validation, debt minimisation (zero-sum partition), bitmask DP, money as integers | [java](java/problems/87-expense-groups/README.md) · [rust](rust/problems/87-expense-groups/README.md) | ☐ |
| Step 8 | [Shared Event Calendar](java/problems/88-event-calendar/README.md) **new** | Medium-Hard | 60 min | interval overlap on half-open ranges, all-or-nothing multi-party booking, ordered multi-lock acquisition, per-user sorted structures, free-slot computation | [java](java/problems/88-event-calendar/README.md) · [rust](rust/problems/88-event-calendar/README.md) | ☐ |
| Step 9 | [Cron Job Scheduler](java/problems/93-cron-job-scheduler/README.md) **new** | Hard | 75 min | cron expression parsing, next-slot computation over a calendar, retry with capped exponential backoff, deterministic polling scheduler, misfire policy, dead letters | [java](java/problems/93-cron-job-scheduler/README.md) · [rust](rust/problems/93-cron-job-scheduler/README.md) | ☐ |

### Stage 4 - Marketplaces and bookings

*Several actors competing for limited stock, with holds, expiry and state machines.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 10 | [Hotel Booking](java/problems/11-hotel-booking/README.md) | Hard | 90 min | availability, overlapping ranges | [java](java/problems/11-hotel-booking/README.md) · [rust](rust/problems/11-hotel-booking/README.md) | ☐ |
| Step 11 | [Movie Ticket Booking](java/problems/89-movie-ticketing/README.md) **new** | Hard | 75 min | seat inventory, time-limited holds with a lazily evaluated clock, hold-to-booking state machine, atomic multi-seat reservation, tiered pricing, refund policy | [java](java/problems/89-movie-ticketing/README.md) · [rust](rust/problems/89-movie-ticketing/README.md) | ☐ |
| Step 12 | [Online Auction](java/problems/91-online-auction/README.md) **new** | Medium-Hard | 60 min | compare-and-update on a single highest bid, anti-sniping time extension with an injected clock, lazy auction closing, outbid notifications, reserve price | [java](java/problems/91-online-auction/README.md) · [rust](rust/problems/91-online-auction/README.md) | ☐ |
| Step 13 | [Food Delivery Dispatch](java/problems/90-food-delivery/README.md) **new** | Hard | 75 min | order state machine, all-or-nothing stock reservation, nearest-resource assignment with deterministic ties, FIFO waiting queue, compensation on cancel | [java](java/problems/90-food-delivery/README.md) · [rust](rust/problems/90-food-delivery/README.md) | ☐ |
| Step 14 | [Chat System](java/problems/92-chat-system/README.md) **new** | Hard | 75 min | per-recipient delivery state, offline queues, group membership over time, total order per chat, status aggregation, read receipts | [java](java/problems/92-chat-system/README.md) · [rust](rust/problems/92-chat-system/README.md) | ☐ |

### Stage 5 - Infrastructure products

*Systems other engineers build on: request pipelines and a query engine.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 15 | [API Gateway](java/problems/94-api-gateway/README.md) **new** | Hard | 75 min | request pipeline, longest-prefix routing, token-bucket rate limiting with lazy refill, round-robin with failover, circuit-breaker state machine, injectable clock, per-key locking without serialising backend calls | [java](java/problems/94-api-gateway/README.md) · [rust](rust/problems/94-api-gateway/README.md) | ☐ |
| Step 16 | [Mini SQL Engine](java/problems/95-sql-engine/README.md) **new** | Hard | 90 min | tokenizer and recursive-descent parser, typed in-memory tables, expression evaluation with AND/OR precedence, hash join, grouping and aggregation, primary-key index, statement atomicity, serialised execution | [java](java/problems/95-sql-engine/README.md) · [rust](rust/problems/95-sql-engine/README.md) | ☐ |

## Distributed systems (in-process simulations)

Senior interviews often ask for these *conceptually* ("how does Raft elect a leader?") and only rarely as code - so treat this track as a way to make the concepts concrete, not as a list of things you will be asked to code in 45 minutes. Everything runs in one process on a **deterministic simulation**: a fake clock (one tick = 1 ms), a message network that can delay, drop, duplicate and reorder messages, nodes that crash and recover, and network partitions. Nothing sleeps and every run is reproducible from its seed, so a failing seed is a bug report. Safety invariants (one leader per term, committed entries never change, ...) are checked after **every** simulated millisecond. **Work top to bottom** - this is the order of learning, not of difficulty. Problems 96-105 are new.

### Stage 1 - Building blocks

*In-process pieces every distributed design uses: placement, deduplication, leases.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 1 | [Consistent Hash Ring](java/problems/96-consistent-hashing/README.md) **new** | Medium-Hard | 60 min | consistent hashing, virtual nodes, weighted nodes, sorted ring with binary search, deterministic hashing, replica placement, minimal key movement, read-mostly concurrency | [java](java/problems/96-consistent-hashing/README.md) · [rust](rust/problems/96-consistent-hashing/README.md) | ☐ |
| Step 2 | [Idempotent Request Processor](java/problems/97-idempotent-requests/README.md) **new** | Medium-Hard | 60 min | idempotency keys, exactly-once effect under retries, request fingerprinting, in-flight deduplication (single flight), failure handling and hand-over, TTL expiry, per-key locking, memory reclamation | [java](java/problems/97-idempotent-requests/README.md) · [rust](rust/problems/97-idempotent-requests/README.md) | ☐ |
| Step 3 | [Lease Lock with Fencing Tokens](java/problems/98-lock-lease-fencing/README.md) **new** | Medium-Hard | 50 min | leases, time-bounded mutual exclusion, fencing tokens, monotonic counters, process pauses and clock reasoning, non-blocking API, per-lock state under concurrency | [java](java/problems/98-lock-lease-fencing/README.md) · [rust](rust/problems/98-lock-lease-fencing/README.md) | ☐ |

### Stage 2 - Replication without a leader

*Replicas that accept writes independently and still converge: gossip, CRDTs, quorums.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 4 | [Gossip Protocol with Failure Detection](java/problems/100-gossip-protocol/README.md) **new** | Hard | 90 min | epidemic dissemination, push-pull anti-entropy, versioned last-writer-wins merge, heartbeat failure detection, deterministic simulation | [java](java/problems/100-gossip-protocol/README.md) · [rust](rust/problems/100-gossip-protocol/README.md) | ☐ |
| Step 5 | [CRDT Replicated Store](java/problems/103-crdt-replication/README.md) **new** | Hard | 75 min | state-based CRDTs, join semilattices, PN-counter, observed-remove set, last-writer-wins register, merge laws (idempotent, commutative, associative), snapshot serialization | [java](java/problems/103-crdt-replication/README.md) · [rust](rust/problems/103-crdt-replication/README.md) | ☐ |
| Step 6 | [Quorum Key-Value Store](java/problems/104-quorum-kv/README.md) **new** | Hard | 90 min | Dynamo-style leaderless replication, majority quorums (W + R > N), versioned writes, read repair, request idempotency, timeouts and retries in a simulated network | [java](java/problems/104-quorum-kv/README.md) · [rust](rust/problems/104-quorum-kv/README.md) | ☐ |

### Stage 3 - Agreement

*One leader, one log, same order everywhere.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 7 | [Raft Leader Election](java/problems/99-leader-election/README.md) **new** | Hard | 90 min | Raft leader election, terms, randomized election timeouts, vote granting rules, heartbeats, check-quorum step-down, split-brain safety, deterministic simulation under crashes, partitions and message loss | [java](java/problems/99-leader-election/README.md) · [rust](rust/problems/99-leader-election/README.md) | ☐ |
| Step 8 | [Raft Log Replication](java/problems/105-raft-log/README.md) **new** | Hard | 120 min | Raft consensus, leader election with the election restriction, replicated log with consistency check, commit rule (own-term entries), check-quorum, safety invariants under crashes, partitions and message duplication | [java](java/problems/105-raft-log/README.md) · [rust](rust/problems/105-raft-log/README.md) | ☐ |

### Stage 4 - Transactions across services

*All-or-nothing across several owners: atomic commit and compensation.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 9 | [Two-Phase Commit](java/problems/101-two-phase-commit/README.md) **new** | Hard | 120 min | atomic commit, coordinator/participant protocol, write-ahead logging, in-doubt state and blocking, presumed abort, recovery from a durable log | [java](java/problems/101-two-phase-commit/README.md) · [rust](rust/problems/101-two-phase-commit/README.md) | ☐ |
| Step 10 | [Saga Orchestrator](java/problems/102-saga-orchestrator/README.md) **new** | Hard | 90 min | sagas and compensation, retry with exponential backoff, write-ahead progress log, crash recovery and resumption, idempotency, per-saga mutual exclusion | [java](java/problems/102-saga-orchestrator/README.md) · [rust](rust/problems/102-saga-orchestrator/README.md) | ☐ |

## Full catalogue

Every problem in folder-number order (Java: `java/problems/<#>-<name>`, Rust: `rust/problems/<#>-<name>`). Scaffold: **L1** almost empty, **L2** minimal external API, **L3** infrastructure provided (fake clock, thread helpers or network simulation).

### 01-32 Classic designs

| # | Problem | Difficulty | Time | Scaffold |
|---|---|---|---:|:---:|
| 01 | [Parking Lot](java/problems/01-parking-lot/README.md) | Medium-Hard | 90 min | L1 |
| 02 | [Elevator System](java/problems/02-elevator/README.md) | Hard | 90 min | L1 |
| 03 | [Library Management](java/problems/03-library/README.md) | Medium | 60 min | L1 |
| 04 | [Coffee Machine](java/problems/04-coffee-machine/README.md) | Medium | 60 min | L1 |
| 05 | [Vending Machine](java/problems/05-vending-machine/README.md) | Medium | 60 min | L1 |
| 06 | [ATM](java/problems/06-atm/README.md) | Medium | 60 min | L1 |
| 07 | [Chess](java/problems/07-chess/README.md) | Senior | 120 min | L1 |
| 08 | [Tic-Tac-Toe](java/problems/08-tic-tac-toe/README.md) | Medium | 45 min | L1 |
| 09 | [Snake and Ladder](java/problems/09-snake-and-ladder/README.md) | Medium | 45 min | L1 |
| 10 | [Flight Booking](java/problems/10-flight-booking/README.md) | Hard | 90 min | L1 |
| 11 | [Hotel Booking](java/problems/11-hotel-booking/README.md) | Hard | 90 min | L1 |
| 12 | [Travel Itinerary Planner](java/problems/12-travel-itinerary/README.md) | Medium-Hard | 90 min | L1 |
| 13 | [Bus / Train Seat Booking](java/problems/13-seat-booking/README.md) | Medium-Hard | 90 min | L1 |
| 14 | [Ride Sharing](java/problems/14-ride-sharing/README.md) | Medium-Hard | 90 min | L1 |
| 15 | [Splitwise](java/problems/15-splitwise/README.md) | Medium | 60 min | L1 |
| 16 | [Digital Wallet](java/problems/16-wallet/README.md) | Medium-Hard | 90 min | L1 |
| 17 | [Stock Exchange Order Book](java/problems/17-order-book/README.md) | Hard | 120 min | L1 |
| 18 | [LRU Cache](java/problems/18-lru-cache/README.md) | Medium | 45 min | L2 |
| 19 | [LFU Cache](java/problems/19-lfu-cache/README.md) | Medium-Hard | 60 min | L2 |
| 20 | [Rate Limiter](java/problems/20-rate-limiter/README.md) | Medium-Hard | 60 min | L3 |
| 21 | [In-Memory Key-Value Store](java/problems/21-key-value-store/README.md) | Medium-Hard | 90 min | L1 |
| 22 | [Pub-Sub Message Broker](java/problems/22-pub-sub/README.md) | Medium-Hard | 90 min | L1 |
| 23 | [Notification System](java/problems/23-notification-system/README.md) | Medium-Hard | 90 min | L2 |
| 24 | [Task Scheduler](java/problems/24-task-scheduler/README.md) | Hard | 90 min | L3 |
| 25 | [Logging Framework](java/problems/25-logging-framework/README.md) | Medium-Hard | 90 min | L2 |
| 26 | [Metrics Collector](java/problems/26-metrics/README.md) | Medium-Hard | 60 min | L1 |
| 27 | [Circuit Breaker](java/problems/27-circuit-breaker/README.md) | Medium-Hard | 60 min | L3 |
| 28 | [Bounded Blocking Buffer](java/problems/28-blocking-queue/README.md) | Hard | 90 min | L3 |
| 29 | [Worker Pool](java/problems/29-worker-pool/README.md) | Hard | 90 min | L3 |
| 30 | [Concurrent Cache with Loaders](java/problems/30-concurrent-cache/README.md) | Hard | 90 min | L3 |
| 31 | [In-Process Event Bus](java/problems/31-event-bus/README.md) | Hard | 120 min | L3 |
| 32 | [Board Game Engine](java/problems/32-board-game-engine/README.md) | Hard | 120 min | L1 |

### 33-73 OOP, principles and patterns

These tests **fail until you implement the stubs** (they do not skip).

| # | Problem | Difficulty | Time | Scaffold | What you practise |
|---|---|---|---:|:---:|---|
| 33 | [Classes and Objects](java/problems/33-classes-objects/README.md) | Easy | 30 min | L2 | `Money` value object: state, equality vs identity, immutability. |
| 34 | [Interfaces](java/problems/34-interfaces/README.md) | Easy | 30 min | L2 | IoT sensors behind a `Sensor` interface; the hub works with implementations it has never seen. |
| 35 | [Inheritance](java/problems/35-inheritance/README.md) | Easy | 30 min | L2 | Payroll hierarchy that reuses a base class (Java) / trait defaults + composition (Rust). |
| 36 | [Polymorphism](java/problems/36-polymorphism/README.md) | Easy | 30 min | L2 | Toll plaza that charges any `Vehicle` through one polymorphic call. |
| 37 | [Abstraction](java/problems/37-abstraction/README.md) | Medium | 30 min | L2 | Bank accounts: an abstract base with shared rules and two abstract decisions. |
| 38 | [Encapsulation](java/problems/38-encapsulation/README.md) | Medium | 30 min | L2 | Warehouse stock item whose invariants cannot be broken through the public API. |
| 39 | [Aggregation](java/problems/39-aggregation/README.md) | Easy | 30 min | L2 | Playlists that share songs: the parts outlive the whole. |
| 40 | [Composition](java/problems/40-composition/README.md) | Medium | 40 min | L2 | Order that owns its line items: the parts die with the owner. |
| 41 | [Association](java/problems/41-association/README.md) | Medium | 40 min | L2 | Students and courses, many-to-many, kept consistent from both sides. |
| 42 | [DRY](java/problems/42-dry/README.md) | Medium | 30 min | L2 | Quote, invoice and credit note sharing one pricing path and one money formatter. |
| 43 | [KISS](java/problems/43-kiss/README.md) | Easy | 20 min | L2 | Library late fee in one small function; the tests enforce a simplicity budget. |
| 44 | [YAGNI](java/problems/44-yagni/README.md) | Easy | 25 min | L2 | URL shortener with exactly two operations; the tests reject speculative extras. |
| 45 | [Law of Demeter](java/problems/45-law-of-demeter/README.md) | Medium | 40 min | L2 | Checkout and shipping services that never reach through their collaborators. |
| 46 | [Single Responsibility](java/problems/46-srp/README.md) | Medium | 40 min | L2 | Invoice issuing split into calculator, renderer, repository and a coordinating service. |
| 47 | [Open/Closed](java/problems/47-ocp/README.md) | Medium | 35 min | L2 | Parking-fee calculator extended with new vehicle policies without being modified. |
| 48 | [Liskov Substitution](java/problems/48-lsp/README.md) | Medium | 40 min | L2 | Coupons that all honour one contract, verified by a shared test suite. |
| 49 | [Interface Segregation](java/problems/49-isp/README.md) | Medium | 35 min | L2 | Printer / scanner / fax: small interfaces, devices implement only what they can do. |
| 50 | [Dependency Inversion](java/problems/50-dip/README.md) | Medium | 35 min | L2 | Appointment reminders with an injected sender, repository and clock. |
| 51 | [SOLID Summary](java/problems/51-solid-summary/README.md) | Hard | 75 min | L2 | Meal-order checkout combining S, O, L, I and D in one mini-system. |
| 52 | [Singleton](java/problems/52-singleton/README.md) | Medium | 30 min | L2 | Lazy, thread-safe application config with exactly one instance. |
| 53 | [Factory Method](java/problems/53-factory-method/README.md) | Medium | 35 min | L2 | Checkout flows whose subclasses choose the payment gateway through a factory method. |
| 54 | [Abstract Factory](java/problems/54-abstract-factory/README.md) | Medium | 45 min | L2 | Regional kits (US/EU/IN) creating consistent families of tax, price and address products. |
| 55 | [Builder](java/problems/55-builder/README.md) | Medium | 45 min | L2 | Immutable HTTP request created by a validating fluent builder. |
| 56 | [Prototype](java/problems/56-prototype/README.md) | Medium | 40 min | L2 | Server blueprints deep-cloned from a registry of prototypes. |
| 57 | [Adapter](java/problems/57-adapter/README.md) | Medium | 35 min | L2 | Legacy XML / integer-code SMS gateway exposed as a clean `Notifier`. |
| 58 | [Facade](java/problems/58-facade/README.md) | Medium | 35 min | L2 | Photo upload hiding scanner, resizer, storage and index behind one call, with compensation. |
| 59 | [Decorator](java/problems/59-decorator/README.md) | Medium | 40 min | L2 | Message sinks (timestamp, redact, retry, count) stacked in any order. |
| 60 | [Composite](java/problems/60-composite/README.md) | Medium | 40 min | L2 | File-system tree where files and folders share one interface. |
| 61 | [Proxy](java/problems/61-proxy/README.md) | Medium | 45 min | L2 | Caching, lazy and access-control proxies around a document store. |
| 62 | [Bridge](java/problems/62-bridge/README.md) | Hard | 50 min | L2 | Alerts x channels: any alert type can use any delivery channel. |
| 63 | [Flyweight](java/problems/63-flyweight/README.md) | Medium | 40 min | L2 | Map markers sharing icon flyweights from a thread-safe factory. |
| 64 | [Iterator](java/problems/64-iterator/README.md) | Medium | 45 min | L2 | Lazy paginated feed iterator and a lazy filter iterator. |
| 65 | [Observer](java/problems/65-observer/README.md) | Medium | 40 min | L2 | Price feed with subscribe/cancel, snapshot semantics and failure isolation. |
| 66 | [Strategy](java/problems/66-strategy/README.md) | Medium | 40 min | L2 | Interchangeable shipping-cost strategies, swapped at runtime. |
| 67 | [Command](java/problems/67-command/README.md) | Hard | 50 min | L2 | Text-editor commands with macros and an undo/redo history. |
| 68 | [State](java/problems/68-state/README.md) | Hard | 60 min | L2 | Support-ticket lifecycle built from state objects instead of status checks. |
| 69 | [Template Method](java/problems/69-template-method/README.md) | Medium | 40 min | L2 | Data-import template with fixed steps, overridable hooks and guaranteed clean-up. |
| 70 | [Visitor](java/problems/70-visitor/README.md) | Hard | 50 min | L2 | Document elements visited by HTML-export and word-count visitors. |
| 71 | [Mediator](java/problems/71-mediator/README.md) | Hard | 60 min | L2 | Air-traffic control tower coordinating aircraft landings. |
| 72 | [Memento](java/problems/72-memento/README.md) | Hard | 50 min | L2 | Game character saved into opaque snapshots: save slots and undo/redo. |
| 73 | [Chain of Responsibility](java/problems/73-chain-of-responsibility/README.md) | Medium | 45 min | L2 | Expense approval chain with configurable order and links. |

### 74-81 Concurrency

| # | Problem | Difficulty | Time | Scaffold |
|---|---|---|---:|:---:|
| 74 | [Concurrent LRU Cache (Read-Write Lock)](java/problems/74-rw-lock-lru-cache/README.md) | Medium-Hard | 60 min | L3 |
| 75 | [Bank Transfers Without Deadlock](java/problems/75-bank-transfer/README.md) | Medium | 45 min | L3 |
| 76 | [Ordered Printer (Turn Taking)](java/problems/76-ordered-printer/README.md) | Medium | 45 min | L3 |
| 77 | [Dining Philosophers](java/problems/77-dining-philosophers/README.md) | Medium-Hard | 60 min | L3 |
| 78 | [Bounded Connection Pool](java/problems/78-connection-pool/README.md) | Medium-Hard | 60 min | L3 |
| 79 | [Parallel Merge Sort](java/problems/79-parallel-merge-sort/README.md) | Medium-Hard | 60 min | L3 |
| 80 | [Parallel Word Count](java/problems/80-parallel-word-count/README.md) | Medium | 45 min | L3 |
| 81 | [Parallel Web Crawler](java/problems/81-parallel-crawler/README.md) | Hard | 90 min | L3 |

### 82-95 Classic product systems

| # | Problem | Difficulty | Time | Scaffold |
|---|---|---|---:|:---:|
| 82 | [URL Shortener](java/problems/82-url-shortener/README.md) | Medium | 45 min | L2 |
| 83 | [Feature Flag Service](java/problems/83-feature-flags/README.md) | Medium | 45 min | L2 |
| 84 | [In-Memory File System with Permissions](java/problems/84-file-system-permissions/README.md) | Medium-Hard | 75 min | L2 |
| 85 | [Text Editor with Undo/Redo](java/problems/85-text-editor-undo/README.md) | Medium | 60 min | L2 |
| 86 | [Document Versioning with Merge](java/problems/86-document-versioning/README.md) | Hard | 75 min | L2 |
| 87 | [Expense Groups (Splitwise)](java/problems/87-expense-groups/README.md) | Hard | 75 min | L2 |
| 88 | [Shared Event Calendar](java/problems/88-event-calendar/README.md) | Medium-Hard | 60 min | L2 |
| 89 | [Movie Ticket Booking](java/problems/89-movie-ticketing/README.md) | Hard | 75 min | L2 |
| 90 | [Food Delivery Dispatch](java/problems/90-food-delivery/README.md) | Hard | 75 min | L2 |
| 91 | [Online Auction](java/problems/91-online-auction/README.md) | Medium-Hard | 60 min | L2 |
| 92 | [Chat System](java/problems/92-chat-system/README.md) | Hard | 75 min | L2 |
| 93 | [Cron Job Scheduler](java/problems/93-cron-job-scheduler/README.md) | Hard | 75 min | L2 |
| 94 | [API Gateway](java/problems/94-api-gateway/README.md) | Hard | 75 min | L2 |
| 95 | [Mini SQL Engine](java/problems/95-sql-engine/README.md) | Hard | 90 min | L2 |

### 96-105 Distributed systems

| # | Problem | Difficulty | Time | Scaffold |
|---|---|---|---:|:---:|
| 96 | [Consistent Hash Ring](java/problems/96-consistent-hashing/README.md) | Medium-Hard | 60 min | L3 |
| 97 | [Idempotent Request Processor](java/problems/97-idempotent-requests/README.md) | Medium-Hard | 60 min | L3 |
| 98 | [Lease Lock with Fencing Tokens](java/problems/98-lock-lease-fencing/README.md) | Medium-Hard | 50 min | L3 |
| 99 | [Raft Leader Election](java/problems/99-leader-election/README.md) | Hard | 90 min | L3 |
| 100 | [Gossip Protocol with Failure Detection](java/problems/100-gossip-protocol/README.md) | Hard | 90 min | L3 |
| 101 | [Two-Phase Commit](java/problems/101-two-phase-commit/README.md) | Hard | 120 min | L3 |
| 102 | [Saga Orchestrator](java/problems/102-saga-orchestrator/README.md) | Hard | 90 min | L3 |
| 103 | [CRDT Replicated Store](java/problems/103-crdt-replication/README.md) | Hard | 75 min | L3 |
| 104 | [Quorum Key-Value Store](java/problems/104-quorum-kv/README.md) | Hard | 90 min | L3 |
| 105 | [Raft Log Replication](java/problems/105-raft-log/README.md) | Hard | 120 min | L3 |

## The `play` runner (rustlings-style)

Run and track each problem separately, from the repo root (std-only Rust CLI in `cli/`):

```bash
cargo play list                 # all problems + your progress (rust and java)
cargo play next                 # first problem that is not passing yet
cargo play run 18               # tests for ONE problem (number or slug part: `lru`)
cargo play watch 18             # re-run on every save
cargo play run 20 --hidden      # + interviewer scenarios
cargo play run 18 -l java       # the Java side (needs Maven + JDK 21)
cargo play run 18 --strict      # "not implemented" counts as a failure
cargo play info 18 followups    # readme | interview | followups | rubric
```

Progress is stored in `.playground/progress.txt` (git-ignored). `PLAYGROUND_LANG=java` changes the default language.

## Running everything

```bash
cd java && mvn test                          # all Java problems (use -Phidden for interviewer scenarios)
cd java/problems/07-chess && mvn test        # one Java problem
cd rust && cargo test                        # all Rust problems
cd rust/problems/07-chess && cargo test      # one Rust problem
STRICT=1 cargo test                          # "not implemented" becomes a failure
```

## Where to write your solutions

> **Do not write solutions on `main`.** Keep `main` as the clean, empty playground.
> Create a separate branch for your solutions and work there:
>
> ```bash
> git checkout -b my-solutions
> ```
>
> That way you can always pull updates to the problems on `main` and start fresh by branching again.
