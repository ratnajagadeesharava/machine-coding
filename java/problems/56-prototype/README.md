# 56 · Prototype (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Blueprint` contract is provided; two blueprint classes and a registry are stubs. |

## Problem

A cloud team stamps out hundreds of near-identical servers. A "prod database" has a particular disk layout, tags
and settings; creating one by hand is error-prone. Keep a catalogue of ready-made blueprints and create new servers
by cloning them.

## Requirements

1. `WebServer(name)`: kind `"web"`, one 20 GB disk, no tags. `DbServer(name)`: kind `"db"`, disks `[100, 500]`, no tags. Blank name → `IllegalArgumentException`.
2. `addTag`: blank and already-present tags are ignored. `tags()` and `diskSizesGb()` return views that cannot change the blueprint. `resizeDisk(index, gb)`: bad index or `gb <= 0` → `IllegalArgumentException`.
3. `copy()` returns an independent copy of the same concrete type with the same name, tags and disk sizes. Changing the clone never changes the original and vice versa.
4. `BlueprintRegistry.register(key, prototype)` stores a **copy** (later changes to the object you passed in must not leak into the catalogue); registering a key again replaces the old one. Blank key or `null` → `IllegalArgumentException`.
5. `create(key, newName)` returns a fresh independent copy renamed to `newName`; unknown key or blank name → `IllegalArgumentException`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
