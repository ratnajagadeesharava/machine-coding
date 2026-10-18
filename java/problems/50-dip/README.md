# 50 · DIP — Dependency Inversion Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — `MessageSender`, `AppointmentRepository` and the `Appointment` record are provided; `ReminderService` is a stub. |

> **Principle:** high-level policy must not depend on low-level details; both depend on abstractions. The service
> decides *what* should happen (remind patients); the SMS gateway, the database and the system clock are details that
> are **handed in** through interfaces.

## Problem

A clinic wants a job that texts patients the day before their appointment. The first version did
`new TwilioClient()` and `DriverManager.getConnection(...)` and `Instant.now()` inside the service, so nobody could
test it without sending real SMS messages at 3 a.m. Rewrite the service so it can run against fakes.

## Requirements

`ReminderService(MessageSender, AppointmentRepository, Clock)` with `int sendDueReminders()`:

1. `now = clock.instant()`; ask the repository for appointments in **`[now, now + 24h]`**.
2. For each, send `"Reminder: appointment at " + time` (`Instant.toString()`) to the appointment's phone.
3. `send` returned `true` → `repository.markReminded(id)` and count it. It returned `false` or threw → do **not** mark it (it is retried next run) and continue with the rest.
4. Return the number of reminders sent. Nothing due → 0 and the sender is never called.
5. The repository is responsible for not returning already-reminded appointments; the service keeps no state of its own.

## Constraints (what the tests catch)

- The tests inject a fake sender, a fake in-memory repository and a hand-driven `Clock`. Anything the service creates itself instead of receiving is invisible to them.
- Reflection asserts the single public constructor takes exactly `(MessageSender, AppointmentRepository, Clock)`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement ReminderService
```
