# Machine Coding + LLD Interview Playground

Practice for **machine-coding and low-level-design (LLD) interviews** at about the 6-year level, in **Java 21** and **Rust**.
**143 problems**, each with requirements, a behavioural contract, runnable tests, optional hints, follow-ups and a scoring rubric.

> **Give me the requirements. Give me the tests. Give me the playground. Do NOT give me the architecture.**

You receive what a senior engineer is handed at the start of a real interview:

```
Problem -> Requirements -> Constraints -> Examples -> Behavioural contract -> Test + build environment
```

You are responsible for everything in the middle:

```
Classes · Interfaces · Enums · Value objects · Services · Policies · State machines
Data structures · Ownership of state · Concurrency model · Implementation · Refactoring
```

READMEs describe **what** to build (observable behaviour, examples, edge cases), never **how**. Design advice lives only in each
problem's opt-in `HINTS.md`, so you can try first and open one hint level at a time when stuck. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Contents

- [Quick start](#quick-start)
- [How a problem works](#how-a-problem-works)
- [Learning tracks](#learning-tracks) and [suggested paths](#suggested-paths)
- [My progress checklist](#my-progress-checklist)
- Tracks: [1 Foundations](#track-1---foundations-principles-and-patterns-1-73) ·
  [2 Concurrency](#track-2---concurrency) · [3 Product systems](#track-3---classic-product-systems) ·
  [4 Distributed systems](#track-4---distributed-systems-in-process-simulations) ·
  [5 Microsoft LLD](#track-5---microsoft-lld-round-questions) · [6 Search and RAG](#track-6---search-typeahead-and-rag)
- [Full catalogue](#full-catalogue)
- [The `play` runner](#the-play-runner)
- [Running everything](#running-everything)
- [Where to write your solutions](#where-to-write-your-solutions)
- [Contributing](#contributing)

## Quick start

Requirements: **JDK 21 + Maven** for the Java side and a **stable Rust toolchain** for the Rust side. The `play` runner needs only Rust.

```bash
cargo play list                  # every problem and your progress
cargo play next                  # first problem that is not passing yet
cargo play info 82               # print the README of problem 82
cargo play run 82                # run its tests (Rust); add  -l java  for the Java side
cargo play run 82 --hidden       # also run the interviewer scenarios
```

Or directly: `cd java/problems/82-url-shortener && mvn test` or `cd rust/problems/82-url-shortener && cargo test`.

Pick a problem, read its `README.md`, start the timer from `INTERVIEW.md`, and **think before you type**.

> **Do not write solutions on `main`.** Keep `main` as the clean playground and work on your own branch; see
> [Where to write your solutions](#where-to-write-your-solutions).

## How a problem works

```
<problem>/
├── README.md              what to build: requirements, examples, edge cases, behavioural contract, how to run
├── INTERVIEW.md           time limit, phases, rules, self-review questions
├── FOLLOWUPS.md           V2…V5 extensions - open one at a time
├── RUBRIC.md              how to score yourself or a candidate
├── HINTS.md               optional: three collapsed levels, smallest nudge first (newer problems)
├── INTERVIEWER_NOTES.md   hidden edge cases, traps, discussion guide - do not read while practising
├── interviewer/           extra scenarios, not in the default test run
├── src/ (and tests/)      your design + the behavioural tests
└── pom.xml | Cargo.toml
```

### Architecture-agnostic tests

Tests talk to your system through a **thin, primitive-typed entry point** (strings, numbers, lists, `Optional`/`Option`).
That entry point is a doorway, not a design:

- **Java** - `contract/*` (provided) and `solution/Solution.java` (you wire your own classes in).
- **Rust** - `src/contract.rs` (provided) and `create(...)` in `src/lib.rs` (you wire your own types in).

Choose any model behind it: inheritance, composition, enums, traits, data-oriented structs. Test names describe behaviour
(`shouldReleaseSpotAfterVehicleExits`), never classes.

### Scaffolding levels

| Level | What you get | Typical problems |
|:-:|---|---|
| **L1** | Almost nothing: a thin entry point and tests. Every domain class is yours. | Classic designs: parking lot, elevator, chess ... |
| **L2** | The contract (an interface or trait the tests use, sometimes a few provided types). Everything behind it is yours. | Concept drills 33-73, caches, most newer problems |
| **L3** | L2 plus shared test infrastructure: fake clock, thread coordination, simulated network. | Concurrency and distributed systems |

### Fresh checkout builds green

Until you wire your solution, behavioural tests are **skipped** (Java) or print `SKIPPED` and return (Rust), so `mvn test` and
`cargo test` pass on a clean repo. Use `STRICT=1` to make "not implemented" a failure:

```bash
STRICT=1 mvn test          # Java
STRICT=1 cargo test        # Rust  (PowerShell: $env:STRICT=1; cargo test)
```

The concept drills (33-73) are different: they ship compiling stubs, so their tests **fail until you implement them**.

### Hidden scenarios

Visible tests teach the contract; they do not list every edge case. Each problem also ships `interviewer/` scenarios on the same
contract: `mvn test -Phidden` / `cargo test --features hidden`.

## Learning tracks

| Track | Problems | Focus | Start with |
|---|---|---|---|
| [1. Foundations](#track-1---foundations-principles-and-patterns-1-73) | 01-73 (except 20, 24, 27-31) | OOP, design principles, SOLID, design patterns, classic LLD designs | [Classes and Objects](java/problems/33-classes-objects/README.md) |
| [2. Concurrency](#track-2---concurrency) | 74-81, 120-122 (+ 13, 20, 22, 24, 27-31, 115, 118) | locks, deadlocks, wait/notify, thread pools, parallel algorithms, concurrent data structures | [Ordered Printer (Turn Taking)](java/problems/76-ordered-printer/README.md) |
| [3. Classic product systems](#track-3---classic-product-systems) | 82-95 (+ 11, 15) | URL shortener, ticketing, food delivery, file system, text editor, SQL engine | [URL Shortener](java/problems/82-url-shortener/README.md) |
| [4. Distributed systems](#track-4---distributed-systems-in-process-simulations) | 96-105 | consistent hashing, idempotency, leases, gossip, CRDTs, quorums, Raft, 2PC, sagas | [Consistent Hash Ring](java/problems/96-consistent-hashing/README.md) |
| [5. Microsoft LLD round](#track-5---microsoft-lld-round-questions) | 106-114, 123-143 (+ many existing, tagged below) | job scheduler, autofill, comments, document search, data-source layer, config service, parking strategies, multi-module logging, elevator feasibility | [Job Scheduler](java/problems/106-job-scheduler/README.md) |
| [6. Search, typeahead and RAG](#track-6---search-typeahead-and-rag) | 115-119 | typeahead, inverted index, query pipeline, result cache, retrieval component | [Autocomplete / Typeahead Service](java/problems/115-autocomplete-typeahead/README.md) |

### Suggested paths

| Goal | Route |
|---|---|
| New to LLD | Track 1 top to bottom (phases 1-7), then Track 3. |
| Interview in a week or two | Track 1 phase 7 (applied designs), Track 5, then pick from Track 3. |
| Microsoft-style LLD rounds | Track 5, plus [Parking Lot](java/problems/01-parking-lot/README.md), [Vending Machine](java/problems/05-vending-machine/README.md), [LRU Cache](java/problems/18-lru-cache/README.md), [Notification System](java/problems/23-notification-system/README.md), [Logging Framework](java/problems/25-logging-framework/README.md) and [Elevator System](java/problems/02-elevator/README.md). |
| Concurrency-heavy roles | Track 2 top to bottom. |
| Search / AI roles | Track 6, then [Concurrent Cache with Loaders](java/problems/30-concurrent-cache/README.md), [URL Shortener](java/problems/82-url-shortener/README.md) and [Consistent Hash Ring](java/problems/96-consistent-hashing/README.md). |
| Senior / staff depth | Track 4 after you are comfortable with concurrency and state machines. |

## My progress checklist

Tick a box when the tests pass **and** you can explain your design. In your own clone edit `[ ]` to `[x]`; GitHub and most editors
render these as checkboxes. `cargo play list` shows the real pass/fail status.

<details>
<summary><b>1-32 Classic designs</b></summary>

- [ ] 01 · [Parking Lot](java/problems/01-parking-lot/README.md) · [rust](rust/problems/01-parking-lot/README.md)
- [ ] 02 · [Elevator System](java/problems/02-elevator/README.md) · [rust](rust/problems/02-elevator/README.md)
- [ ] 03 · [Library Management](java/problems/03-library/README.md) · [rust](rust/problems/03-library/README.md)
- [ ] 04 · [Coffee Machine](java/problems/04-coffee-machine/README.md) · [rust](rust/problems/04-coffee-machine/README.md)
- [ ] 05 · [Vending Machine](java/problems/05-vending-machine/README.md) · [rust](rust/problems/05-vending-machine/README.md)
- [ ] 06 · [ATM](java/problems/06-atm/README.md) · [rust](rust/problems/06-atm/README.md)
- [ ] 07 · [Chess](java/problems/07-chess/README.md) · [rust](rust/problems/07-chess/README.md)
- [ ] 08 · [Tic-Tac-Toe](java/problems/08-tic-tac-toe/README.md) · [rust](rust/problems/08-tic-tac-toe/README.md)
- [ ] 09 · [Snake and Ladder](java/problems/09-snake-and-ladder/README.md) · [rust](rust/problems/09-snake-and-ladder/README.md)
- [ ] 10 · [Flight Booking](java/problems/10-flight-booking/README.md) · [rust](rust/problems/10-flight-booking/README.md)
- [ ] 11 · [Hotel Booking](java/problems/11-hotel-booking/README.md) · [rust](rust/problems/11-hotel-booking/README.md)
- [ ] 12 · [Travel Itinerary Planner](java/problems/12-travel-itinerary/README.md) · [rust](rust/problems/12-travel-itinerary/README.md)
- [ ] 13 · [Bus / Train Seat Booking](java/problems/13-seat-booking/README.md) · [rust](rust/problems/13-seat-booking/README.md)
- [ ] 14 · [Ride Sharing](java/problems/14-ride-sharing/README.md) · [rust](rust/problems/14-ride-sharing/README.md)
- [ ] 15 · [Splitwise](java/problems/15-splitwise/README.md) · [rust](rust/problems/15-splitwise/README.md)
- [ ] 16 · [Digital Wallet](java/problems/16-wallet/README.md) · [rust](rust/problems/16-wallet/README.md)
- [ ] 17 · [Stock Exchange Order Book](java/problems/17-order-book/README.md) · [rust](rust/problems/17-order-book/README.md)
- [ ] 18 · [LRU Cache](java/problems/18-lru-cache/README.md) · [rust](rust/problems/18-lru-cache/README.md)
- [ ] 19 · [LFU Cache](java/problems/19-lfu-cache/README.md) · [rust](rust/problems/19-lfu-cache/README.md)
- [ ] 20 · [Rate Limiter](java/problems/20-rate-limiter/README.md) · [rust](rust/problems/20-rate-limiter/README.md)
- [ ] 21 · [In-Memory Key-Value Store](java/problems/21-key-value-store/README.md) · [rust](rust/problems/21-key-value-store/README.md)
- [ ] 22 · [Pub-Sub Message Broker](java/problems/22-pub-sub/README.md) · [rust](rust/problems/22-pub-sub/README.md)
- [ ] 23 · [Notification System](java/problems/23-notification-system/README.md) · [rust](rust/problems/23-notification-system/README.md)
- [ ] 24 · [Task Scheduler](java/problems/24-task-scheduler/README.md) · [rust](rust/problems/24-task-scheduler/README.md)
- [ ] 25 · [Logging Framework](java/problems/25-logging-framework/README.md) · [rust](rust/problems/25-logging-framework/README.md)
- [ ] 26 · [Metrics Collector](java/problems/26-metrics/README.md) · [rust](rust/problems/26-metrics/README.md)
- [ ] 27 · [Circuit Breaker](java/problems/27-circuit-breaker/README.md) · [rust](rust/problems/27-circuit-breaker/README.md)
- [ ] 28 · [Bounded Blocking Buffer](java/problems/28-blocking-queue/README.md) · [rust](rust/problems/28-blocking-queue/README.md)
- [ ] 29 · [Worker Pool](java/problems/29-worker-pool/README.md) · [rust](rust/problems/29-worker-pool/README.md)
- [ ] 30 · [Concurrent Cache with Loaders](java/problems/30-concurrent-cache/README.md) · [rust](rust/problems/30-concurrent-cache/README.md)
- [ ] 31 · [In-Process Event Bus](java/problems/31-event-bus/README.md) · [rust](rust/problems/31-event-bus/README.md)
- [ ] 32 · [Board Game Engine](java/problems/32-board-game-engine/README.md) · [rust](rust/problems/32-board-game-engine/README.md)

</details>

<details>
<summary><b>33-73 OOP, principles and patterns</b></summary>

- [ ] 33 · [Classes and Objects](java/problems/33-classes-objects/README.md) · [rust](rust/problems/33-classes-objects/README.md)
- [ ] 34 · [Interfaces](java/problems/34-interfaces/README.md) · [rust](rust/problems/34-interfaces/README.md)
- [ ] 35 · [Inheritance](java/problems/35-inheritance/README.md) · [rust](rust/problems/35-inheritance/README.md)
- [ ] 36 · [Polymorphism](java/problems/36-polymorphism/README.md) · [rust](rust/problems/36-polymorphism/README.md)
- [ ] 37 · [Abstraction](java/problems/37-abstraction/README.md) · [rust](rust/problems/37-abstraction/README.md)
- [ ] 38 · [Encapsulation](java/problems/38-encapsulation/README.md) · [rust](rust/problems/38-encapsulation/README.md)
- [ ] 39 · [Aggregation](java/problems/39-aggregation/README.md) · [rust](rust/problems/39-aggregation/README.md)
- [ ] 40 · [Composition](java/problems/40-composition/README.md) · [rust](rust/problems/40-composition/README.md)
- [ ] 41 · [Association](java/problems/41-association/README.md) · [rust](rust/problems/41-association/README.md)
- [ ] 42 · [DRY — Don't Repeat Yourself](java/problems/42-dry/README.md) · [rust](rust/problems/42-dry/README.md)
- [ ] 43 · [KISS — Keep It Simple](java/problems/43-kiss/README.md) · [rust](rust/problems/43-kiss/README.md)
- [ ] 44 · [YAGNI — You Aren't Gonna Need It](java/problems/44-yagni/README.md) · [rust](rust/problems/44-yagni/README.md)
- [ ] 45 · [Law of Demeter](java/problems/45-law-of-demeter/README.md) · [rust](rust/problems/45-law-of-demeter/README.md)
- [ ] 46 · [SRP — Single Responsibility Principle](java/problems/46-srp/README.md) · [rust](rust/problems/46-srp/README.md)
- [ ] 47 · [OCP — Open/Closed Principle](java/problems/47-ocp/README.md) · [rust](rust/problems/47-ocp/README.md)
- [ ] 48 · [LSP — Liskov Substitution Principle](java/problems/48-lsp/README.md) · [rust](rust/problems/48-lsp/README.md)
- [ ] 49 · [ISP — Interface Segregation Principle](java/problems/49-isp/README.md) · [rust](rust/problems/49-isp/README.md)
- [ ] 50 · [DIP — Dependency Inversion Principle](java/problems/50-dip/README.md) · [rust](rust/problems/50-dip/README.md)
- [ ] 51 · [SOLID Summary — Meal Order Checkout](java/problems/51-solid-summary/README.md) · [rust](rust/problems/51-solid-summary/README.md)
- [ ] 52 · [Singleton](java/problems/52-singleton/README.md) · [rust](rust/problems/52-singleton/README.md)
- [ ] 53 · [Factory Method](java/problems/53-factory-method/README.md) · [rust](rust/problems/53-factory-method/README.md)
- [ ] 54 · [Abstract Factory](java/problems/54-abstract-factory/README.md) · [rust](rust/problems/54-abstract-factory/README.md)
- [ ] 55 · [Builder](java/problems/55-builder/README.md) · [rust](rust/problems/55-builder/README.md)
- [ ] 56 · [Prototype](java/problems/56-prototype/README.md) · [rust](rust/problems/56-prototype/README.md)
- [ ] 57 · [Adapter](java/problems/57-adapter/README.md) · [rust](rust/problems/57-adapter/README.md)
- [ ] 58 · [Facade](java/problems/58-facade/README.md) · [rust](rust/problems/58-facade/README.md)
- [ ] 59 · [Decorator](java/problems/59-decorator/README.md) · [rust](rust/problems/59-decorator/README.md)
- [ ] 60 · [Composite](java/problems/60-composite/README.md) · [rust](rust/problems/60-composite/README.md)
- [ ] 61 · [Proxy](java/problems/61-proxy/README.md) · [rust](rust/problems/61-proxy/README.md)
- [ ] 62 · [Bridge](java/problems/62-bridge/README.md) · [rust](rust/problems/62-bridge/README.md)
- [ ] 63 · [Flyweight](java/problems/63-flyweight/README.md) · [rust](rust/problems/63-flyweight/README.md)
- [ ] 64 · [Iterator](java/problems/64-iterator/README.md) · [rust](rust/problems/64-iterator/README.md)
- [ ] 65 · [Observer](java/problems/65-observer/README.md) · [rust](rust/problems/65-observer/README.md)
- [ ] 66 · [Strategy](java/problems/66-strategy/README.md) · [rust](rust/problems/66-strategy/README.md)
- [ ] 67 · [Command](java/problems/67-command/README.md) · [rust](rust/problems/67-command/README.md)
- [ ] 68 · [State](java/problems/68-state/README.md) · [rust](rust/problems/68-state/README.md)
- [ ] 69 · [Template Method](java/problems/69-template-method/README.md) · [rust](rust/problems/69-template-method/README.md)
- [ ] 70 · [Visitor](java/problems/70-visitor/README.md) · [rust](rust/problems/70-visitor/README.md)
- [ ] 71 · [Mediator](java/problems/71-mediator/README.md) · [rust](rust/problems/71-mediator/README.md)
- [ ] 72 · [Memento](java/problems/72-memento/README.md) · [rust](rust/problems/72-memento/README.md)
- [ ] 73 · [Chain of Responsibility](java/problems/73-chain-of-responsibility/README.md) · [rust](rust/problems/73-chain-of-responsibility/README.md)

</details>

<details>
<summary><b>74-81 Concurrency</b></summary>

- [ ] 74 · [Concurrent LRU Cache (Read-Write Lock)](java/problems/74-rw-lock-lru-cache/README.md) · [rust](rust/problems/74-rw-lock-lru-cache/README.md)
- [ ] 75 · [Bank Transfers Without Deadlock](java/problems/75-bank-transfer/README.md) · [rust](rust/problems/75-bank-transfer/README.md)
- [ ] 76 · [Ordered Printer (Turn Taking)](java/problems/76-ordered-printer/README.md) · [rust](rust/problems/76-ordered-printer/README.md)
- [ ] 77 · [Dining Philosophers](java/problems/77-dining-philosophers/README.md) · [rust](rust/problems/77-dining-philosophers/README.md)
- [ ] 78 · [Bounded Connection Pool](java/problems/78-connection-pool/README.md) · [rust](rust/problems/78-connection-pool/README.md)
- [ ] 79 · [Parallel Merge Sort](java/problems/79-parallel-merge-sort/README.md) · [rust](rust/problems/79-parallel-merge-sort/README.md)
- [ ] 80 · [Parallel Word Count](java/problems/80-parallel-word-count/README.md) · [rust](rust/problems/80-parallel-word-count/README.md)
- [ ] 81 · [Parallel Web Crawler](java/problems/81-parallel-crawler/README.md) · [rust](rust/problems/81-parallel-crawler/README.md)

</details>

<details>
<summary><b>82-95 Classic product systems</b></summary>

- [ ] 82 · [URL Shortener](java/problems/82-url-shortener/README.md) · [rust](rust/problems/82-url-shortener/README.md)
- [ ] 83 · [Feature Flag Service](java/problems/83-feature-flags/README.md) · [rust](rust/problems/83-feature-flags/README.md)
- [ ] 84 · [In-Memory File System with Permissions](java/problems/84-file-system-permissions/README.md) · [rust](rust/problems/84-file-system-permissions/README.md)
- [ ] 85 · [Text Editor with Undo/Redo](java/problems/85-text-editor-undo/README.md) · [rust](rust/problems/85-text-editor-undo/README.md)
- [ ] 86 · [Document Versioning with Merge](java/problems/86-document-versioning/README.md) · [rust](rust/problems/86-document-versioning/README.md)
- [ ] 87 · [Expense Groups (Splitwise)](java/problems/87-expense-groups/README.md) · [rust](rust/problems/87-expense-groups/README.md)
- [ ] 88 · [Shared Event Calendar](java/problems/88-event-calendar/README.md) · [rust](rust/problems/88-event-calendar/README.md)
- [ ] 89 · [Movie Ticket Booking](java/problems/89-movie-ticketing/README.md) · [rust](rust/problems/89-movie-ticketing/README.md)
- [ ] 90 · [Food Delivery Dispatch](java/problems/90-food-delivery/README.md) · [rust](rust/problems/90-food-delivery/README.md)
- [ ] 91 · [Online Auction](java/problems/91-online-auction/README.md) · [rust](rust/problems/91-online-auction/README.md)
- [ ] 92 · [Chat System](java/problems/92-chat-system/README.md) · [rust](rust/problems/92-chat-system/README.md)
- [ ] 93 · [Cron Job Scheduler](java/problems/93-cron-job-scheduler/README.md) · [rust](rust/problems/93-cron-job-scheduler/README.md)
- [ ] 94 · [API Gateway](java/problems/94-api-gateway/README.md) · [rust](rust/problems/94-api-gateway/README.md)
- [ ] 95 · [Mini SQL Engine](java/problems/95-sql-engine/README.md) · [rust](rust/problems/95-sql-engine/README.md)

</details>

<details>
<summary><b>96-105 Distributed systems</b></summary>

- [ ] 96 · [Consistent Hash Ring](java/problems/96-consistent-hashing/README.md) · [rust](rust/problems/96-consistent-hashing/README.md)
- [ ] 97 · [Idempotent Request Processor](java/problems/97-idempotent-requests/README.md) · [rust](rust/problems/97-idempotent-requests/README.md)
- [ ] 98 · [Lease Lock with Fencing Tokens](java/problems/98-lock-lease-fencing/README.md) · [rust](rust/problems/98-lock-lease-fencing/README.md)
- [ ] 99 · [Raft Leader Election](java/problems/99-leader-election/README.md) · [rust](rust/problems/99-leader-election/README.md)
- [ ] 100 · [Gossip Protocol with Failure Detection](java/problems/100-gossip-protocol/README.md) · [rust](rust/problems/100-gossip-protocol/README.md)
- [ ] 101 · [Two-Phase Commit](java/problems/101-two-phase-commit/README.md) · [rust](rust/problems/101-two-phase-commit/README.md)
- [ ] 102 · [Saga Orchestrator](java/problems/102-saga-orchestrator/README.md) · [rust](rust/problems/102-saga-orchestrator/README.md)
- [ ] 103 · [CRDT Replicated Store](java/problems/103-crdt-replication/README.md) · [rust](rust/problems/103-crdt-replication/README.md)
- [ ] 104 · [Quorum Key-Value Store](java/problems/104-quorum-kv/README.md) · [rust](rust/problems/104-quorum-kv/README.md)
- [ ] 105 · [Raft Log Replication](java/problems/105-raft-log/README.md) · [rust](rust/problems/105-raft-log/README.md)

</details>

<details>
<summary><b>106-114 Microsoft LLD round</b></summary>

- [ ] 106 · [Job Scheduler](java/problems/106-job-scheduler/README.md) · [rust](rust/problems/106-job-scheduler/README.md)
- [ ] 107 · [Excel-Like Autofill Engine](java/problems/107-excel-autofill/README.md) · [rust](rust/problems/107-excel-autofill/README.md)
- [ ] 108 · [Document Comment Service](java/problems/108-document-comments/README.md) · [rust](rust/problems/108-document-comments/README.md)
- [ ] 109 · [Document Line Search](java/problems/109-document-line-search/README.md) · [rust](rust/problems/109-document-line-search/README.md)
- [ ] 110 · [# 110 Runtime Data Source Access Layer](java/problems/110-data-source-layer/README.md) · [rust](rust/problems/110-data-source-layer/README.md)
- [ ] 111 · [Configuration Management Service](java/problems/111-config-management/README.md) · [rust](rust/problems/111-config-management/README.md)
- [ ] 112 · [Parking Lot with Allocation Strategies](java/problems/112-parking-lot-strategies/README.md) · [rust](rust/problems/112-parking-lot-strategies/README.md)
- [ ] 113 · [Multi-Module Logging Framework](java/problems/113-multi-module-logging/README.md) · [rust](rust/problems/113-multi-module-logging/README.md)
- [ ] 114 · [Elevator Request Feasibility](java/problems/114-elevator-feasibility/README.md) · [rust](rust/problems/114-elevator-feasibility/README.md)

</details>

<details>
<summary><b>115-119 Search, typeahead and RAG</b></summary>

- [ ] 115 · [Autocomplete / Typeahead Service](java/problems/115-autocomplete-typeahead/README.md) · [rust](rust/problems/115-autocomplete-typeahead/README.md)
- [ ] 116 · [In-Memory Search Engine](java/problems/116-inverted-index-search/README.md) · [rust](rust/problems/116-inverted-index-search/README.md)
- [ ] 117 · [Search Query Pipeline](java/problems/117-query-pipeline/README.md) · [rust](rust/problems/117-query-pipeline/README.md)
- [ ] 118 · [Search Result Cache](java/problems/118-search-result-cache/README.md) · [rust](rust/problems/118-search-result-cache/README.md)
- [ ] 119 · [Mini RAG Retrieval Component](java/problems/119-mini-rag-retriever/README.md) · [rust](rust/problems/119-mini-rag-retriever/README.md)

</details>

<details>
<summary><b>120-122 Concurrent data structures</b></summary>

- [ ] 120 · [Thread-Safe Trie](java/problems/120-thread-safe-trie/README.md) · [rust](rust/problems/120-thread-safe-trie/README.md)
- [ ] 121 · [Concurrent Bloom Filter](java/problems/121-concurrent-bloom-filter/README.md) · [rust](rust/problems/121-concurrent-bloom-filter/README.md)
- [ ] 122 · [Versioned Value Store](java/problems/122-versioned-value-store/README.md) · [rust](rust/problems/122-versioned-value-store/README.md)

</details>

<details>
<summary><b>123-143 Microsoft LLD round (more questions)</b></summary>

- [ ] 123 · [Snake Game With Food And Score](java/problems/123-snake-game/README.md) · [rust](rust/problems/123-snake-game/README.md)
- [ ] 124 · [Kafka-Like Message Streaming Service](java/problems/124-kafka-streaming/README.md) · [rust](rust/problems/124-kafka-streaming/README.md)
- [ ] 125 · [Music Streaming and Search](java/problems/125-music-streaming/README.md) · [rust](rust/problems/125-music-streaming/README.md)
- [ ] 126 · [Alert Monitoring System](java/problems/126-alert-monitoring/README.md) · [rust](rust/problems/126-alert-monitoring/README.md)
- [ ] 127 · [Browser History Manager](java/problems/127-browser-history/README.md) · [rust](rust/problems/127-browser-history/README.md)
- [ ] 128 · [Local Search with Ranking](java/problems/128-local-search-ranking/README.md) · [rust](rust/problems/128-local-search-ranking/README.md)
- [ ] 129 · [Job Posting with Candidate Matching](java/problems/129-job-candidate-matching/README.md) · [rust](rust/problems/129-job-candidate-matching/README.md)
- [ ] 130 · [Ludo](java/problems/130-ludo/README.md) · [rust](rust/problems/130-ludo/README.md)
- [ ] 131 · [Shortest Distance and Nearest Landmarks](java/problems/131-landmark-distances/README.md) · [rust](rust/problems/131-landmark-distances/README.md)
- [ ] 132 · [Word Processor](java/problems/132-word-processor/README.md) · [rust](rust/problems/132-word-processor/README.md)
- [ ] 133 · [Spreadsheet Grid](java/problems/133-spreadsheet-grid/README.md) · [rust](rust/problems/133-spreadsheet-grid/README.md)
- [ ] 134 · [Dictionary App](java/problems/134-dictionary-app/README.md) · [rust](rust/problems/134-dictionary-app/README.md)
- [ ] 135 · [Container Orchestrator](java/problems/135-container-orchestrator/README.md) · [rust](rust/problems/135-container-orchestrator/README.md)
- [ ] 136 · [Fantasy Team Leaderboard](java/problems/136-fantasy-leaderboard/README.md) · [rust](rust/problems/136-fantasy-leaderboard/README.md)
- [ ] 137 · [Configurable Time Window Hit Counter](java/problems/137-hit-counter-window/README.md) · [rust](rust/problems/137-hit-counter-window/README.md)
- [ ] 138 · [Multi-Threaded Page Visit Counter](java/problems/138-page-visit-counter/README.md) · [rust](rust/problems/138-page-visit-counter/README.md)
- [ ] 139 · [Shopping Cart](java/problems/139-shopping-cart/README.md) · [rust](rust/problems/139-shopping-cart/README.md)
- [ ] 140 · [Custom HashMap](java/problems/140-custom-hashmap/README.md) · [rust](rust/problems/140-custom-hashmap/README.md)
- [ ] 141 · [Matrix Traversal Strategies](java/problems/141-matrix-traversal/README.md) · [rust](rust/problems/141-matrix-traversal/README.md)
- [ ] 142 · [Restaurant Customer Tokens](java/problems/142-restaurant-tokens/README.md) · [rust](rust/problems/142-restaurant-tokens/README.md)
- [ ] 143 · [Excel Cell Values and Sum Formulas](java/problems/143-excel-formulas/README.md) · [rust](rust/problems/143-excel-formulas/README.md)

</details>

## Track 1 - Foundations, principles and patterns (1-73)

The first 73 problems are sequenced so each builds on the ones before it: fundamentals, principles, patterns, open-ended design, and finally the larger applied designs. The concurrency-focused designs (20, 24, 27-31) are taught in [Track 2](#track-2---concurrency) instead. Work top to bottom. The *Step* column is the suggested sequence; `#` is the folder number (`cargo play run <#>`).
Problems 33-73 are small concept drills (exact API, stubs that make the tests compile, tests that check the *concept*); 1-32 are open designs.

### Phase 1 - OOP fundamentals

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

### Phase 2 - Design principles

Habits that keep designs small and clean.

|---|---|---|---:|:---:|---:|
| Step 10 | [KISS](java/problems/43-kiss/README.md) | Easy | 20 min | L2 | 43 |
| Step 11 | [YAGNI](java/problems/44-yagni/README.md) | Easy | 25 min | L2 | 44 |
| Step 12 | [DRY](java/problems/42-dry/README.md) | Medium | 30 min | L2 | 42 |
| Step 13 | [Law of Demeter](java/problems/45-law-of-demeter/README.md) | Medium | 40 min | L2 | 45 |

### Phase 3 - SOLID

Applies the habits to class design; finish with the combined exercise.

|---|---|---|---:|:---:|---:|
| Step 14 | [Single Responsibility](java/problems/46-srp/README.md) | Medium | 40 min | L2 | 46 |
| Step 15 | [Open/Closed](java/problems/47-ocp/README.md) | Medium | 35 min | L2 | 47 |
| Step 16 | [Liskov Substitution](java/problems/48-lsp/README.md) | Medium | 40 min | L2 | 48 |
| Step 17 | [Interface Segregation](java/problems/49-isp/README.md) | Medium | 35 min | L2 | 49 |
| Step 18 | [Dependency Inversion](java/problems/50-dip/README.md) | Medium | 35 min | L2 | 50 |
| Step 19 | [SOLID Summary](java/problems/51-solid-summary/README.md) | Hard | 75 min | L2 | 51 |

### Phase 4 - Creational patterns

How objects get made.

|---|---|---|---:|:---:|---:|
| Step 20 | [Singleton](java/problems/52-singleton/README.md) | Medium | 30 min | L2 | 52 |
| Step 21 | [Factory Method](java/problems/53-factory-method/README.md) | Medium | 35 min | L2 | 53 |
| Step 22 | [Abstract Factory](java/problems/54-abstract-factory/README.md) | Medium | 45 min | L2 | 54 |
| Step 23 | [Builder](java/problems/55-builder/README.md) | Medium | 45 min | L2 | 55 |
| Step 24 | [Prototype](java/problems/56-prototype/README.md) | Medium | 40 min | L2 | 56 |

### Phase 5 - Structural patterns

How objects are wired together. Bridge comes last since it builds on Adapter/Composite thinking.

|---|---|---|---:|:---:|---:|
| Step 25 | [Adapter](java/problems/57-adapter/README.md) | Medium | 35 min | L2 | 57 |
| Step 26 | [Facade](java/problems/58-facade/README.md) | Medium | 35 min | L2 | 58 |
| Step 27 | [Decorator](java/problems/59-decorator/README.md) | Medium | 40 min | L2 | 59 |
| Step 28 | [Composite](java/problems/60-composite/README.md) | Medium | 40 min | L2 | 60 |
| Step 29 | [Proxy](java/problems/61-proxy/README.md) | Medium | 45 min | L2 | 61 |
| Step 30 | [Flyweight](java/problems/63-flyweight/README.md) | Medium | 40 min | L2 | 63 |
| Step 31 | [Bridge](java/problems/62-bridge/README.md) | Hard | 50 min | L2 | 62 |

### Phase 6 - Behavioural patterns

How objects collaborate. Start with the everyday ones (Strategy, Observer); Visitor last.

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

### Phase 7 - Applied design (state machines, data structures)

Open-ended problems that reuse the patterns above. Small games and caches first, then multi-entity systems.

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

## Track 2 - Concurrency

Multithreading and parallel-programming practice at SDE-3 level (Java and Rust). **Work top to bottom** - this is the order of
learning, not of difficulty: each step uses an idea the earlier ones introduced. Difficulty is shown for reference only.
Problems marked **new** (74-81, 120-122) were added for this track; the others already live in the foundations track above and are reused here.
Tick the box when the tests pass **and** you can explain your locking choices and the follow-ups out loud.

### Stage 1 - Waiting and waking

*One thread waits until another says go. Learn the condition-variable protocol before anything else.*

| Step | Problem | Difficulty | Time | Key concepts | Code | Done |
|---|---|---|---:|---|---|:---:|
| Step 1 | [Ordered Printer (Turn Taking)](java/problems/76-ordered-printer/README.md) **new** | Medium | 45 min | wait/notify, `Condvar`, condition loops, lost wake-ups, hand-off | [java](java/problems/76-ordered-printer/README.md) · [rust](rust/problems/76-ordered-printer/README.md) | ☐ |
| Step 2 | [Bounded Blocking Buffer](java/problems/28-blocking-queue/README.md) | Hard | 90 min | producer-consumer, two conditions, close/drain semantics, timeouts | [java](java/problems/28-blocking-queue/README.md) · [rust](rust/problems/28-blocking-queue/README.md) | ☐ |

### Stage 2 - Locks and deadlocks

*Several locks, several objects. Learn lock ordering and what a deadlock needs.*

|---|---|---|---:|---|---|:---:|
| Step 3 | [Bank Transfers Without Deadlock](java/problems/75-bank-transfer/README.md) **new** | Medium | 45 min | mutual exclusion, lock ordering, deadlock, atomic multi-object updates, consistent snapshots | [java](java/problems/75-bank-transfer/README.md) · [rust](rust/problems/75-bank-transfer/README.md) | ☐ |
| Step 4 | [Dining Philosophers](java/problems/77-dining-philosophers/README.md) **new** | Medium-Hard | 60 min | deadlock conditions, resource ordering, starvation, scalable fairness | [java](java/problems/77-dining-philosophers/README.md) · [rust](rust/problems/77-dining-philosophers/README.md) | ☐ |

### Stage 3 - Shared state and resources

*Read-write locks, bounded resources, timeouts, shutdown.*

|---|---|---|---:|---|---|:---:|
| Step 5 | [Concurrent LRU Cache (Read-Write Lock)](java/problems/74-rw-lock-lru-cache/README.md) **new** | Medium-Hard | 60 min | read-write locks, lock upgrade pitfalls, shared mutable recency order, visibility, invariants across two data structures | [java](java/problems/74-rw-lock-lru-cache/README.md) · [rust](rust/problems/74-rw-lock-lru-cache/README.md) | ☐ |
| Step 6 | [Bounded Connection Pool](java/problems/78-connection-pool/README.md) **new** | Medium-Hard | 60 min | semaphores, permits vs identity, timeouts, close semantics, resource leaks | [java](java/problems/78-connection-pool/README.md) · [rust](rust/problems/78-connection-pool/README.md) | ☐ |
| Step 7 | [Rate Limiter](java/problems/20-rate-limiter/README.md) | Medium-Hard | 60 min | token bucket, injectable clock, atomics vs lock | [java](java/problems/20-rate-limiter/README.md) · [rust](rust/problems/20-rate-limiter/README.md) | ☐ |
| Step 8 | [Worker Pool](java/problems/29-worker-pool/README.md) | Hard | 90 min | thread pool, task queue, graceful shutdown | [java](java/problems/29-worker-pool/README.md) · [rust](rust/problems/29-worker-pool/README.md) | ☐ |

### Stage 4 - Splitting work across threads

*Divide, run in parallel, combine. Learn bounded parallelism and termination.*

|---|---|---|---:|---|---|:---:|
| Step 9 | [Parallel Merge Sort](java/problems/79-parallel-merge-sort/README.md) **new** | Medium-Hard | 60 min | fork-join, work splitting, thresholds, bounded parallelism, disjoint ranges | [java](java/problems/79-parallel-merge-sort/README.md) · [rust](rust/problems/79-parallel-merge-sort/README.md) | ☐ |
| Step 10 | [Parallel Word Count](java/problems/80-parallel-word-count/README.md) **new** | Medium | 45 min | partition and merge, thread-local aggregation, contention on shared maps, split boundaries | [java](java/problems/80-parallel-word-count/README.md) · [rust](rust/problems/80-parallel-word-count/README.md) | ☐ |
| Step 11 | [Parallel Web Crawler](java/problems/81-parallel-crawler/README.md) **new** | Hard | 90 min | shared visited set, work queue, termination detection, bounded workers, error isolation | [java](java/problems/81-parallel-crawler/README.md) · [rust](rust/problems/81-parallel-crawler/README.md) | ☐ |

### Stage 5 - Whole systems

*Everything together: caches that load, races on bookings, schedulers and event buses.*

|---|---|---|---:|---|---|:---:|
| Step 12 | [Concurrent Cache with Loaders](java/problems/30-concurrent-cache/README.md) | Hard | 90 min | per-key in-flight loads, thundering herd, failure handling | [java](java/problems/30-concurrent-cache/README.md) · [rust](rust/problems/30-concurrent-cache/README.md) | ☐ |
| Step 13 | [Bus / Train Seat Booking](java/problems/13-seat-booking/README.md) | Medium-Hard | 90 min | check-then-act races, double booking, holds and expiry | [java](java/problems/13-seat-booking/README.md) · [rust](rust/problems/13-seat-booking/README.md) | ☐ |
| Step 14 | [Circuit Breaker](java/problems/27-circuit-breaker/README.md) | Medium-Hard | 60 min | state machine under concurrency, time windows | [java](java/problems/27-circuit-breaker/README.md) · [rust](rust/problems/27-circuit-breaker/README.md) | ☐ |
| Step 15 | [Task Scheduler](java/problems/24-task-scheduler/README.md) | Hard | 90 min | delayed and recurring tasks, delay queue, cancellation | [java](java/problems/24-task-scheduler/README.md) · [rust](rust/problems/24-task-scheduler/README.md) | ☐ |
| Step 16 | [Pub-Sub Message Broker](java/problems/22-pub-sub/README.md) | Medium-Hard | 90 min | topics, subscribers, delivery ordering | [java](java/problems/22-pub-sub/README.md) · [rust](rust/problems/22-pub-sub/README.md) | ☐ |
| Step 17 | [In-Process Event Bus](java/problems/31-event-bus/README.md) | Hard | 120 min | multiple subscribers, back-pressure, failure isolation, async delivery | [java](java/problems/31-event-bus/README.md) · [rust](rust/problems/31-event-bus/README.md) | ☐ |

### Stage 6 - Concurrent data structures

*Classic structures made safe for many threads: shared state, snapshots and thundering herds. Problems marked **new** were added for this stage.*

|---|---|---|---:|---|---|:---:|
| Step 18 | [Concurrent Bloom Filter](java/problems/121-concurrent-bloom-filter/README.md) **new** | Hard | 60 min | atomic bit updates, no false negatives under concurrency, deterministic hashing, bounded false-positive rate | [java](java/problems/121-concurrent-bloom-filter/README.md) · [rust](rust/problems/121-concurrent-bloom-filter/README.md) | ☐ |
| Step 19 | [Thread-Safe Trie](java/problems/120-thread-safe-trie/README.md) **new** | Hard | 75 min | atomic and snapshot-consistent reads and writes on a shared tree, lock granularity, linearisable operations | [java](java/problems/120-thread-safe-trie/README.md) · [rust](rust/problems/120-thread-safe-trie/README.md) | ☐ |
| Step 20 | [Versioned Value Store](java/problems/122-versioned-value-store/README.md) **new** | Hard | 75 min | gap-free global versions, consistent snapshots, compare-and-set, atomic batches, compaction | [java](java/problems/122-versioned-value-store/README.md) · [rust](rust/problems/122-versioned-value-store/README.md) | ☐ |
| Step 21 | [Autocomplete Typeahead Service](java/problems/115-autocomplete-typeahead/README.md) | Hard | 75 min | hot counters under concurrent writers, readers during updates, ranking rules | [java](java/problems/115-autocomplete-typeahead/README.md) · [rust](rust/problems/115-autocomplete-typeahead/README.md) | ☐ |
| Step 22 | [Search Result Cache](java/problems/118-search-result-cache/README.md) | Hard | 75 min | request coalescing, invalidation racing with in-flight loads, TTL and LRU under contention | [java](java/problems/118-search-result-cache/README.md) · [rust](rust/problems/118-search-result-cache/README.md) | ☐ |

### How these tests work

- They are black-box: they only use the thin contract in each problem, so any class design works.
- Many threads hit one object; deadlock timeouts turn a hang into a failure (Java: `Concurrently`; Rust: `run_concurrently` aborts with `TIMEOUT` after 60 s).
- The new problems repeat a randomized workload **100 times** to catch rare interleavings. To hunt a flaky pass: `for i in $(seq 20); do cargo play run 77 || break; done`.
- Each new problem has a `HINTS.md` with three collapsed levels (small nudge, which tool, shape of the solution) - open one level at a time.
- Optional: Rust solutions can also be model-checked with [`loom`](https://github.com/tokio-rs/loom); it is not needed for the provided tests.

## Track 3 - Classic product systems

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

|---|---|---|---:|---|---|:---:|
| Step 3 | [In-Memory File System with Permissions](java/problems/84-file-system-permissions/README.md) **new** | Medium-Hard | 75 min | tree model, path parsing, unix permission bits, owner/group/other classes, traversal checks, move invariants, locking a tree | [java](java/problems/84-file-system-permissions/README.md) · [rust](rust/problems/84-file-system-permissions/README.md) | ☐ |
| Step 4 | [Text Editor with Undo/Redo](java/problems/85-text-editor-undo/README.md) **new** | Medium | 60 min | command pattern vs snapshots, undo/redo stacks, operation grouping (coalescing), cursor and selection model, state restoration | [java](java/problems/85-text-editor-undo/README.md) · [rust](rust/problems/85-text-editor-undo/README.md) | ☐ |
| Step 5 | [Document Versioning with Merge](java/problems/86-document-versioning/README.md) **new** | Hard | 75 min | append-only history, optimistic concurrency, line diff (LCS), three-way merge, conflict detection, compare-and-append under a lock | [java](java/problems/86-document-versioning/README.md) · [rust](rust/problems/86-document-versioning/README.md) | ☐ |

### Stage 3 - Money and time

*Balances that must add up, and calendars/schedules driven by an injected clock.*

|---|---|---|---:|---|---|:---:|
| Step 6 | [Splitwise](java/problems/15-splitwise/README.md) | Medium | 60 min | balances, split strategies | [java](java/problems/15-splitwise/README.md) · [rust](rust/problems/15-splitwise/README.md) | ☐ |
| Step 7 | [Expense Groups (Splitwise)](java/problems/87-expense-groups/README.md) **new** | Hard | 75 min | ledger and net balances, fair integer splitting, atomic validation, debt minimisation (zero-sum partition), bitmask DP, money as integers | [java](java/problems/87-expense-groups/README.md) · [rust](rust/problems/87-expense-groups/README.md) | ☐ |
| Step 8 | [Shared Event Calendar](java/problems/88-event-calendar/README.md) **new** | Medium-Hard | 60 min | interval overlap on half-open ranges, all-or-nothing multi-party booking, ordered multi-lock acquisition, per-user sorted structures, free-slot computation | [java](java/problems/88-event-calendar/README.md) · [rust](rust/problems/88-event-calendar/README.md) | ☐ |
| Step 9 | [Cron Job Scheduler](java/problems/93-cron-job-scheduler/README.md) **new** | Hard | 75 min | cron expression parsing, next-slot computation over a calendar, retry with capped exponential backoff, deterministic polling scheduler, misfire policy, dead letters | [java](java/problems/93-cron-job-scheduler/README.md) · [rust](rust/problems/93-cron-job-scheduler/README.md) | ☐ |

### Stage 4 - Marketplaces and bookings

*Several actors competing for limited stock, with holds, expiry and state machines.*

|---|---|---|---:|---|---|:---:|
| Step 10 | [Hotel Booking](java/problems/11-hotel-booking/README.md) | Hard | 90 min | availability, overlapping ranges | [java](java/problems/11-hotel-booking/README.md) · [rust](rust/problems/11-hotel-booking/README.md) | ☐ |
| Step 11 | [Movie Ticket Booking](java/problems/89-movie-ticketing/README.md) **new** | Hard | 75 min | seat inventory, time-limited holds with a lazily evaluated clock, hold-to-booking state machine, atomic multi-seat reservation, tiered pricing, refund policy | [java](java/problems/89-movie-ticketing/README.md) · [rust](rust/problems/89-movie-ticketing/README.md) | ☐ |
| Step 12 | [Online Auction](java/problems/91-online-auction/README.md) **new** | Medium-Hard | 60 min | compare-and-update on a single highest bid, anti-sniping time extension with an injected clock, lazy auction closing, outbid notifications, reserve price | [java](java/problems/91-online-auction/README.md) · [rust](rust/problems/91-online-auction/README.md) | ☐ |
| Step 13 | [Food Delivery Dispatch](java/problems/90-food-delivery/README.md) **new** | Hard | 75 min | order state machine, all-or-nothing stock reservation, nearest-resource assignment with deterministic ties, FIFO waiting queue, compensation on cancel | [java](java/problems/90-food-delivery/README.md) · [rust](rust/problems/90-food-delivery/README.md) | ☐ |
| Step 14 | [Chat System](java/problems/92-chat-system/README.md) **new** | Hard | 75 min | per-recipient delivery state, offline queues, group membership over time, total order per chat, status aggregation, read receipts | [java](java/problems/92-chat-system/README.md) · [rust](rust/problems/92-chat-system/README.md) | ☐ |

### Stage 5 - Infrastructure products

*Systems other engineers build on: request pipelines and a query engine.*

|---|---|---|---:|---|---|:---:|
| Step 15 | [API Gateway](java/problems/94-api-gateway/README.md) **new** | Hard | 75 min | request pipeline, longest-prefix routing, token-bucket rate limiting with lazy refill, round-robin with failover, circuit-breaker state machine, injectable clock, per-key locking without serialising backend calls | [java](java/problems/94-api-gateway/README.md) · [rust](rust/problems/94-api-gateway/README.md) | ☐ |
| Step 16 | [Mini SQL Engine](java/problems/95-sql-engine/README.md) **new** | Hard | 90 min | tokenizer and recursive-descent parser, typed in-memory tables, expression evaluation with AND/OR precedence, hash join, grouping and aggregation, primary-key index, statement atomicity, serialised execution | [java](java/problems/95-sql-engine/README.md) · [rust](rust/problems/95-sql-engine/README.md) | ☐ |

## Track 4 - Distributed systems (in-process simulations)

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

|---|---|---|---:|---|---|:---:|
| Step 4 | [Gossip Protocol with Failure Detection](java/problems/100-gossip-protocol/README.md) **new** | Hard | 90 min | epidemic dissemination, push-pull anti-entropy, versioned last-writer-wins merge, heartbeat failure detection, deterministic simulation | [java](java/problems/100-gossip-protocol/README.md) · [rust](rust/problems/100-gossip-protocol/README.md) | ☐ |
| Step 5 | [CRDT Replicated Store](java/problems/103-crdt-replication/README.md) **new** | Hard | 75 min | state-based CRDTs, join semilattices, PN-counter, observed-remove set, last-writer-wins register, merge laws (idempotent, commutative, associative), snapshot serialization | [java](java/problems/103-crdt-replication/README.md) · [rust](rust/problems/103-crdt-replication/README.md) | ☐ |
| Step 6 | [Quorum Key-Value Store](java/problems/104-quorum-kv/README.md) **new** | Hard | 90 min | Dynamo-style leaderless replication, majority quorums (W + R > N), versioned writes, read repair, request idempotency, timeouts and retries in a simulated network | [java](java/problems/104-quorum-kv/README.md) · [rust](rust/problems/104-quorum-kv/README.md) | ☐ |

### Stage 3 - Agreement

*One leader, one log, same order everywhere.*

|---|---|---|---:|---|---|:---:|
| Step 7 | [Raft Leader Election](java/problems/99-leader-election/README.md) **new** | Hard | 90 min | Raft leader election, terms, randomized election timeouts, vote granting rules, heartbeats, check-quorum step-down, split-brain safety, deterministic simulation under crashes, partitions and message loss | [java](java/problems/99-leader-election/README.md) · [rust](rust/problems/99-leader-election/README.md) | ☐ |
| Step 8 | [Raft Log Replication](java/problems/105-raft-log/README.md) **new** | Hard | 120 min | Raft consensus, leader election with the election restriction, replicated log with consistency check, commit rule (own-term entries), check-quorum, safety invariants under crashes, partitions and message duplication | [java](java/problems/105-raft-log/README.md) · [rust](rust/problems/105-raft-log/README.md) | ☐ |

### Stage 4 - Transactions across services

*All-or-nothing across several owners: atomic commit and compensation.*

|---|---|---|---:|---|---|:---:|
| Step 9 | [Two-Phase Commit](java/problems/101-two-phase-commit/README.md) **new** | Hard | 120 min | atomic commit, coordinator/participant protocol, write-ahead logging, in-doubt state and blocking, presumed abort, recovery from a durable log | [java](java/problems/101-two-phase-commit/README.md) · [rust](rust/problems/101-two-phase-commit/README.md) | ☐ |
| Step 10 | [Saga Orchestrator](java/problems/102-saga-orchestrator/README.md) **new** | Hard | 90 min | sagas and compensation, retry with exponential backoff, write-ahead progress log, crash recovery and resumption, idempotency, per-saga mutual exclusion | [java](java/problems/102-saga-orchestrator/README.md) · [rust](rust/problems/102-saga-orchestrator/README.md) | ☐ |

## Track 5 - Microsoft LLD round questions

Problems modelled on the Low Level Design questions reported for Microsoft interviews (several are DSA-flavoured: caches, autofill, search, schedulers). Do them in this order; each is Level 2 (thin entry point, your design) and has a Java and a Rust version.

| Step | Problem | Difficulty | Time | Focus | Links | Done |
|---|---|---|---|---|---|---|
| Step 1 | [Job Scheduler (Capability Matching)](java/problems/106-job-scheduler/README.md) **new** | Medium | 60 min | machines with capability sets, pluggable machine-selection criteria, counters on completion, deterministic tie-break | [java](java/problems/106-job-scheduler/README.md) · [rust](rust/problems/106-job-scheduler/README.md) | ☐ |
| Step 2 | [Excel-Like Autofill Engine](java/problems/107-excel-autofill/README.md) **new** | Medium | 45 min | numeric and letter series detection, case and sign preservation, number formatting, extensible series kinds | [java](java/problems/107-excel-autofill/README.md) · [rust](rust/problems/107-excel-autofill/README.md) | ☐ |
| Step 3 | [Document Comment Service](java/problems/108-document-comments/README.md) **new** | Easy-Medium | 30 min | insertion-ordered comments, unknown-document rejection, concurrent adds | [java](java/problems/108-document-comments/README.md) · [rust](rust/problems/108-document-comments/README.md) | ☐ |
| Step 4 | [Document Line Search](java/problems/109-document-line-search/README.md) **new** | Medium | 45 min | whole-word case-insensitive AND search, stable line ids across deletion, repeated-search efficiency | [java](java/problems/109-document-line-search/README.md) · [rust](rust/problems/109-document-line-search/README.md) | ☐ |
| Step 5 | [Runtime Data Source Access Layer](java/problems/110-data-source-layer/README.md) **new** | Medium | 45 min | one stable interface over switchable sources with independent records, source-specific rules | [java](java/problems/110-data-source-layer/README.md) · [rust](rust/problems/110-data-source-layer/README.md) | ☐ |
| Step 6 | [Configuration Management Service](java/problems/111-config-management/README.md) **new** | Medium | 60 min | versioned records, optimistic updates, history and rollback, validation before mutation | [java](java/problems/111-config-management/README.md) · [rust](rust/problems/111-config-management/README.md) | ☐ |
| Step 7 | [Parking Lot with Allocation Strategies](java/problems/112-parking-lot-strategies/README.md) **new** | Medium-Hard | 75 min | two/four-wheeler spots, lowest-index vs most-free-floor allocation, lookup by registration or ticket | [java](java/problems/112-parking-lot-strategies/README.md) · [rust](rust/problems/112-parking-lot-strategies/README.md) | ☐ |
| Step 8 | [Multi-Module Logging Framework](java/problems/113-multi-module-logging/README.md) **new** | Hard | 90 min | shared buffer, acknowledge after successful write, sequence-range retrieval, producer-consumer coordination | [java](java/problems/113-multi-module-logging/README.md) · [rust](rust/problems/113-multi-module-logging/README.md) | ☐ |
| Step 9 | [Elevator Request Feasibility](java/problems/114-elevator-feasibility/README.md) **new** | Hard | 90 min | accept/reject against capacity and per-rider stop limits without breaking earlier promises | [java](java/problems/114-elevator-feasibility/README.md) · [rust](rust/problems/114-elevator-feasibility/README.md) | ☐ |

Also practise the existing [Parking Lot](java/problems/01-parking-lot/README.md), [Vending Machine](java/problems/05-vending-machine/README.md), [Notification System](java/problems/23-notification-system/README.md), [Logging Framework](java/problems/25-logging-framework/README.md), [Elevator](java/problems/02-elevator/README.md) and [LRU Cache](java/problems/18-lru-cache/README.md) problems, which these interviews also ask.

### 115-119 Search, typeahead and RAG

| # | Problem | Difficulty | Time | Level |
|---|---|---|---|---|
| 115 | [Autocomplete Typeahead Service](java/problems/115-autocomplete-typeahead/README.md) | Hard | 75 min | L2 |
| 116 | [In-Memory Search Engine](java/problems/116-inverted-index-search/README.md) | Hard | 90 min | L2 |
| 117 | [Search Query Pipeline](java/problems/117-query-pipeline/README.md) | Medium-Hard | 60 min | L2 |
| 118 | [Search Result Cache](java/problems/118-search-result-cache/README.md) | Hard | 75 min | L2 |
| 119 | [Mini RAG Retrieval Component](java/problems/119-mini-rag-retriever/README.md) | Hard | 90 min | L2 |
| Step 10 | [Snake Game With Food And Score](java/problems/123-snake-game/README.md) **new** | Medium | 45 min | grid movement rules, growth on food, wall/self collision, tail-vacating move, scale | [java](java/problems/123-snake-game/README.md) · [rust](rust/problems/123-snake-game/README.md) | ☐ |
| Step 11 | [Kafka-Like Message Streaming Service](java/problems/124-kafka-streaming/README.md) **new** | Hard | 90 min | partitions and offsets, key hashing, consumer groups, range rebalance, commits, retention | [java](java/problems/124-kafka-streaming/README.md) · [rust](rust/problems/124-kafka-streaming/README.md) | ☐ |
| Step 12 | [Music Streaming and Search](java/problems/125-music-streaming/README.md) **new** | Medium-Hard | 75 min | prefix search ranking, play counts, playlists, queue, recommendations, concurrent counters | [java](java/problems/125-music-streaming/README.md) · [rust](rust/problems/125-music-streaming/README.md) | ☐ |
| Step 13 | [Alert Monitoring System](java/problems/126-alert-monitoring/README.md) **new** | Medium-Hard | 75 min | windowed aggregates, OK/PENDING/FIRING/RESOLVED state machine, cooldown, channel routing | [java](java/problems/126-alert-monitoring/README.md) · [rust](rust/problems/126-alert-monitoring/README.md) | ☐ |
| Step 14 | [Browser History Manager](java/problems/127-browser-history/README.md) **new** | Medium | 45 min | tabs with back/forward, global history queries, most visited, capacity eviction | [java](java/problems/127-browser-history/README.md) · [rust](rust/problems/127-browser-history/README.md) | ☐ |
| Step 15 | [Local Search with Ranking](java/problems/128-local-search-ranking/README.md) **new** | Medium-Hard | 60 min | filters plus an exact integer ranking formula, ties, atomic updates, scale | [java](java/problems/128-local-search-ranking/README.md) · [rust](rust/problems/128-local-search-ranking/README.md) | ☐ |
| Step 16 | [Job Posting with Candidate Matching](java/problems/129-job-candidate-matching/README.md) **new** | Medium-Hard | 60 min | eligibility rules, an exact match score, ranking both ways, applications and closing jobs | [java](java/problems/129-job-candidate-matching/README.md) · [rust](rust/problems/129-job-candidate-matching/README.md) | ☐ |
| Step 17 | [Ludo](java/problems/130-ludo/README.md) **new** | Hard | 90 min | dice-driven turns, captures, safe cells and blocks, home column, three-sixes rule | [java](java/problems/130-ludo/README.md) · [rust](rust/problems/130-ludo/README.md) | ☐ |
| Step 18 | [Shortest Distance and Nearest Landmarks](java/problems/131-landmark-distances/README.md) **new** | Medium | 60 min | weighted graph shortest paths, lexicographic tie-break, nearest landmarks, updates | [java](java/problems/131-landmark-distances/README.md) · [rust](rust/problems/131-landmark-distances/README.md) | ☐ |
| Step 19 | [Word Processor](java/problems/132-word-processor/README.md) **new** | Medium-Hard | 75 min | rich-text runs, shared format values, find/replace, paragraph alignment, million-character scale | [java](java/problems/132-word-processor/README.md) · [rust](rust/problems/132-word-processor/README.md) | ☐ |
| Step 20 | [Spreadsheet Grid](java/problems/133-spreadsheet-grid/README.md) **new** | Medium-Hard | 75 min | cell values, shared styles, copy/move/sort, row and column insertion, sparse scale | [java](java/problems/133-spreadsheet-grid/README.md) · [rust](rust/problems/133-spreadsheet-grid/README.md) | ☐ |
| Step 21 | [Dictionary App](java/problems/134-dictionary-app/README.md) **new** | Medium | 45 min | words with multiple meanings, synonyms, prefix and substring search, edit-distance suggestions | [java](java/problems/134-dictionary-app/README.md) · [rust](rust/problems/134-dictionary-app/README.md) | ☐ |
| Step 22 | [Container Orchestrator](java/problems/135-container-orchestrator/README.md) **new** | Hard | 90 min | fit strategies, preemption by priority, node failure rescheduling, no over-commit | [java](java/problems/135-container-orchestrator/README.md) · [rust](rust/problems/135-container-orchestrator/README.md) | ☐ |
| Step 23 | [Fantasy Team Leaderboard](java/problems/136-fantasy-leaderboard/README.md) **new** | Medium-Hard | 60 min | roster validation, exact half-point scoring, competition ranking, corrections, concurrency | [java](java/problems/136-fantasy-leaderboard/README.md) · [rust](rust/problems/136-fantasy-leaderboard/README.md) | ☐ |
| Step 24 | [Configurable Time Window Hit Counter](java/problems/137-hit-counter-window/README.md) **new** | Medium | 45 min | time windows, expiry boundary, shrinking and growing windows, bounded memory, clock clamping | [java](java/problems/137-hit-counter-window/README.md) · [rust](rust/problems/137-hit-counter-window/README.md) | ☐ |
| Step 25 | [Multi-Threaded Page Visit Counter](java/problems/138-page-visit-counter/README.md) **new** | Medium | 45 min | URL normalisation, exact concurrent totals and uniques, consistent snapshots, memory bound | [java](java/problems/138-page-visit-counter/README.md) · [rust](rust/problems/138-page-visit-counter/README.md) | ☐ |
| Step 26 | [Shopping Cart](java/problems/139-shopping-cart/README.md) **new** | Medium | 60 min | stock, promotions, coupons, tax allocation, atomic checkout | [java](java/problems/139-shopping-cart/README.md) · [rust](rust/problems/139-shopping-cart/README.md) | ☐ |
| Step 27 | [Custom HashMap](java/problems/140-custom-hashmap/README.md) **new** | Medium | 60 min | hasher-driven buckets, exact resize rule, insertion-ordered iteration, collisions | [java](java/problems/140-custom-hashmap/README.md) · [rust](rust/problems/140-custom-hashmap/README.md) | ☐ |
| Step 28 | [Matrix Traversal Strategies](java/problems/141-matrix-traversal/README.md) **new** | Medium | 45 min | nine traversal orders, lazy cursors, user-defined strategies, huge virtual matrices | [java](java/problems/141-matrix-traversal/README.md) · [rust](rust/problems/141-matrix-traversal/README.md) | ☐ |
| Step 29 | [Restaurant Customer Tokens](java/problems/142-restaurant-tokens/README.md) **new** | Medium | 45 min | priority queue with anti-starvation, no-shows and recall, wait estimates | [java](java/problems/142-restaurant-tokens/README.md) · [rust](rust/problems/142-restaurant-tokens/README.md) | ☐ |
| Step 30 | [Excel Cell Values and Sum Formulas](java/problems/143-excel-formulas/README.md) **new** | Hard | 90 min | parser and evaluator, SUM ranges, dependency recompute counts, cycles and error values | [java](java/problems/143-excel-formulas/README.md) · [rust](rust/problems/143-excel-formulas/README.md) | ☐ |

### Existing problems also asked in Microsoft LLD rounds

These live in other tracks; they carry the **Microsoft LLD** tag in the [catalogue](#full-catalogue).

| Question | Problem here |
|---|---|
| Parking Lot | [Parking Lot](java/problems/01-parking-lot/README.md) |
| Elevator System | [Elevator System](java/problems/02-elevator/README.md) |
| Vending Machine (exact payment is one of its scenarios) | [Vending Machine](java/problems/05-vending-machine/README.md) |
| Chess | [Chess](java/problems/07-chess/README.md) |
| Tic-Tac-Toe | [Tic-Tac-Toe](java/problems/08-tic-tac-toe/README.md) |
| Snake and Ladder | [Snake and Ladder](java/problems/09-snake-and-ladder/README.md) |
| Rate Limiter | [Rate Limiter](java/problems/20-rate-limiter/README.md) |
| Notification System | [Notification System](java/problems/23-notification-system/README.md) |
| Logging Framework | [Logging Framework](java/problems/25-logging-framework/README.md) |
| Text Editor with Undo and Redo | [Text Editor with Undo/Redo](java/problems/85-text-editor-undo/README.md) |
| Movie Ticket Booking | [Movie Ticket Booking](java/problems/89-movie-ticketing/README.md) |
| Autocomplete (Google-like search autocomplete) | [Autocomplete / Typeahead Service](java/problems/115-autocomplete-typeahead/README.md) |
| Parking Lot with strategies, multi-threaded | [Parking Lot](java/problems/01-parking-lot/README.md), [Parking Lot with Allocation Strategies](java/problems/112-parking-lot-strategies/README.md) |
| Single-lift elevator and request feasibility | [Elevator System](java/problems/02-elevator/README.md), [Elevator Request Feasibility](java/problems/114-elevator-feasibility/README.md) |


## Track 6 - Search, typeahead and RAG

Search-system questions that interviews often build on a first-round design: typeahead, an inverted-index engine, a query pipeline, a result cache and a small retrieval-augmented-generation component. Each is Level 2 with a Java and a Rust version. Suggested order:

| Step | Problem | Difficulty | Time | Focus | Links | Done |
|---|---|---|---|---|---|---|
| Step 1 | [Autocomplete Typeahead Service](java/problems/115-autocomplete-typeahead/README.md) **new** | Hard | 75 min | prefix lookup, frequency/recency/personalised ranking, deterministic ties, concurrent updates, scale | [java](java/problems/115-autocomplete-typeahead/README.md) · [rust](rust/problems/115-autocomplete-typeahead/README.md) | ☐ |
| Step 2 | [In-Memory Search Engine](java/problems/116-inverted-index-search/README.md) **new** | Hard | 90 min | tokenisation, AND/OR queries, TF-IDF ranking, delete/update, pagination | [java](java/problems/116-inverted-index-search/README.md) · [rust](rust/problems/116-inverted-index-search/README.md) | ☐ |
| Step 3 | [Search Query Pipeline](java/problems/117-query-pipeline/README.md) **new** | Medium-Hard | 60 min | parse, spell-correct, synonyms, filter, rank, highlight; adding a step without touching the others | [java](java/problems/117-query-pipeline/README.md) · [rust](rust/problems/117-query-pipeline/README.md) | ☐ |
| Step 4 | [Search Result Cache](java/problems/118-search-result-cache/README.md) **new** | Hard | 75 min | normalised query keys, TTL and LRU, invalidation on index change, request coalescing | [java](java/problems/118-search-result-cache/README.md) · [rust](rust/problems/118-search-result-cache/README.md) | ☐ |
| Step 5 | [Mini RAG Retrieval Component](java/problems/119-mini-rag-retriever/README.md) **new** | Hard | 90 min | chunking, swappable embedder and vector store, cosine retrieval, budgeted prompt building | [java](java/problems/119-mini-rag-retriever/README.md) · [rust](rust/problems/119-mini-rag-retriever/README.md) | ☐ |

### 120-122 Concurrent data structures

| # | Problem | Difficulty | Time | Level |
|---|---|---|---|---|
| 120 | [Thread-Safe Trie](java/problems/120-thread-safe-trie/README.md) | Hard | 75 min | L2 |
| 121 | [Concurrent Bloom Filter](java/problems/121-concurrent-bloom-filter/README.md) | Hard | 60 min | L2 |
| 122 | [Versioned Value Store](java/problems/122-versioned-value-store/README.md) | Hard | 75 min | L2 |

## Full catalogue

Every problem, in folder order. The **Microsoft LLD** tag marks questions reported for Microsoft LLD rounds. Level: L1 almost empty, L2 contract provided, L3 contract plus test infrastructure.

| # | Problem | Difficulty | Time | Level | Track | Tags | Code |
|--:|---|---|---:|:---:|---|---|---|
| 01 | Parking Lot | Medium-Hard | 90 min | L1 | Foundations | Microsoft LLD | [java](java/problems/01-parking-lot/README.md) · [rust](rust/problems/01-parking-lot/README.md) |
| 02 | Elevator System | Hard | 90 min | L1 | Foundations | Microsoft LLD | [java](java/problems/02-elevator/README.md) · [rust](rust/problems/02-elevator/README.md) |
| 03 | Library Management | Medium | 60 min | L1 | Foundations |  | [java](java/problems/03-library/README.md) · [rust](rust/problems/03-library/README.md) |
| 04 | Coffee Machine | Medium | 60 min | L1 | Foundations |  | [java](java/problems/04-coffee-machine/README.md) · [rust](rust/problems/04-coffee-machine/README.md) |
| 05 | Vending Machine | Medium | 60 min | L1 | Foundations | Microsoft LLD | [java](java/problems/05-vending-machine/README.md) · [rust](rust/problems/05-vending-machine/README.md) |
| 06 | ATM | Medium | 60 min | L1 | Foundations |  | [java](java/problems/06-atm/README.md) · [rust](rust/problems/06-atm/README.md) |
| 07 | Chess | Senior | 120 min | L1 | Foundations | Microsoft LLD | [java](java/problems/07-chess/README.md) · [rust](rust/problems/07-chess/README.md) |
| 08 | Tic-Tac-Toe | Medium | 45 min | L1 | Foundations | Microsoft LLD | [java](java/problems/08-tic-tac-toe/README.md) · [rust](rust/problems/08-tic-tac-toe/README.md) |
| 09 | Snake and Ladder | Medium | 45 min | L1 | Foundations | Microsoft LLD | [java](java/problems/09-snake-and-ladder/README.md) · [rust](rust/problems/09-snake-and-ladder/README.md) |
| 10 | Flight Booking | Hard | 90 min | L1 | Foundations |  | [java](java/problems/10-flight-booking/README.md) · [rust](rust/problems/10-flight-booking/README.md) |
| 11 | Hotel Booking | Hard | 90 min | L1 | Foundations (+ Product systems) |  | [java](java/problems/11-hotel-booking/README.md) · [rust](rust/problems/11-hotel-booking/README.md) |
| 12 | Travel Itinerary Planner | Medium-Hard | 90 min | L1 | Foundations |  | [java](java/problems/12-travel-itinerary/README.md) · [rust](rust/problems/12-travel-itinerary/README.md) |
| 13 | Bus / Train Seat Booking | Medium-Hard | 90 min | L1 | Foundations (+ Concurrency) |  | [java](java/problems/13-seat-booking/README.md) · [rust](rust/problems/13-seat-booking/README.md) |
| 14 | Ride Sharing | Medium-Hard | 90 min | L1 | Foundations |  | [java](java/problems/14-ride-sharing/README.md) · [rust](rust/problems/14-ride-sharing/README.md) |
| 15 | Splitwise | Medium | 60 min | L1 | Foundations (+ Product systems) |  | [java](java/problems/15-splitwise/README.md) · [rust](rust/problems/15-splitwise/README.md) |
| 16 | Digital Wallet | Medium-Hard | 90 min | L1 | Foundations |  | [java](java/problems/16-wallet/README.md) · [rust](rust/problems/16-wallet/README.md) |
| 17 | Stock Exchange Order Book | Hard | 120 min | L1 | Foundations |  | [java](java/problems/17-order-book/README.md) · [rust](rust/problems/17-order-book/README.md) |
| 18 | LRU Cache | Medium | 45 min | L2 | Foundations |  | [java](java/problems/18-lru-cache/README.md) · [rust](rust/problems/18-lru-cache/README.md) |
| 19 | LFU Cache | Medium-Hard | 60 min | L2 | Foundations |  | [java](java/problems/19-lfu-cache/README.md) · [rust](rust/problems/19-lfu-cache/README.md) |
| 20 | Rate Limiter | Medium-Hard | 60 min | L3 | Foundations (+ Concurrency) | Microsoft LLD | [java](java/problems/20-rate-limiter/README.md) · [rust](rust/problems/20-rate-limiter/README.md) |
| 21 | In-Memory Key-Value Store | Medium-Hard | 90 min | L1 | Foundations |  | [java](java/problems/21-key-value-store/README.md) · [rust](rust/problems/21-key-value-store/README.md) |
| 22 | Pub-Sub Message Broker | Medium-Hard | 90 min | L1 | Foundations (+ Concurrency) |  | [java](java/problems/22-pub-sub/README.md) · [rust](rust/problems/22-pub-sub/README.md) |
| 23 | Notification System | Medium-Hard | 90 min | L2 | Foundations | Microsoft LLD | [java](java/problems/23-notification-system/README.md) · [rust](rust/problems/23-notification-system/README.md) |
| 24 | Task Scheduler | Hard | 90 min | L3 | Foundations (+ Concurrency) |  | [java](java/problems/24-task-scheduler/README.md) · [rust](rust/problems/24-task-scheduler/README.md) |
| 25 | Logging Framework | Medium-Hard | 90 min | L2 | Foundations | Microsoft LLD | [java](java/problems/25-logging-framework/README.md) · [rust](rust/problems/25-logging-framework/README.md) |
| 26 | Metrics Collector | Medium-Hard | 60 min | L1 | Foundations |  | [java](java/problems/26-metrics/README.md) · [rust](rust/problems/26-metrics/README.md) |
| 27 | Circuit Breaker | Medium-Hard | 60 min | L3 | Foundations (+ Concurrency) |  | [java](java/problems/27-circuit-breaker/README.md) · [rust](rust/problems/27-circuit-breaker/README.md) |
| 28 | Bounded Blocking Buffer | Hard | 90 min | L3 | Foundations (+ Concurrency) |  | [java](java/problems/28-blocking-queue/README.md) · [rust](rust/problems/28-blocking-queue/README.md) |
| 29 | Worker Pool | Hard | 90 min | L3 | Foundations (+ Concurrency) |  | [java](java/problems/29-worker-pool/README.md) · [rust](rust/problems/29-worker-pool/README.md) |
| 30 | Concurrent Cache with Loaders | Hard | 90 min | L3 | Foundations (+ Concurrency) |  | [java](java/problems/30-concurrent-cache/README.md) · [rust](rust/problems/30-concurrent-cache/README.md) |
| 31 | In-Process Event Bus | Hard | 120 min | L3 | Foundations (+ Concurrency) |  | [java](java/problems/31-event-bus/README.md) · [rust](rust/problems/31-event-bus/README.md) |
| 32 | Board Game Engine | Hard | 120 min | L1 | Foundations |  | [java](java/problems/32-board-game-engine/README.md) · [rust](rust/problems/32-board-game-engine/README.md) |
| 33 | Classes and Objects | Easy | 30 min | L2 | Foundations |  | [java](java/problems/33-classes-objects/README.md) · [rust](rust/problems/33-classes-objects/README.md) |
| 34 | Interfaces | Easy | 30 min | L2 | Foundations |  | [java](java/problems/34-interfaces/README.md) · [rust](rust/problems/34-interfaces/README.md) |
| 35 | Inheritance | Easy | 30 min | L2 | Foundations |  | [java](java/problems/35-inheritance/README.md) · [rust](rust/problems/35-inheritance/README.md) |
| 36 | Polymorphism | Easy | 30 min | L2 | Foundations |  | [java](java/problems/36-polymorphism/README.md) · [rust](rust/problems/36-polymorphism/README.md) |
| 37 | Abstraction | Medium | 30 min | L2 | Foundations |  | [java](java/problems/37-abstraction/README.md) · [rust](rust/problems/37-abstraction/README.md) |
| 38 | Encapsulation | Medium | 30 min | L2 | Foundations |  | [java](java/problems/38-encapsulation/README.md) · [rust](rust/problems/38-encapsulation/README.md) |
| 39 | Aggregation | Easy | 30 min | L2 | Foundations |  | [java](java/problems/39-aggregation/README.md) · [rust](rust/problems/39-aggregation/README.md) |
| 40 | Composition | Medium | 40 min | L2 | Foundations |  | [java](java/problems/40-composition/README.md) · [rust](rust/problems/40-composition/README.md) |
| 41 | Association | Medium | 40 min | L2 | Foundations |  | [java](java/problems/41-association/README.md) · [rust](rust/problems/41-association/README.md) |
| 42 | DRY — Don't Repeat Yourself | Medium | 30 min | L2 | Foundations |  | [java](java/problems/42-dry/README.md) · [rust](rust/problems/42-dry/README.md) |
| 43 | KISS — Keep It Simple | Easy | 20 min | L2 | Foundations |  | [java](java/problems/43-kiss/README.md) · [rust](rust/problems/43-kiss/README.md) |
| 44 | YAGNI — You Aren't Gonna Need It | Easy | 25 min | L2 | Foundations |  | [java](java/problems/44-yagni/README.md) · [rust](rust/problems/44-yagni/README.md) |
| 45 | Law of Demeter | Medium | 40 min | L2 | Foundations |  | [java](java/problems/45-law-of-demeter/README.md) · [rust](rust/problems/45-law-of-demeter/README.md) |
| 46 | SRP — Single Responsibility Principle | Medium | 40 min | L2 | Foundations |  | [java](java/problems/46-srp/README.md) · [rust](rust/problems/46-srp/README.md) |
| 47 | OCP — Open/Closed Principle | Medium | 35 min | L2 | Foundations |  | [java](java/problems/47-ocp/README.md) · [rust](rust/problems/47-ocp/README.md) |
| 48 | LSP — Liskov Substitution Principle | Medium | 40 min | L2 | Foundations |  | [java](java/problems/48-lsp/README.md) · [rust](rust/problems/48-lsp/README.md) |
| 49 | ISP — Interface Segregation Principle | Medium | 35 min | L2 | Foundations |  | [java](java/problems/49-isp/README.md) · [rust](rust/problems/49-isp/README.md) |
| 50 | DIP — Dependency Inversion Principle | Medium | 35 min | L2 | Foundations |  | [java](java/problems/50-dip/README.md) · [rust](rust/problems/50-dip/README.md) |
| 51 | SOLID Summary — Meal Order Checkout | Hard | 75 min | L2 | Foundations |  | [java](java/problems/51-solid-summary/README.md) · [rust](rust/problems/51-solid-summary/README.md) |
| 52 | Singleton | Medium | 30 min | L2 | Foundations |  | [java](java/problems/52-singleton/README.md) · [rust](rust/problems/52-singleton/README.md) |
| 53 | Factory Method | Medium | 35 min | L2 | Foundations |  | [java](java/problems/53-factory-method/README.md) · [rust](rust/problems/53-factory-method/README.md) |
| 54 | Abstract Factory | Medium | 45 min | L2 | Foundations |  | [java](java/problems/54-abstract-factory/README.md) · [rust](rust/problems/54-abstract-factory/README.md) |
| 55 | Builder | Medium | 45 min | L2 | Foundations |  | [java](java/problems/55-builder/README.md) · [rust](rust/problems/55-builder/README.md) |
| 56 | Prototype | Medium | 40 min | L2 | Foundations |  | [java](java/problems/56-prototype/README.md) · [rust](rust/problems/56-prototype/README.md) |
| 57 | Adapter | Medium | 35 min | L2 | Foundations |  | [java](java/problems/57-adapter/README.md) · [rust](rust/problems/57-adapter/README.md) |
| 58 | Facade | Medium | 35 min | L2 | Foundations |  | [java](java/problems/58-facade/README.md) · [rust](rust/problems/58-facade/README.md) |
| 59 | Decorator | Medium | 40 min | L2 | Foundations |  | [java](java/problems/59-decorator/README.md) · [rust](rust/problems/59-decorator/README.md) |
| 60 | Composite | Medium | 40 min | L2 | Foundations |  | [java](java/problems/60-composite/README.md) · [rust](rust/problems/60-composite/README.md) |
| 61 | Proxy | Medium | 45 min | L2 | Foundations |  | [java](java/problems/61-proxy/README.md) · [rust](rust/problems/61-proxy/README.md) |
| 62 | Bridge | Hard | 50 min | L2 | Foundations |  | [java](java/problems/62-bridge/README.md) · [rust](rust/problems/62-bridge/README.md) |
| 63 | Flyweight | Medium | 40 min | L2 | Foundations |  | [java](java/problems/63-flyweight/README.md) · [rust](rust/problems/63-flyweight/README.md) |
| 64 | Iterator | Medium | 45 min | L2 | Foundations |  | [java](java/problems/64-iterator/README.md) · [rust](rust/problems/64-iterator/README.md) |
| 65 | Observer | Medium | 40 min | L2 | Foundations |  | [java](java/problems/65-observer/README.md) · [rust](rust/problems/65-observer/README.md) |
| 66 | Strategy | Medium | 40 min | L2 | Foundations |  | [java](java/problems/66-strategy/README.md) · [rust](rust/problems/66-strategy/README.md) |
| 67 | Command | Hard | 50 min | L2 | Foundations |  | [java](java/problems/67-command/README.md) · [rust](rust/problems/67-command/README.md) |
| 68 | State | Hard | 60 min | L2 | Foundations |  | [java](java/problems/68-state/README.md) · [rust](rust/problems/68-state/README.md) |
| 69 | Template Method | Medium | 40 min | L2 | Foundations |  | [java](java/problems/69-template-method/README.md) · [rust](rust/problems/69-template-method/README.md) |
| 70 | Visitor | Hard | 50 min | L2 | Foundations |  | [java](java/problems/70-visitor/README.md) · [rust](rust/problems/70-visitor/README.md) |
| 71 | Mediator | Hard | 60 min | L2 | Foundations |  | [java](java/problems/71-mediator/README.md) · [rust](rust/problems/71-mediator/README.md) |
| 72 | Memento | Hard | 50 min | L2 | Foundations |  | [java](java/problems/72-memento/README.md) · [rust](rust/problems/72-memento/README.md) |
| 73 | Chain of Responsibility | Medium | 45 min | L2 | Foundations |  | [java](java/problems/73-chain-of-responsibility/README.md) · [rust](rust/problems/73-chain-of-responsibility/README.md) |
| 74 | Concurrent LRU Cache (Read-Write Lock) | Medium-Hard | 60 min | L3 | Concurrency |  | [java](java/problems/74-rw-lock-lru-cache/README.md) · [rust](rust/problems/74-rw-lock-lru-cache/README.md) |
| 75 | Bank Transfers Without Deadlock | Medium | 45 min | L3 | Concurrency |  | [java](java/problems/75-bank-transfer/README.md) · [rust](rust/problems/75-bank-transfer/README.md) |
| 76 | Ordered Printer (Turn Taking) | Medium | 45 min | L3 | Concurrency |  | [java](java/problems/76-ordered-printer/README.md) · [rust](rust/problems/76-ordered-printer/README.md) |
| 77 | Dining Philosophers | Medium-Hard | 60 min | L3 | Concurrency |  | [java](java/problems/77-dining-philosophers/README.md) · [rust](rust/problems/77-dining-philosophers/README.md) |
| 78 | Bounded Connection Pool | Medium-Hard | 60 min | L3 | Concurrency |  | [java](java/problems/78-connection-pool/README.md) · [rust](rust/problems/78-connection-pool/README.md) |
| 79 | Parallel Merge Sort | Medium-Hard | 60 min | L3 | Concurrency |  | [java](java/problems/79-parallel-merge-sort/README.md) · [rust](rust/problems/79-parallel-merge-sort/README.md) |
| 80 | Parallel Word Count | Medium | 45 min | L3 | Concurrency |  | [java](java/problems/80-parallel-word-count/README.md) · [rust](rust/problems/80-parallel-word-count/README.md) |
| 81 | Parallel Web Crawler | Hard | 90 min | L3 | Concurrency |  | [java](java/problems/81-parallel-crawler/README.md) · [rust](rust/problems/81-parallel-crawler/README.md) |
| 82 | URL Shortener | Medium | 45 min | L2 | Product systems |  | [java](java/problems/82-url-shortener/README.md) · [rust](rust/problems/82-url-shortener/README.md) |
| 83 | Feature Flag Service | Medium | 45 min | L2 | Product systems |  | [java](java/problems/83-feature-flags/README.md) · [rust](rust/problems/83-feature-flags/README.md) |
| 84 | In-Memory File System with Permissions | Medium-Hard | 75 min | L2 | Product systems |  | [java](java/problems/84-file-system-permissions/README.md) · [rust](rust/problems/84-file-system-permissions/README.md) |
| 85 | Text Editor with Undo/Redo | Medium | 60 min | L2 | Product systems | Microsoft LLD | [java](java/problems/85-text-editor-undo/README.md) · [rust](rust/problems/85-text-editor-undo/README.md) |
| 86 | Document Versioning with Merge | Hard | 75 min | L2 | Product systems |  | [java](java/problems/86-document-versioning/README.md) · [rust](rust/problems/86-document-versioning/README.md) |
| 87 | Expense Groups (Splitwise) | Hard | 75 min | L2 | Product systems |  | [java](java/problems/87-expense-groups/README.md) · [rust](rust/problems/87-expense-groups/README.md) |
| 88 | Shared Event Calendar | Medium-Hard | 60 min | L2 | Product systems |  | [java](java/problems/88-event-calendar/README.md) · [rust](rust/problems/88-event-calendar/README.md) |
| 89 | Movie Ticket Booking | Hard | 75 min | L2 | Product systems | Microsoft LLD | [java](java/problems/89-movie-ticketing/README.md) · [rust](rust/problems/89-movie-ticketing/README.md) |
| 90 | Food Delivery Dispatch | Hard | 75 min | L2 | Product systems |  | [java](java/problems/90-food-delivery/README.md) · [rust](rust/problems/90-food-delivery/README.md) |
| 91 | Online Auction | Medium-Hard | 60 min | L2 | Product systems |  | [java](java/problems/91-online-auction/README.md) · [rust](rust/problems/91-online-auction/README.md) |
| 92 | Chat System | Hard | 75 min | L2 | Product systems |  | [java](java/problems/92-chat-system/README.md) · [rust](rust/problems/92-chat-system/README.md) |
| 93 | Cron Job Scheduler | Hard | 75 min | L2 | Product systems |  | [java](java/problems/93-cron-job-scheduler/README.md) · [rust](rust/problems/93-cron-job-scheduler/README.md) |
| 94 | API Gateway | Hard | 75 min | L2 | Product systems |  | [java](java/problems/94-api-gateway/README.md) · [rust](rust/problems/94-api-gateway/README.md) |
| 95 | Mini SQL Engine | Hard | 90 min | L2 | Product systems |  | [java](java/problems/95-sql-engine/README.md) · [rust](rust/problems/95-sql-engine/README.md) |
| 96 | Consistent Hash Ring | Medium-Hard | 60 min | L2 | Distributed systems |  | [java](java/problems/96-consistent-hashing/README.md) · [rust](rust/problems/96-consistent-hashing/README.md) |
| 97 | Idempotent Request Processor | Medium-Hard | 60 min | L2 | Distributed systems |  | [java](java/problems/97-idempotent-requests/README.md) · [rust](rust/problems/97-idempotent-requests/README.md) |
| 98 | Lease Lock with Fencing Tokens | Medium-Hard | 50 min | L2 | Distributed systems |  | [java](java/problems/98-lock-lease-fencing/README.md) · [rust](rust/problems/98-lock-lease-fencing/README.md) |
| 99 | Raft Leader Election | Hard | 90 min | L3 | Distributed systems |  | [java](java/problems/99-leader-election/README.md) · [rust](rust/problems/99-leader-election/README.md) |
| 100 | Gossip Protocol with Failure Detection | Hard | 90 min | L3 | Distributed systems |  | [java](java/problems/100-gossip-protocol/README.md) · [rust](rust/problems/100-gossip-protocol/README.md) |
| 101 | Two-Phase Commit | Hard | 120 min | L3 | Distributed systems |  | [java](java/problems/101-two-phase-commit/README.md) · [rust](rust/problems/101-two-phase-commit/README.md) |
| 102 | Saga Orchestrator | Hard | 90 min | L3 | Distributed systems |  | [java](java/problems/102-saga-orchestrator/README.md) · [rust](rust/problems/102-saga-orchestrator/README.md) |
| 103 | CRDT Replicated Store | Hard | 75 min | L3 | Distributed systems |  | [java](java/problems/103-crdt-replication/README.md) · [rust](rust/problems/103-crdt-replication/README.md) |
| 104 | Quorum Key-Value Store | Hard | 90 min | L3 | Distributed systems |  | [java](java/problems/104-quorum-kv/README.md) · [rust](rust/problems/104-quorum-kv/README.md) |
| 105 | Raft Log Replication | Hard | 120 min | L3 | Distributed systems |  | [java](java/problems/105-raft-log/README.md) · [rust](rust/problems/105-raft-log/README.md) |
| 106 | Job Scheduler | Medium | 60 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/106-job-scheduler/README.md) · [rust](rust/problems/106-job-scheduler/README.md) |
| 107 | Excel-Like Autofill Engine | Medium | 45 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/107-excel-autofill/README.md) · [rust](rust/problems/107-excel-autofill/README.md) |
| 108 | Document Comment Service | Easy-Medium | 30 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/108-document-comments/README.md) · [rust](rust/problems/108-document-comments/README.md) |
| 109 | Document Line Search | Medium | 45 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/109-document-line-search/README.md) · [rust](rust/problems/109-document-line-search/README.md) |
| 110 | # 110 Runtime Data Source Access Layer | Medium | 45 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/110-data-source-layer/README.md) · [rust](rust/problems/110-data-source-layer/README.md) |
| 111 | Configuration Management Service | Medium | 60 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/111-config-management/README.md) · [rust](rust/problems/111-config-management/README.md) |
| 112 | Parking Lot with Allocation Strategies | Medium-Hard | 75 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/112-parking-lot-strategies/README.md) · [rust](rust/problems/112-parking-lot-strategies/README.md) |
| 113 | Multi-Module Logging Framework | Hard | 90 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/113-multi-module-logging/README.md) · [rust](rust/problems/113-multi-module-logging/README.md) |
| 114 | Elevator Request Feasibility | Hard | 90 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/114-elevator-feasibility/README.md) · [rust](rust/problems/114-elevator-feasibility/README.md) |
| 115 | Autocomplete / Typeahead Service | Hard | 75 min | L2 | Search and RAG (+ Concurrency) | Microsoft LLD | [java](java/problems/115-autocomplete-typeahead/README.md) · [rust](rust/problems/115-autocomplete-typeahead/README.md) |
| 116 | In-Memory Search Engine | Hard | 90 min | L2 | Search and RAG |  | [java](java/problems/116-inverted-index-search/README.md) · [rust](rust/problems/116-inverted-index-search/README.md) |
| 117 | Search Query Pipeline | Medium-Hard | 60 min | L2 | Search and RAG |  | [java](java/problems/117-query-pipeline/README.md) · [rust](rust/problems/117-query-pipeline/README.md) |
| 118 | Search Result Cache | Hard | 75 min | L2 | Search and RAG (+ Concurrency) |  | [java](java/problems/118-search-result-cache/README.md) · [rust](rust/problems/118-search-result-cache/README.md) |
| 119 | Mini RAG Retrieval Component | Hard | 90 min | L2 | Search and RAG |  | [java](java/problems/119-mini-rag-retriever/README.md) · [rust](rust/problems/119-mini-rag-retriever/README.md) |
| 120 | Thread-Safe Trie | Hard | 75 min | L2 | Concurrency |  | [java](java/problems/120-thread-safe-trie/README.md) · [rust](rust/problems/120-thread-safe-trie/README.md) |
| 121 | Concurrent Bloom Filter | Hard | 60 min | L2 | Concurrency |  | [java](java/problems/121-concurrent-bloom-filter/README.md) · [rust](rust/problems/121-concurrent-bloom-filter/README.md) |
| 122 | Versioned Value Store | Hard | 75 min | L2 | Concurrency |  | [java](java/problems/122-versioned-value-store/README.md) · [rust](rust/problems/122-versioned-value-store/README.md) |
| 123 | Snake Game With Food And Score | Medium | 45 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/123-snake-game/README.md) · [rust](rust/problems/123-snake-game/README.md) |
| 124 | Kafka-Like Message Streaming Service | Hard | 90 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/124-kafka-streaming/README.md) · [rust](rust/problems/124-kafka-streaming/README.md) |
| 125 | Music Streaming and Search | Medium-Hard | 75 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/125-music-streaming/README.md) · [rust](rust/problems/125-music-streaming/README.md) |
| 126 | Alert Monitoring System | Medium-Hard | 75 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/126-alert-monitoring/README.md) · [rust](rust/problems/126-alert-monitoring/README.md) |
| 127 | Browser History Manager | Medium | 45 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/127-browser-history/README.md) · [rust](rust/problems/127-browser-history/README.md) |
| 128 | Local Search with Ranking | Medium-Hard | 60 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/128-local-search-ranking/README.md) · [rust](rust/problems/128-local-search-ranking/README.md) |
| 129 | Job Posting with Candidate Matching | Medium-Hard | 60 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/129-job-candidate-matching/README.md) · [rust](rust/problems/129-job-candidate-matching/README.md) |
| 130 | Ludo | Hard | 90 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/130-ludo/README.md) · [rust](rust/problems/130-ludo/README.md) |
| 131 | Shortest Distance and Nearest Landmarks | Medium | 60 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/131-landmark-distances/README.md) · [rust](rust/problems/131-landmark-distances/README.md) |
| 132 | Word Processor | Medium-Hard | 75 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/132-word-processor/README.md) · [rust](rust/problems/132-word-processor/README.md) |
| 133 | Spreadsheet Grid | Medium-Hard | 75 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/133-spreadsheet-grid/README.md) · [rust](rust/problems/133-spreadsheet-grid/README.md) |
| 134 | Dictionary App | Medium | 45 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/134-dictionary-app/README.md) · [rust](rust/problems/134-dictionary-app/README.md) |
| 135 | Container Orchestrator | Hard | 90 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/135-container-orchestrator/README.md) · [rust](rust/problems/135-container-orchestrator/README.md) |
| 136 | Fantasy Team Leaderboard | Medium-Hard | 60 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/136-fantasy-leaderboard/README.md) · [rust](rust/problems/136-fantasy-leaderboard/README.md) |
| 137 | Configurable Time Window Hit Counter | Medium | 45 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/137-hit-counter-window/README.md) · [rust](rust/problems/137-hit-counter-window/README.md) |
| 138 | Multi-Threaded Page Visit Counter | Medium | 45 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/138-page-visit-counter/README.md) · [rust](rust/problems/138-page-visit-counter/README.md) |
| 139 | Shopping Cart | Medium | 60 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/139-shopping-cart/README.md) · [rust](rust/problems/139-shopping-cart/README.md) |
| 140 | Custom HashMap | Medium | 60 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/140-custom-hashmap/README.md) · [rust](rust/problems/140-custom-hashmap/README.md) |
| 141 | Matrix Traversal Strategies | Medium | 45 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/141-matrix-traversal/README.md) · [rust](rust/problems/141-matrix-traversal/README.md) |
| 142 | Restaurant Customer Tokens | Medium | 45 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/142-restaurant-tokens/README.md) · [rust](rust/problems/142-restaurant-tokens/README.md) |
| 143 | Excel Cell Values and Sum Formulas | Hard | 90 min | L2 | Microsoft LLD | Microsoft LLD | [java](java/problems/143-excel-formulas/README.md) · [rust](rust/problems/143-excel-formulas/README.md) |

### What the concept drills (33-73) practise

| # | Problem | What you practise |
|--:|---|---|
| 33 | [Classes and Objects](java/problems/33-classes-objects/README.md) | `Money` value object: state, equality vs identity, immutability. |
| 34 | [Interfaces](java/problems/34-interfaces/README.md) | IoT sensors behind a `Sensor` interface; the hub works with implementations it has never seen. |
| 35 | [Inheritance](java/problems/35-inheritance/README.md) | Payroll hierarchy that reuses a base class (Java) / trait defaults + composition (Rust). |
| 36 | [Polymorphism](java/problems/36-polymorphism/README.md) | Toll plaza that charges any `Vehicle` through one polymorphic call. |
| 37 | [Abstraction](java/problems/37-abstraction/README.md) | Bank accounts: an abstract base with shared rules and two abstract decisions. |
| 38 | [Encapsulation](java/problems/38-encapsulation/README.md) | Warehouse stock item whose invariants cannot be broken through the public API. |
| 39 | [Aggregation](java/problems/39-aggregation/README.md) | Playlists that share songs: the parts outlive the whole. |
| 40 | [Composition](java/problems/40-composition/README.md) | Order that owns its line items: the parts die with the owner. |
| 41 | [Association](java/problems/41-association/README.md) | Students and courses, many-to-many, kept consistent from both sides. |
| 42 | [DRY — Don't Repeat Yourself](java/problems/42-dry/README.md) | Quote, invoice and credit note sharing one pricing path and one money formatter. |
| 43 | [KISS — Keep It Simple](java/problems/43-kiss/README.md) | Library late fee in one small function; the tests enforce a simplicity budget. |
| 44 | [YAGNI — You Aren't Gonna Need It](java/problems/44-yagni/README.md) | URL shortener with exactly two operations; the tests reject speculative extras. |
| 45 | [Law of Demeter](java/problems/45-law-of-demeter/README.md) | Checkout and shipping services that never reach through their collaborators. |
| 46 | [SRP — Single Responsibility Principle](java/problems/46-srp/README.md) | Invoice issuing split into calculator, renderer, repository and a coordinating service. |
| 47 | [OCP — Open/Closed Principle](java/problems/47-ocp/README.md) | Parking-fee calculator extended with new vehicle policies without being modified. |
| 48 | [LSP — Liskov Substitution Principle](java/problems/48-lsp/README.md) | Coupons that all honour one contract, verified by a shared test suite. |
| 49 | [ISP — Interface Segregation Principle](java/problems/49-isp/README.md) | Printer / scanner / fax: small interfaces, devices implement only what they can do. |
| 50 | [DIP — Dependency Inversion Principle](java/problems/50-dip/README.md) | Appointment reminders with an injected sender, repository and clock. |
| 51 | [SOLID Summary — Meal Order Checkout](java/problems/51-solid-summary/README.md) | Meal-order checkout combining S, O, L, I and D in one mini-system. |
| 52 | [Singleton](java/problems/52-singleton/README.md) | Lazy, thread-safe application config with exactly one instance. |
| 53 | [Factory Method](java/problems/53-factory-method/README.md) | Checkout flows whose subclasses choose the payment gateway through a factory method. |
| 54 | [Abstract Factory](java/problems/54-abstract-factory/README.md) | Regional kits (US/EU/IN) creating consistent families of tax, price and address products. |
| 55 | [Builder](java/problems/55-builder/README.md) | Immutable HTTP request created by a validating fluent builder. |
| 56 | [Prototype](java/problems/56-prototype/README.md) | Server blueprints deep-cloned from a registry of prototypes. |
| 57 | [Adapter](java/problems/57-adapter/README.md) | Legacy XML / integer-code SMS gateway exposed as a clean `Notifier`. |
| 58 | [Facade](java/problems/58-facade/README.md) | Photo upload hiding scanner, resizer, storage and index behind one call, with compensation. |
| 59 | [Decorator](java/problems/59-decorator/README.md) | Message sinks (timestamp, redact, retry, count) stacked in any order. |
| 60 | [Composite](java/problems/60-composite/README.md) | File-system tree where files and folders share one interface. |
| 61 | [Proxy](java/problems/61-proxy/README.md) | Caching, lazy and access-control proxies around a document store. |
| 62 | [Bridge](java/problems/62-bridge/README.md) | Alerts x channels: any alert type can use any delivery channel. |
| 63 | [Flyweight](java/problems/63-flyweight/README.md) | Map markers sharing icon flyweights from a thread-safe factory. |
| 64 | [Iterator](java/problems/64-iterator/README.md) | Lazy paginated feed iterator and a lazy filter iterator. |
| 65 | [Observer](java/problems/65-observer/README.md) | Price feed with subscribe/cancel, snapshot semantics and failure isolation. |
| 66 | [Strategy](java/problems/66-strategy/README.md) | Interchangeable shipping-cost strategies, swapped at runtime. |
| 67 | [Command](java/problems/67-command/README.md) | Text-editor commands with macros and an undo/redo history. |
| 68 | [State](java/problems/68-state/README.md) | Support-ticket lifecycle built from state objects instead of status checks. |
| 69 | [Template Method](java/problems/69-template-method/README.md) | Data-import template with fixed steps, overridable hooks and guaranteed clean-up. |
| 70 | [Visitor](java/problems/70-visitor/README.md) | Document elements visited by HTML-export and word-count visitors. |
| 71 | [Mediator](java/problems/71-mediator/README.md) | Air-traffic control tower coordinating aircraft landings. |
| 72 | [Memento](java/problems/72-memento/README.md) | Game character saved into opaque snapshots: save slots and undo/redo. |
| 73 | [Chain of Responsibility](java/problems/73-chain-of-responsibility/README.md) | Expense approval chain with configurable order and links. |

## The `play` runner

A std-only Rust CLI (in `cli/`) to run and track each problem separately. Run it from the repo root:

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
cd java && mvn test                          # all Java problems (add -Phidden for interviewer scenarios)
cd java/problems/07-chess && mvn test        # one Java problem
cd rust && cargo test                        # all Rust problems
cd rust/problems/07-chess && cargo test      # one Rust problem
STRICT=1 cargo test                          # "not implemented" becomes a failure
```

Behind a proxy, give Maven the proxy settings through `MAVEN_OPTS` (`-Dhttps.proxyHost=... -Dhttps.proxyPort=...`).

## Where to write your solutions

> **Do not write solutions on `main`.** Keep `main` as the clean, empty playground and create a separate branch for your solutions:
>
> ```bash
> git checkout -b my-solutions
> ```
>
> You can then merge `main` into your solutions branch whenever problems are added or improved, and start fresh by branching again.

## Contributing

Adding or editing a problem? Read [CONTRIBUTING.md](CONTRIBUTING.md): READMEs state observable behaviour only, stubs carry at
most a one-line role comment, hints go in `HINTS.md`, and every new problem is validated against a throwaway reference solution.
