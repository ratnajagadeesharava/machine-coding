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
| 74 | [Concurrent LRU Cache (Read-Write Lock)](problems/74-rw-lock-lru-cache/README.md) | Medium-Hard | 60 min | L3 |
| 75 | [Bank Transfers Without Deadlock](problems/75-bank-transfer/README.md) | Medium | 45 min | L3 |
| 76 | [Ordered Printer (Turn Taking)](problems/76-ordered-printer/README.md) | Medium | 45 min | L3 |
| 77 | [Dining Philosophers](problems/77-dining-philosophers/README.md) | Medium-Hard | 60 min | L3 |
| 78 | [Bounded Connection Pool](problems/78-connection-pool/README.md) | Medium-Hard | 60 min | L3 |
| 79 | [Parallel Merge Sort](problems/79-parallel-merge-sort/README.md) | Medium-Hard | 60 min | L3 |
| 80 | [Parallel Word Count](problems/80-parallel-word-count/README.md) | Medium | 45 min | L3 |
| 81 | [Parallel Web Crawler](problems/81-parallel-crawler/README.md) | Hard | 90 min | L3 |
| 82 | [URL Shortener](problems/82-url-shortener/README.md) | Medium | 45 min | L2 |
| 83 | [Feature Flag Service](problems/83-feature-flags/README.md) | Medium | 45 min | L2 |
| 84 | [In-Memory File System with Permissions](problems/84-file-system-permissions/README.md) | Medium-Hard | 75 min | L2 |
| 85 | [Text Editor with Undo/Redo](problems/85-text-editor-undo/README.md) | Medium | 60 min | L2 |
| 86 | [Document Versioning with Merge](problems/86-document-versioning/README.md) | Hard | 75 min | L2 |
| 87 | [Expense Groups (Splitwise)](problems/87-expense-groups/README.md) | Hard | 75 min | L2 |
| 88 | [Shared Event Calendar](problems/88-event-calendar/README.md) | Medium-Hard | 60 min | L2 |
| 89 | [Movie Ticket Booking](problems/89-movie-ticketing/README.md) | Hard | 75 min | L2 |
| 90 | [Food Delivery Dispatch](problems/90-food-delivery/README.md) | Hard | 75 min | L2 |
| 91 | [Online Auction](problems/91-online-auction/README.md) | Medium-Hard | 60 min | L2 |
| 92 | [Chat System](problems/92-chat-system/README.md) | Hard | 75 min | L2 |
| 93 | [Cron Job Scheduler](problems/93-cron-job-scheduler/README.md) | Hard | 75 min | L2 |
| 94 | [API Gateway](problems/94-api-gateway/README.md) | Hard | 75 min | L2 |
| 95 | [Mini SQL Engine](problems/95-sql-engine/README.md) | Hard | 90 min | L2 |
| 96 | [Consistent Hash Ring](problems/96-consistent-hashing/README.md) | Medium-Hard | 60 min | L3 |
| 97 | [Idempotent Request Processor](problems/97-idempotent-requests/README.md) | Medium-Hard | 60 min | L3 |
| 98 | [Lease Lock with Fencing Tokens](problems/98-lock-lease-fencing/README.md) | Medium-Hard | 50 min | L3 |
| 99 | [Raft Leader Election](problems/99-leader-election/README.md) | Hard | 90 min | L3 |
| 100 | [Gossip Protocol with Failure Detection](problems/100-gossip-protocol/README.md) | Hard | 90 min | L3 |
| 101 | [Two-Phase Commit](problems/101-two-phase-commit/README.md) | Hard | 120 min | L3 |
| 102 | [Saga Orchestrator](problems/102-saga-orchestrator/README.md) | Hard | 90 min | L3 |
| 103 | [CRDT Replicated Store](problems/103-crdt-replication/README.md) | Hard | 75 min | L3 |
| 104 | [Quorum Key-Value Store](problems/104-quorum-kv/README.md) | Hard | 90 min | L3 |
| 105 | [Raft Log Replication](problems/105-raft-log/README.md) | Hard | 120 min | L3 |
