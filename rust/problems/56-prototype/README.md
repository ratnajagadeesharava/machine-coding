# 56 · Prototype (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Blueprint` trait (with `clone_box`) and the error types are provided; two blueprint types and a registry are stubs. |

> **Pattern:** *Prototype* — create new objects by **copying an existing one** instead of building from scratch. The
> caller does not need to know the concrete type; the object knows how to copy itself. The copy must be **deep**.
>
> **Rust adaptation:** `#[derive(Clone)]` clones a *concrete* type, but `Box<dyn Blueprint>` cannot be cloned
> directly. The standard trick is a `clone_box(&self) -> Box<dyn Blueprint>` method on the trait; each concrete type
> implements it (usually `Box::new(self.clone())`). That trait method is the prototype operation here.

## Problem

A cloud team stamps out hundreds of near-identical servers. A "prod database" has a particular disk layout and tags;
creating one by hand is error-prone. Keep a catalogue of ready-made blueprints and create new servers by cloning them.

## Requirements

1. `WebServer::new(name)`: kind `"web"`, one 20 GB disk, no tags. `DbServer::new(name)`: kind `"db"`, disks `[100, 500]`, no tags. Blank name → `Err(BlueprintError::InvalidName)`.
2. `add_tag`: blank and already-present tags are ignored. `resize_disk(index, gb)`: bad index or 0 GB → `Err(BlueprintError::InvalidDisk)`.
3. `clone_box()` returns a **deep, independent** copy of the same concrete type with the same name, tags and disk sizes. Changing the clone never changes the original and vice versa.
4. `BlueprintRegistry::register(key, &dyn Blueprint)` stores a **copy** (later changes to the original must not leak into the catalogue); registering a key again replaces the old one. Blank key → `Err(RegistryError::BlankKey)`.
5. `create(key, new_name)` returns a fresh independent copy renamed to `new_name`; unknown key → `Err(UnknownKey)`, blank name → `Err(BlankName)`.

## Constraints (what the tests catch)

- The registry works with a `GpuServer` type that exists only in the test file and counts its `clone_box` calls (one on `register`, one per `create`).

## Running

```bash
cargo test -p p56_prototype     # from rust/ — tests FAIL until you implement the stubs
cargo play run 56
```
