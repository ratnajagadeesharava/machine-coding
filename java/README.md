# Java track

* Java 21+, Maven 3.9+, JUnit 5, AssertJ.
* One Maven module per problem under `problems/`; `java/pom.xml` is the parent.

```bash
mvn test                                     # every problem (skips unimplemented ones)
mvn -pl problems/07-chess test               # one problem from this directory
cd problems/07-chess && mvn test             # or from inside the problem
STRICT=1 mvn -pl problems/07-chess test      # not-implemented = failure
mvn -pl problems/07-chess test -Phidden      # + interviewer scenarios
```

## What is provided vs what is yours

| Provided | Yours |
|---|---|
| `contract/*Api.java` — thin, primitive-typed test entry point | everything else under `src/main/java` |
| `solution/Solution.java` — one static factory per contract | the body of those factories, and every class behind them |
| `testkit/` — `FakeClock`, `Concurrently`, `Eventually`, `Candidate` (shared, test-scope) | your own extra tests |

`testkit` is added as a test source root by the parent pom, so a problem builds on its own.

Idioms to practice: records and `final` fields for value types, sealed interfaces/enums where a closed set fits,
`java.util.concurrent` (locks, `ConcurrentHashMap`, executors, latches) over hand-rolled threading, no `synchronized` on everything.

## Problems

| # | Problem | Difficulty | Time | Scaffold |
|---|---|---|---:|:---:|
| 01 | [Parking Lot](problems/01-parking-lot/README.md) | Medium-Hard | 90 min | L1 |
| 02 | [Elevator System](problems/02-elevator/README.md) | Hard | 90 min | L1 |
| 03 | [Library Management](problems/03-library/README.md) | Medium | 60 min | L1 |
| 04 | [Coffee Machine](problems/04-coffee-machine/README.md) | Medium | 60 min | L1 |
| 05 | [Vending Machine](problems/05-vending-machine/README.md) | Medium | 60 min | L1 |
| 06 | [ATM](problems/06-atm/README.md) | Medium | 60 min | L1 |
| 07 | [Chess](problems/07-chess/README.md) | Senior | 120 min | L1 |
| 08 | [Tic-Tac-Toe](problems/08-tic-tac-toe/README.md) | Medium | 45 min | L1 |
| 09 | [Snake and Ladder](problems/09-snake-and-ladder/README.md) | Medium | 45 min | L1 |
| 10 | [Flight Booking](problems/10-flight-booking/README.md) | Hard | 90 min | L1 |
| 11 | [Hotel Booking](problems/11-hotel-booking/README.md) | Hard | 90 min | L1 |
| 12 | [Travel Itinerary Planner](problems/12-travel-itinerary/README.md) | Medium-Hard | 90 min | L1 |
| 13 | [Bus / Train Seat Booking](problems/13-seat-booking/README.md) | Medium-Hard | 90 min | L1 |
| 14 | [Ride Sharing](problems/14-ride-sharing/README.md) | Medium-Hard | 90 min | L1 |
| 15 | [Splitwise](problems/15-splitwise/README.md) | Medium | 60 min | L1 |
| 16 | [Digital Wallet](problems/16-wallet/README.md) | Medium-Hard | 90 min | L1 |
| 17 | [Stock Exchange Order Book](problems/17-order-book/README.md) | Hard | 120 min | L1 |
| 18 | [LRU Cache](problems/18-lru-cache/README.md) | Medium | 45 min | L2 |
| 19 | [LFU Cache](problems/19-lfu-cache/README.md) | Medium-Hard | 60 min | L2 |
| 20 | [Rate Limiter](problems/20-rate-limiter/README.md) | Medium-Hard | 60 min | L3 |
| 21 | [In-Memory Key-Value Store](problems/21-key-value-store/README.md) | Medium-Hard | 90 min | L1 |
| 22 | [Pub-Sub Message Broker](problems/22-pub-sub/README.md) | Medium-Hard | 90 min | L1 |
| 23 | [Notification System](problems/23-notification-system/README.md) | Medium-Hard | 90 min | L2 |
| 24 | [Task Scheduler](problems/24-task-scheduler/README.md) | Hard | 90 min | L3 |
| 25 | [Logging Framework](problems/25-logging-framework/README.md) | Medium-Hard | 90 min | L2 |
| 26 | [Metrics Collector](problems/26-metrics/README.md) | Medium-Hard | 60 min | L1 |
| 27 | [Circuit Breaker](problems/27-circuit-breaker/README.md) | Medium-Hard | 60 min | L3 |
| 28 | [Bounded Blocking Buffer](problems/28-blocking-queue/README.md) | Hard | 90 min | L3 |
| 29 | [Worker Pool](problems/29-worker-pool/README.md) | Hard | 90 min | L3 |
| 30 | [Concurrent Cache with Loaders](problems/30-concurrent-cache/README.md) | Hard | 90 min | L3 |
| 31 | [In-Process Event Bus](problems/31-event-bus/README.md) | Hard | 120 min | L3 |
| 32 | [Board Game Engine](problems/32-board-game-engine/README.md) | Hard | 120 min | L1 |
