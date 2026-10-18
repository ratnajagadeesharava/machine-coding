# Rust track

* Stable Rust, Cargo workspace. One crate per problem under `problems/`, plus `support/` (shared test infrastructure).

```bash
cargo test                                   # every problem (unimplemented ones print SKIPPED)
cargo test -p p07_chess                      # one problem
cd problems/07-chess && cargo test           # or from inside the problem
STRICT=1 cargo test -p p07_chess             # not-implemented = failure (PowerShell: $env:STRICT=1)
cargo test -p p07_chess --features hidden    # + interviewer scenarios
cargo fmt --check
cargo clippy --all-targets
```

## What is provided vs what is yours

| Provided | Yours |
|---|---|
| `src/contract.rs` — thin, primitive-typed test entry point (a trait + plain types) | every other module/type/trait/enum |
| `create(..)` in `src/lib.rs` — returns `None` until you wire it | the implementation behind it |
| `support/` — `Clock`, `FakeClock`, `SystemClock`, `run_concurrently`, `eventually`, `sut!` | your own extra tests |

Contract traits take `&mut self` for single-threaded problems and `&self` + `Send + Sync` where the problem
demands concurrent use (you then decide how to protect state: `Mutex`, `RwLock`, atomics, channels, sharding…).

Idioms to practice: enums with data over class hierarchies, traits only where polymorphism is needed,
newtypes for IDs, borrowing over cloning, `Arc`/`Mutex`/channels only where actually required.

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
