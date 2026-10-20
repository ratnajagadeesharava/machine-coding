# 50 · DIP — Dependency Inversion Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — `MessageSender`, `AppointmentRepository` and the `Appointment` record are provided; `ReminderService` is yours to write. |

## Problem

A clinic wants a job that texts patients the day before their appointment. The service must be testable without
sending real SMS messages: the tests drive it with a fake sender, a fake in-memory repository and a hand-driven `Clock`.

## Requirements

`ReminderService(MessageSender, AppointmentRepository, Clock)` with `int sendDueReminders()`:

1. `now = clock.instant()`; ask the repository for appointments in **`[now, now + 24h]`**.
2. For each, send `"Reminder: appointment at " + time` (`Instant.toString()`) to the appointment's phone.
3. `send` returned `true` → `repository.markReminded(id)` and count it. It returned `false` or threw → do **not** mark it (it is retried next run) and continue with the rest.
4. Return the number of reminders sent. Nothing due → 0 and the sender is never called.
5. The repository is responsible for not returning already-reminded appointments; the service keeps no state of its own.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement ReminderService
```
