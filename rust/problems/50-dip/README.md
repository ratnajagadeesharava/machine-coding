# 50 · DIP — Dependency Inversion Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `MessageSender` and `AppointmentRepository` traits and the `Appointment` struct are provided; `ReminderService` is a stub. The clock is `playground_support::Clock`. |

> **Principle:** high-level policy must not depend on low-level details; both depend on abstractions. The service
> decides *what* should happen (remind patients); the SMS gateway, the database and the system clock are details that
> are **handed in** as trait objects.

## Problem

A clinic wants a job that texts patients the day before their appointment. The first version created an SMS client,
opened a database connection and called `SystemTime::now()` inside the service, so nobody could test it without
sending real texts at 3 a.m. Rewrite the service so it can run against fakes.

## Requirements

`ReminderService::new(Box<dyn MessageSender>, Box<dyn AppointmentRepository>, Arc<dyn Clock>)` with `send_due_reminders(&self) -> usize`:

1. `now = clock.now_millis()`; ask the repository for appointments in **`[now, now + 24h]`** (milliseconds).
2. For each, send `format!("Reminder: appointment at {time_ms}")` to the appointment's phone.
3. `send` returned `true` → `repository.mark_reminded(id)` and count it. It returned `false` → do **not** mark it (it is retried next run) and continue with the rest.
4. Return the number of reminders sent. Nothing due → 0 and the sender is never called.
5. The repository is responsible for not returning already-reminded appointments; the service keeps no state of its own.

## Constraints (what the tests catch)

- The tests inject a fake sender, a fake in-memory repository and a `FakeClock`. Anything the service creates itself instead of receiving is invisible to them.
- **Rust adaptation:** Java also covers a sender that throws and a reflection check of the constructor; in Rust, failure is a `false` return and the constructor signature is enforced by the compiler.

## Running

```bash
cargo test -p p50_dip     # from rust/ — tests FAIL until you implement ReminderService
cargo play run 50
```
