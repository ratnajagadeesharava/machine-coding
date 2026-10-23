# 56 · Prototype (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Blueprint` trait and the error types are provided; two blueprint types and a registry are stubs. |

## Problem

**What you are building.** A cloud team stamps out hundreds of near-identical servers. A "prod database" has a
particular disk layout and tags; creating one by hand is error-prone. You build a catalogue of ready-made server
blueprints: register a template once under a key, then create as many new servers from it as you like, each with its own
name and each completely independent of the template and of each other.

**Glossary**
- *Blueprint* — a description of a server: name, kind, tags, disk sizes (the `Blueprint` trait in `src/contract.rs`).
- *Kind* — a short label of the server type: `"web"`, `"db"` (a test even defines `"gpu"`).
- *Prototype* — a blueprint that is used as the template to copy from.
- *Independent copy* — changing the copy never changes the original and vice versa (tags and disk sizes included).

## Requirements

Errors: `BlueprintError::{InvalidDisk, InvalidName}` and `RegistryError::{BlankKey, BlankName, UnknownKey}` (provided).

1. `WebServer::new(name) -> Result<WebServer, BlueprintError>`: kind `"web"`, one 20 GB disk (`[20]`), no tags. `DbServer::new(name)`: kind `"db"`, two disks `[100, 500]` (GB), no tags. Blank name → `Err(BlueprintError::InvalidName)`.
2. `add_tag(tag)`: a blank tag and a tag already present are silently ignored; `tags()` returns tags in insertion order. `disk_sizes_gb()` returns disk sizes in GB in disk order. `resize_disk(index, gb) -> Result<(), BlueprintError>` sets one disk size; an index out of range or `0` GB → `Err(BlueprintError::InvalidDisk)` and nothing changes. `rename(new_name)` changes `name()`.
3. `clone_box()` returns an independent copy of the **same concrete type** (a cloned `DbServer` has kind `"db"`) with the same name, tags and disk sizes.
4. `BlueprintRegistry::new()`, then `register(&mut self, key, &dyn Blueprint) -> Result<(), RegistryError>` stores a **copy** of the prototype (later changes to the original must not leak into the catalogue); registering a key again replaces the earlier one. Blank key → `Err(RegistryError::BlankKey)`.
5. `create(&self, key, new_name) -> Result<Box<dyn Blueprint>, RegistryError>` returns a fresh, independent copy of the registered prototype, renamed to `new_name`. Unknown key → `Err(UnknownKey)`, blank name → `Err(BlankName)`. Every call returns a new value; changing one created server never affects another or the catalogue.
6. The registry must work with blueprint types it has never seen (e.g. a `GpuServer` defined by a test, kind `"gpu"`, disks `[1000]`), cloning them through their own `clone_box()`; `register` asks for one copy and each `create` asks for one copy (so register + create = 2 copies).

## Examples

```
db = DbServer::new("d1"); db.add_tag("prod"); db.add_tag("eu"); db.resize_disk(1, 750);
clone = db.clone_box();
  -> name "d1", kind "db", tags [prod, eu], disks [100, 750]

web = WebServer::new("w1"); web.add_tag("blue"); clone = web.clone_box();
clone.add_tag("canary"); clone.rename("w1-clone"); clone.resize_disk(0, 99);
web.add_tag("green");       web.resize_disk(0, 40);
  -> web:   name "w1",       tags [blue, green],  disks [40]
  -> clone: name "w1-clone", tags [blue, canary], disks [99]

registry.register("db-prod", &prototype);   // DbServer "template" tagged "prod"
one = registry.create("db-prod", "db-1"); two = registry.create("db-prod", "db-2");
one.add_tag("only-one"); one.resize_disk(0, 1);
  -> two: name "db-2", kind "db", tags [prod], disks [100, 500]; a later create("db-prod","db-3") has tags [prod]

registry.register("web", &web_template); web_template.add_tag("late"); web_template.resize_disk(0, 999);
  -> registry.create("web","w1") has no tags and disks [20]

registry.register("x", &WebServer::new("a")); registry.register("x", &DbServer::new("b"));
  -> registry.create("x","n").kind() is "db"
```

## Edge cases

- `resize_disk(5, 10)` on a web server (one disk) and `resize_disk(0, 0)` both return `Err(InvalidDisk)` and leave disks `[20]`.
- `create("nope", "n")` → `Err(UnknownKey)`; `create("web", " ")` → `Err(BlankName)`.
- `register(" ", &bp)` → `Err(BlankKey)`.
- `WebServer::new(" ")` → `Err(InvalidName)`.

## Running

```bash
cargo test -p p56_prototype     # from rust/ — tests FAIL until you implement the stubs
cargo play run 56
```
