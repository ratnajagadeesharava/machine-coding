# 71 · Mediator (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 60 min | Level 2 — the `Aircraft` trait and `TowerError` are provided; you write `ControlTower`. |


## Problem

An airport has one runway and many aircraft. The **control tower** decides who lands next and tells every aircraft
its position.

What you are building: the air-traffic logic of a single-runway airport. Pilots ask the tower for permission to land.
If the runway is free the aircraft is told to land at once; otherwise it is told to hold and given its place in the
landing queue. Whenever the queue changes (someone lands, diverts or declares an emergency) every aircraft whose
place changed must be told its new position, in a precise order. Aircraft only ever talk to the tower, never to each
other.

### Glossary

- **Runway**: holds at most one aircraft at a time, the one currently cleared to land.
- **Queue**: the waiting aircraft in landing order; position 1 is next to land.
- **Cleared**: the tower called `on_cleared_to_land()` on the aircraft; it is now the one on the runway.
- **Divert**: the aircraft gives up and goes elsewhere (`cancel`).
- **Emergency**: a waiting aircraft jumps to the front of the queue.

## Requirements

`ControlTower::new()`; aircraft are handed over as `Rc<dyn Aircraft>` and identified by `id()` (two aircraft with the same id are the same aircraft). The tower's `landed`, `cancel` and `declare_emergency` take the aircraft id (`&str`):

1. `request_landing(a)`: runway free → `a.on_cleared_to_land()`; otherwise `a` joins the queue and gets `on_hold(position)` (1 = next). An aircraft already on the runway or in the queue → `Err(TowerError::Duplicate)`.
2. `landed(id)`: `id` must be the aircraft on the runway, else `Err(TowerError::NotOnRunway)` (this includes an aircraft that is only waiting, an unknown id, or an empty runway). The runway is freed; if someone is waiting, the first in the queue moves onto the runway and gets `on_cleared_to_land()`, **then** each aircraft still waiting gets `on_hold(new_position)`.
3. `cancel(id)`: a diversion. On the runway → same as `landed`. Waiting → removed, and each aircraft **behind** it gets `on_hold(new_position)` (aircraft in front of it hear nothing). Returns `true` when the id was known, `false` if the id is unknown (including one already cancelled).
4. `declare_emergency(id)`: moves a waiting aircraft to the front; it gets `on_hold(1)` **first**, then each aircraft it overtook gets its new position, in queue order. Already first → `Ok(())` and nothing happens (no callbacks). Unknown or already cleared (on the runway) → `Err(TowerError::Unknown)`.
5. `on_runway()` returns `Option<String>` (the id on the runway, `None` when free); `queue()` returns the waiting ids as `Vec<String>` in landing order. After everyone has landed both are empty and the next request is cleared at once.

## Examples

Callbacks received, in order (`A:CLEARED` means `A.on_cleared_to_land()`, `B:HOLD 1` means `B.on_hold(1)`).

| Call | Callbacks | Runway | Queue |
|---|---|---|---|
| `request_landing(A)` | `A:CLEARED` | A | [] |
| `request_landing(B)`, `(C)`, `(D)` | `B:HOLD 1`, `C:HOLD 2`, `D:HOLD 3` | A | [B, C, D] |
| `declare_emergency("D")` | `D:HOLD 1`, `B:HOLD 2`, `C:HOLD 3` | A | [D, B, C] |
| `cancel("B")` (waiting) | `C:HOLD 2` | A | [D, C] |
| `landed("A")` | `D:CLEARED`, `C:HOLD 1` | D | [C] |
| `cancel("D")` (on runway) | `C:CLEARED` | C | [] |

## Edge cases

- `request_landing` with a different object carrying an id already in use → `Err(Duplicate)`.
- `landed("B")` for a waiting `B`, or for an unknown id → `Err(NotOnRunway)`; state is unchanged.
- `declare_emergency` on the aircraft already first in the queue is a silent no-op.
- `cancel` of the last queued aircraft sends no callbacks (nobody is behind it).

## Running

```bash
cargo test -p p71_mediator     # from rust/ — tests FAIL until you implement ControlTower
cargo play run 71
```
