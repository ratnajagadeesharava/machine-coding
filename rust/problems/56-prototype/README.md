# 56 · Prototype (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Blueprint` trait and the error types are provided; two blueprint types and a registry are stubs. |

## Problem

A cloud team stamps out hundreds of near-identical servers. A "prod database" has a particular disk layout and tags;
creating one by hand is error-prone. Keep a catalogue of ready-made blueprints and create new servers by cloning them.

## Requirements

1. `WebServer::new(name)`: kind `"web"`, one 20 GB disk, no tags. `DbServer::new(name)`: kind `"db"`, disks `[100, 500]`, no tags. Blank name → `Err(BlueprintError::InvalidName)`.
2. `add_tag`: blank and already-present tags are ignored. `resize_disk(index, gb)`: bad index or 0 GB → `Err(BlueprintError::InvalidDisk)`.
3. `clone_box()` returns an independent copy of the same concrete type with the same name, tags and disk sizes. Changing the clone never changes the original and vice versa.
4. `BlueprintRegistry::register(key, &dyn Blueprint)` stores a **copy** (later changes to the original must not leak into the catalogue); registering a key again replaces the old one. Blank key → `Err(RegistryError::BlankKey)`.
5. `create(key, new_name)` returns a fresh independent copy renamed to `new_name`; unknown key → `Err(UnknownKey)`, blank name → `Err(BlankName)`.

## Running

```bash
cargo test -p p56_prototype     # from rust/ — tests FAIL until you implement the stubs
cargo play run 56
```
