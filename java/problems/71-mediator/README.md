# 71 · Mediator (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 60 min | Level 2 — the `Aircraft` and `Tower` interfaces are provided; `ControlTower` and `PassengerJet` are stubs. |

> **Pattern:** *Mediator* — define an object that encapsulates how a set of objects interact. The colleagues no
> longer refer to each other (many-to-many becomes one-to-many); all the coordination logic lives in the mediator.

## Problem

An airport has one runway and many aircraft. If aircraft negotiated with each other directly, every plane would need
to know every other plane. Instead, planes talk only to the **control tower**, which decides who lands next and tells
everyone their position.

## Requirements

`ControlTower implements Tower` (aircraft are identified by `id()`; two objects with the same id are the same aircraft):

1. `requestLanding(a)`: runway free → `a.onClearedToLand()`; otherwise `a` joins the queue and gets `onHold(position)` (1 = next). `null`, or an aircraft already on the runway or in the queue → `IllegalArgumentException`.
2. `landed(a)`: `a` must be the aircraft on the runway, else `IllegalStateException`. The runway is freed; if someone is waiting, the first in the queue moves onto the runway and gets `onClearedToLand()`, **then** each aircraft still waiting gets `onHold(newPosition)`.
3. `cancel(a)`: a diversion. On the runway → same as `landed`. Waiting → removed, and each aircraft **behind** it gets `onHold(newPosition)`. Returns `false` if `a` is unknown.
4. `declareEmergency(a)`: moves a waiting aircraft to the front; it gets `onHold(1)` **first**, then each aircraft it overtook gets its new position, in queue order. Already first → nothing happens. Unknown or already cleared → `IllegalArgumentException`.
5. `onRunway()` / `queue()` report the current state (ids).
6. `PassengerJet(id, tower)` is a real aircraft: `requestLanding()`, `touchdown()` (tells the tower, status `LANDED`), `divert()` (status `DIVERTED`), `status()` ∈ `AIRBORNE`, `HOLDING #n`, `CLEARED`, `LANDED`, `DIVERTED`; the callbacks set `CLEARED` / `HOLDING #n`.

## Constraints (what the tests catch)

- The tests use test-defined aircraft that log every callback, so the exact order and numbers of notifications are asserted.
- Reflection: `PassengerJet` has **no field** whose type is an `Aircraft`/`PassengerJet` — colleagues only know the tower.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
