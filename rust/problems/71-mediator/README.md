# 71 · Mediator (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 60 min | Level 2 — the `Aircraft` trait and `TowerError` are provided; `ControlTower` is a stub. |

> **Pattern:** *Mediator* — define an object that encapsulates how a set of objects interact. The colleagues no
> longer refer to each other (many-to-many becomes one-to-many); all the coordination logic lives in the mediator.
>
> **Rust adaptation:** the Java version also asks for a `PassengerJet` class whose fields are checked by reflection;
> in Rust the aircraft are `Rc<dyn Aircraft>` test types that only ever receive callbacks, and the tower identifies
> them by `id()`.

## Problem

An airport has one runway and many aircraft. If aircraft negotiated with each other directly, every plane would need
to know every other plane. Instead, planes talk only to the **control tower**, which decides who lands next and tells
everyone their position.

## Requirements

`ControlTower` (aircraft are identified by `id()`):

1. `request_landing(a)`: runway free → `a.on_cleared_to_land()`; otherwise `a` joins the queue and gets `on_hold(position)` (1 = next). An aircraft already on the runway or in the queue → `Err(TowerError::Duplicate)`.
2. `landed(id)`: `id` must be the aircraft on the runway, else `Err(TowerError::NotOnRunway)`. The runway is freed; if someone is waiting, the first in the queue moves onto the runway and gets `on_cleared_to_land()`, **then** each aircraft still waiting gets `on_hold(new_position)`.
3. `cancel(id)`: a diversion. On the runway → same as `landed`. Waiting → removed, and each aircraft **behind** it gets `on_hold(new_position)`. Returns `false` if the id is unknown.
4. `declare_emergency(id)`: moves a waiting aircraft to the front; it gets `on_hold(1)` **first**, then each aircraft it overtook gets its new position, in queue order. Already first → nothing happens. Unknown or already cleared → `Err(TowerError::Unknown)`.
5. `on_runway()` / `queue()` report the current state (ids).

## Constraints (what the tests catch)

- The test aircraft log every callback, so the exact order and numbers of notifications are asserted.

## Running

```bash
cargo test -p p71_mediator     # from rust/ — tests FAIL until you implement ControlTower
cargo play run 71
```
