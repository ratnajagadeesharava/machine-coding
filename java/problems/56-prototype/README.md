# 56 · Prototype (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Blueprint` contract is provided; two blueprint classes and a registry are stubs. |

## Problem

**What you are building.** A cloud team stamps out hundreds of near-identical servers. A "prod database" has a
particular disk layout, tags and settings; creating one by hand is error-prone. You build a catalogue of ready-made
server blueprints: register a template once under a key, then create as many new servers from it as you like, each
with its own name and each completely independent of the template and of each other.

**Glossary**
- *Blueprint* — a description of a server: name, kind, tags, disk sizes (the `Blueprint` interface in `contract/`).
- *Kind* — a short label of the server type: `"web"`, `"db"` (a test even defines `"gpu"`).
- *Prototype* — a blueprint that is used as the template to copy from.
- *Independent copy* — changing the copy never changes the original and vice versa (tags and disk sizes included).

## Requirements

1. `WebServer(name)`: kind `"web"`, one 20 GB disk (`[20]`), no tags. `DbServer(name)`: kind `"db"`, two disks `[100, 500]` (GB), no tags. Blank (or null) name → `IllegalArgumentException`.
2. `addTag(tag)`: a blank tag and a tag already present are silently ignored; `tags()` lists tags in insertion order. `diskSizesGb()` lists disk sizes in GB in disk order. `tags()` and `diskSizesGb()` return values that cannot change the blueprint (a read-only view or a throw-away copy are both fine; modifying the returned list must leave the blueprint unchanged). `resizeDisk(index, gb)` sets one disk size; an index out of range or `gb <= 0` → `IllegalArgumentException` and nothing changes. `rename(newName)` changes `name()`.
3. `copy()` returns an independent copy of the **same concrete type** (a copied `DbServer` is a `DbServer`) with the same name, tags and disk sizes.
4. `BlueprintRegistry.register(key, prototype)` stores a **copy** of the prototype (later changes to the object you passed in must not leak into the catalogue); registering a key again replaces the earlier one. Blank key or `null` prototype → `IllegalArgumentException`.
5. `create(key, newName)` returns a fresh, independent copy of the registered prototype, renamed to `newName`. Unknown key or blank name → `IllegalArgumentException`. Every call returns a different object; changing one created server never affects another or the catalogue.
6. The registry must work with blueprint types it has never seen (e.g. a `GpuServer` defined by a test, kind `"gpu"`, disks `[1000]`), cloning them through their own `copy()`; `register` asks for one copy and each `create` asks for one copy (so register + create = 2 copies).

## Examples

```
db = new DbServer("d1");  db.addTag("prod"); db.addTag("eu"); db.resizeDisk(1, 750);
clone = db.copy();
  -> clone is a DbServer: name "d1", kind "db", tags [prod, eu], disks [100, 750]

web = new WebServer("w1"); web.addTag("blue"); clone = web.copy();
clone.addTag("canary"); clone.rename("w1-clone"); clone.resizeDisk(0, 99);
web.addTag("green");       web.resizeDisk(0, 40);
  -> web:   name "w1",       tags [blue, green],  disks [40]
  -> clone: name "w1-clone", tags [blue, canary], disks [99]

registry.register("db-prod", prototypeWithTag("prod"));   // DbServer "template"
one = registry.create("db-prod", "db-1"); two = registry.create("db-prod", "db-2");
one.addTag("only-one"); one.resizeDisk(0, 1);
  -> two: name "db-2", tags [prod], disks [100, 500]; a later create("db-prod","db-3") has tags [prod]

registry.register("web", webTemplate); webTemplate.addTag("late"); webTemplate.resizeDisk(0, 999);
  -> registry.create("web","w1") has no tags and disks [20]

registry.register("x", new WebServer("a")); registry.register("x", new DbServer("b"));
  -> registry.create("x","n").kind() is "db"
```

## Edge cases

- `resizeDisk(5, 10)` on a web server (one disk) and `resizeDisk(0, 0)` both throw and leave disks `[20]`.
- `create("nope", "n")` (unknown key) and `create("web", " ")` (blank name) throw.
- `register(" ", bp)` and `register("k", null)` throw.
- `new WebServer(" ")` throws.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
