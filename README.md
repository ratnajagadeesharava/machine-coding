# Machine Coding + LLD Interview Playground

Senior-level (≈6 yrs) practice for **machine-coding / low-level-design interviews**, in **Java 21** and **Rust**.

> **Give me the requirements. Give me the tests. Give me the playground. Do NOT give me the architecture.**

You receive what a senior engineer would be handed at the start of a real interview:

```
Problem → Requirements → Constraints → Examples → Behavioral contract → Test + build environment
```

You are responsible for everything in the middle:

```
Classes · Interfaces · Enums · Value objects · Services · Policies · State machines
Data structures · Ownership of state · Concurrency model · Implementation · Refactoring
```

There are **no** `ParkingLot.java` / `Piece.rs` skeletons with `TODO`s. `src/main` is empty except for one wiring file.

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

## Problems

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

*(Scaffold: L1 almost empty · L2 minimal external API · L3 infrastructure provided.)*

## OOP, Design Principles and Design Patterns (33-73)

Forty-one smaller problems, one per concept, in the same Java + Rust layout. They are **not** open design problems: each ships a
short `README.md` with the exact API, **stubs** (`todo!()` / `UnsupportedOperationException`) so the tests compile, and tests that check
the *concept* (lifecycle, substitutability, call order, laziness, immutability...), not only the output.

> **These tests FAIL until you implement the stubs** (they do not skip). To run one problem: `cargo play run 52` or
> `cd java/problems/52-singleton && mvn test`. Reference solutions are kept out of the way in the git-ignored `solutions/` folder.

### A. OOP fundamentals (9)

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

### B. Design principles (4)

| # | Problem | Difficulty | Time | Scaffold | What you practise |
|---|---|---|---:|:---:|---|
| 42 | [DRY](java/problems/42-dry/README.md) | Medium | 30 min | L2 | Quote, invoice and credit note sharing one pricing path and one money formatter. |
| 43 | [KISS](java/problems/43-kiss/README.md) | Easy | 20 min | L2 | Library late fee in one small function; the tests enforce a simplicity budget. |
| 44 | [YAGNI](java/problems/44-yagni/README.md) | Easy | 25 min | L2 | URL shortener with exactly two operations; the tests reject speculative extras. |
| 45 | [Law of Demeter](java/problems/45-law-of-demeter/README.md) | Medium | 40 min | L2 | Checkout and shipping services that never reach through their collaborators. |

### C. SOLID (6)

| # | Problem | Difficulty | Time | Scaffold | What you practise |
|---|---|---|---:|:---:|---|
| 46 | [Single Responsibility](java/problems/46-srp/README.md) | Medium | 40 min | L2 | Invoice issuing split into calculator, renderer, repository and a coordinating service. |
| 47 | [Open/Closed](java/problems/47-ocp/README.md) | Medium | 35 min | L2 | Parking-fee calculator extended with new vehicle policies without being modified. |
| 48 | [Liskov Substitution](java/problems/48-lsp/README.md) | Medium | 40 min | L2 | Coupons that all honour one contract, verified by a shared test suite. |
| 49 | [Interface Segregation](java/problems/49-isp/README.md) | Medium | 35 min | L2 | Printer / scanner / fax: small interfaces, devices implement only what they can do. |
| 50 | [Dependency Inversion](java/problems/50-dip/README.md) | Medium | 35 min | L2 | Appointment reminders with an injected sender, repository and clock. |
| 51 | [SOLID Summary](java/problems/51-solid-summary/README.md) | Hard | 75 min | L2 | Meal-order checkout combining S, O, L, I and D in one mini-system. |

### D. Creational patterns (5)

| # | Problem | Difficulty | Time | Scaffold | What you practise |
|---|---|---|---:|:---:|---|
| 52 | [Singleton](java/problems/52-singleton/README.md) | Medium | 30 min | L2 | Lazy, thread-safe application config with exactly one instance. |
| 53 | [Factory Method](java/problems/53-factory-method/README.md) | Medium | 35 min | L2 | Checkout flows whose subclasses choose the payment gateway through a factory method. |
| 54 | [Abstract Factory](java/problems/54-abstract-factory/README.md) | Medium | 45 min | L2 | Regional kits (US/EU/IN) creating consistent families of tax, price and address products. |
| 55 | [Builder](java/problems/55-builder/README.md) | Medium | 45 min | L2 | Immutable HTTP request created by a validating fluent builder. |
| 56 | [Prototype](java/problems/56-prototype/README.md) | Medium | 40 min | L2 | Server blueprints deep-cloned from a registry of prototypes. |

### E. Structural patterns (7)

| # | Problem | Difficulty | Time | Scaffold | What you practise |
|---|---|---|---:|:---:|---|
| 57 | [Adapter](java/problems/57-adapter/README.md) | Medium | 35 min | L2 | Legacy XML / integer-code SMS gateway exposed as a clean `Notifier`. |
| 58 | [Facade](java/problems/58-facade/README.md) | Medium | 35 min | L2 | Photo upload hiding scanner, resizer, storage and index behind one call, with compensation. |
| 59 | [Decorator](java/problems/59-decorator/README.md) | Medium | 40 min | L2 | Message sinks (timestamp, redact, retry, count) stacked in any order. |
| 60 | [Composite](java/problems/60-composite/README.md) | Medium | 40 min | L2 | File-system tree where files and folders share one interface. |
| 61 | [Proxy](java/problems/61-proxy/README.md) | Medium | 45 min | L2 | Caching, lazy and access-control proxies around a document store. |
| 62 | [Bridge](java/problems/62-bridge/README.md) | Hard | 50 min | L2 | Alerts x channels: any alert type can use any delivery channel. |
| 63 | [Flyweight](java/problems/63-flyweight/README.md) | Medium | 40 min | L2 | Map markers sharing icon flyweights from a thread-safe factory. |

### F. Behavioural patterns (10)

| # | Problem | Difficulty | Time | Scaffold | What you practise |
|---|---|---|---:|:---:|---|
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

Java and Rust share the **same behavioral requirements**, not the same code — Java problems lean on classes,
collections, executors and locks; Rust ones on structs/enums/traits, ownership, `Arc`/`Mutex`/channels where
(and only where) required. Rust traits use `&mut self` unless the problem requires concurrent access, in which
case they take `&self` + `Send + Sync`.

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

## Getting started

```bash
cd java && mvn test                  # all problems
cd java/problems/07-chess && mvn test
cd rust && cargo test
cd rust/problems/07-chess && cargo test
```

Pick a problem, open its `README.md`, start the timer in `INTERVIEW.md`, and **stare at the requirements before typing**.

## Where to write your solutions

> **Do not write solutions on `main`.** Keep `main` as the clean, empty playground.
> Create a separate branch for your solutions and work there:
>
> ```bash
> git checkout -b my-solutions
> ```
>
> That way you can always pull updates to the problems on `main` and start fresh by branching again.
