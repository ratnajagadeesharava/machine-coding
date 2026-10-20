# 71 · Mediator (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 60 min | Level 2 — the `Aircraft` and `Tower` interfaces are provided; you write `ControlTower` and `PassengerJet`. |


## Problem

An airport has one runway and many aircraft. The **control tower** decides who lands next and tells every aircraft
its position.

What you are building: the air-traffic logic of a single-runway airport. Pilots ask the tower for permission to land.
If the runway is free the aircraft is told to land at once; otherwise it is told to hold and given its place in the
landing queue. Whenever the queue changes (someone lands, diverts or declares an emergency) every aircraft whose
place changed must be told its new position, in a precise order. Aircraft only ever talk to the tower, never to each
other. You also build `PassengerJet`, a real aircraft that reports its own status.

### Glossary

- **Runway**: holds at most one aircraft at a time, the one currently cleared to land.
- **Queue**: the waiting aircraft in landing order; position 1 is next to land.
- **Cleared**: the tower told the aircraft `onClearedToLand()`; it is now the one on the runway.
- **Divert**: the aircraft gives up and goes elsewhere (`cancel`).
- **Emergency**: a waiting aircraft jumps to the front of the queue.

## Requirements

`ControlTower implements Tower` (aircraft are identified by `id()`; two objects with the same id are the same aircraft):

1. `requestLanding(a)`: runway free → `a.onClearedToLand()`; otherwise `a` joins the queue and gets `onHold(position)` (1 = next). `null`, or an aircraft already on the runway or in the queue → `IllegalArgumentException`.
2. `landed(a)`: `a` must be the aircraft on the runway, else `IllegalStateException` (this includes an aircraft that is only waiting, an unknown aircraft, or an empty runway). The runway is freed; if someone is waiting, the first in the queue moves onto the runway and gets `onClearedToLand()`, **then** each aircraft still waiting gets `onHold(newPosition)`.
3. `cancel(a)`: a diversion. On the runway → same as `landed`. Waiting → removed, and each aircraft **behind** it gets `onHold(newPosition)` (aircraft in front of it hear nothing). Returns `true` when `a` was known, `false` if `a` is unknown (including one already cancelled).
4. `declareEmergency(a)`: moves a waiting aircraft to the front; it gets `onHold(1)` **first**, then each aircraft it overtook gets its new position, in queue order. Already first → nothing happens (no callbacks). Unknown or already cleared (on the runway) → `IllegalArgumentException`.
5. `onRunway()` returns the id of the aircraft on the runway (`Optional.empty()` when free); `queue()` returns the waiting ids in landing order. After everyone has landed both are empty and the next request is cleared at once.
6. `PassengerJet(id, tower)` is a real aircraft: `requestLanding()` (asks the tower), `touchdown()` (tells the tower it landed, status `LANDED`), `divert()` (tells the tower it cancels, status `DIVERTED`), `status()` ∈ `AIRBORNE` (initial), `HOLDING #n`, `CLEARED`, `LANDED`, `DIVERTED`; the tower's callbacks set `CLEARED` / `HOLDING #n`. A jet never keeps a reference to another aircraft (checked by reflection).

## Examples

Callbacks received, in order (`A:CLEARED` means `A.onClearedToLand()`, `B:HOLD 1` means `B.onHold(1)`).

| Call | Callbacks | Runway | Queue |
|---|---|---|---|
| `requestLanding(A)` | `A:CLEARED` | A | [] |
| `requestLanding(B)`, `(C)`, `(D)` | `B:HOLD 1`, `C:HOLD 2`, `D:HOLD 3` | A | [B, C, D] |
| `declareEmergency(D)` | `D:HOLD 1`, `B:HOLD 2`, `C:HOLD 3` | A | [D, B, C] |
| `cancel(B)` (waiting) | `C:HOLD 2` | A | [D, C] |
| `landed(A)` | `D:CLEARED`, `C:HOLD 1` | D | [C] |
| `cancel(D)` (on runway) | `C:CLEARED` | C | [] |

Jets: `QF1`, `QF2`, `QF3` all call `requestLanding()` → statuses `CLEARED`, `HOLDING #1`, `HOLDING #2`. `QF1.touchdown()` → `QF1` is `LANDED`, `QF2` is `CLEARED`, `QF3` is `HOLDING #1`. `QF2.divert()` → `QF2` is `DIVERTED`, `QF3` is `CLEARED`.

## Edge cases

- `requestLanding` with a different object carrying an id already in use → `IllegalArgumentException`.
- `landed(b)` for a waiting `b`, or for an unknown aircraft → `IllegalStateException`; state is unchanged.
- `declareEmergency` on the aircraft already first in the queue is a silent no-op.
- `cancel` of the last queued aircraft sends no callbacks (nobody is behind it).

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
