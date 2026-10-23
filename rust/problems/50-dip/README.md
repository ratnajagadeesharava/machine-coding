# 50 · DIP — Dependency Inversion Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `MessageSender` and `AppointmentRepository` traits and the `Appointment` struct are provided; `ReminderService` is yours to write. The clock is `playground_support::Clock`. |

## Problem

A clinic wants a job that texts patients the day before their appointment. The service must be testable without
sending real texts: the tests drive it with a fake sender, a fake in-memory repository and a `FakeClock`.

## Requirements

`ReminderService::new(Box<dyn MessageSender>, Box<dyn AppointmentRepository>, Arc<dyn Clock>)` with `send_due_reminders(&self) -> usize`:

1. `now = clock.now_millis()`; ask the repository for appointments in **`[now, now + 24h]`** (milliseconds).
2. For each, send `format!("Reminder: appointment at {time_ms}")` to the appointment's phone.
3. `send` returned `true` → `repository.mark_reminded(id)` and count it. It returned `false` → do **not** mark it (it is retried next run) and continue with the rest.
4. Return the number of reminders sent. Nothing due → 0 and the sender is never called.
5. The repository is responsible for not returning already-reminded appointments; the service keeps no state of its own.

## Running

```bash
cargo test -p p50_dip     # from rust/ — tests FAIL until you implement ReminderService
cargo play run 50
```
