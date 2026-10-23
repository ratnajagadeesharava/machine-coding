# 50 · DIP — Dependency Inversion Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `MessageSender` and `AppointmentRepository` traits and the `Appointment` struct are provided; `ReminderService` is yours to write. The clock is `playground_support::Clock`. |

## Problem

**What you are building.** A clinic wants a scheduled job that texts each patient a reminder when their appointment is
coming up within the next 24 hours. Each time the job runs it looks up the appointments that are due, sends one text per
appointment, and records which ones were reminded so that patients are not texted twice. The code must be testable without
sending real texts: the tests drive it with a fake sender, a fake in-memory repository and a `FakeClock`.

**Glossary**
- *Due* — an appointment whose `time_ms` lies in the window `[now, now + 24h]`, both ends inclusive, and that has not been reminded yet.
- *Reminded* — the repository has been told (via `mark_reminded`) that the reminder for that appointment went out.
- *Run* — one call to `send_due_reminders()`. The job may be run many times; each run is independent.
- Times are `u64` milliseconds since the Unix epoch; 24h = `86_400_000` ms.

## Requirements

`ReminderService::new(Box<dyn MessageSender>, Box<dyn AppointmentRepository>, Arc<dyn Clock>)` with `send_due_reminders(&self) -> usize`:

1. `now = clock.now_millis()` (read on every run, so it follows the clock as time passes); ask the repository **exactly once per run** for appointments in **`[now, now + 24h]`** via `find_between(now, now + 24h)` (milliseconds).
2. For each appointment returned, call `sender.send(phone, text)` with the appointment's phone and the text `format!("Reminder: appointment at {time_ms}")` (the plain decimal millisecond value).
3. `send` returned `true` → call `repository.mark_reminded(id)` and count it. It returned `false` → do **not** mark it (it will be retried on the next run), do not count it, and continue with the remaining appointments.
4. Return the number of reminders successfully sent. Nothing due → return 0 and the sender is never called.
5. The repository is responsible for not returning already-reminded appointments; the service keeps no state of its own between runs.

## Examples

`now = 1_000_000_000` ms, `HOUR = 3_600_000`; all appointments are unreminded unless stated.

| Appointments (id, phone, time) | Sender behaviour | Result | Marked reminded | Texts sent |
|---|---|---|---|---|
| `a1`, `+100`, now + 3h | accepts | `1` | `a1` | `+100 \| Reminder: appointment at 1010800000` |
| `a1` +100 (now+1h), `a2` +200 (now+2h), `a3` +300 (now+3h) | `+200` returns `false` | `2` | `a1`, `a3` | all three attempted |
| `late` (now+30h), `past` (now−5 min) | — | `0` | none | none |
| `now` (+0), `edge` (now+24h), `just-after` (now+24h+1ms) | accepts | `2` | `now`, `edge` | two texts |

Retry across runs: after the second row, if `+200` now accepts, a second `send_due_reminders()` returns `1` and sends only to `+200`.

Clock moves: an appointment at now+30h gives `0` on the first run. After the clock advances by 10h the next run queries `[now+10h, now+34h]` and returns `1`.

## Edge cases

- An appointment exactly at `now` or exactly at `now + 24h` is due; 1 ms after that, or any time before `now`, is not (the repository filters this; you only supply the window).
- No appointments at all → the repository is still queried once, the result is `0`.
- One refused recipient never stops the others from being processed.

## Running

```bash
cargo test -p p50_dip     # from rust/ — tests FAIL until you implement ReminderService
cargo play run 50
```
