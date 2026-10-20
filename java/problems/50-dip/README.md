# 50 · DIP — Dependency Inversion Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — `MessageSender`, `AppointmentRepository` and the `Appointment` record are provided; `ReminderService` is yours to write. |

## Problem

**What you are building.** A clinic wants a scheduled job that texts each patient a reminder when their appointment is
coming up within the next 24 hours. Each time the job runs it looks up the appointments that are due, sends one text per
appointment, and records which ones were reminded so that patients are not texted twice. The code must be testable without
sending real SMS messages: the tests drive it with a fake sender, a fake in-memory repository and a hand-driven `Clock`.

**Glossary**
- *Due* — an appointment whose time lies in the window `[now, now + 24h]`, both ends inclusive, and that has not been reminded yet.
- *Reminded* — the repository has been told (via `markReminded`) that the reminder for that appointment went out.
- *Run* — one call to `sendDueReminders()`. The job may be run many times; each run is independent.

## Requirements

`ReminderService(MessageSender, AppointmentRepository, Clock)` with `int sendDueReminders()`:

1. `now = clock.instant()` (read on every run, so it follows the clock as time passes); ask the repository **exactly once per run** for appointments in **`[now, now + 24h]`** via `findBetween(now, now.plus(24 hours))`.
2. For each appointment returned, call `sender.send(phone, text)` with the appointment's phone and the text `"Reminder: appointment at " + time`, where `time` is `Instant.toString()` (e.g. `2026-03-10T11:00:00Z`).
3. `send` returned `true` → call `repository.markReminded(id)` and count it. It returned `false` **or threw an exception** → do **not** mark it (it will be retried on the next run), do not count it, and continue with the remaining appointments. The exception is not propagated to the caller.
4. Return the number of reminders successfully sent. Nothing due → return 0 and the sender is never called.
5. The repository is responsible for not returning already-reminded appointments; the service keeps no state of its own between runs.

## Examples

`now = 2026-03-10T08:00:00Z`; all appointments are unreminded unless stated.

| Appointments (id, phone, time) | Sender behaviour | Result | Marked reminded | Texts sent |
|---|---|---|---|---|
| `a1`, `+100`, 11:00Z (now + 3h) | accepts | `1` | `a1` | `+100 \| Reminder: appointment at 2026-03-10T11:00:00Z` |
| `a1` +100 (now+1h), `a2` +200 (now+2h), `a3` +300 (now+3h) | `+200` returns `false` | `2` | `a1`, `a3` | all three attempted |
| `a1` +100 (now+1h), `a2` +200 (now+2h) | `+100` throws | `1` | `a2` | `a2` still gets its text |
| `late` (now+30h), `past` (now−5 min) | — | `0` | none | none |

Retry across runs: after the second row above, if `+200` now accepts, a second `sendDueReminders()` returns `1` and sends only to `+200` (the repository no longer returns `a1` and `a3`).

Clock moves: an appointment at now+30h gives `0` on the first run. After the clock advances by 10h it is 20h away, so the next run queries `[now+10h, now+34h]` and returns `1`.

## Edge cases

- An appointment exactly at `now` or exactly at `now + 24h` is due; one a moment after `now + 24h` or any time before `now` is not (the repository filters this; you only supply the window).
- No appointments at all → the repository is still queried once, the result is `0`.
- One failing recipient never stops the others from being processed.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement ReminderService
```
