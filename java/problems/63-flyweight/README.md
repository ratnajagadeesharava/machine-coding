# 63 · Flyweight (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — four stub classes: `MarkerStyle` (the flyweight), `StyleFactory`, `Marker` and `MapLayer`. |

> **Pattern:** *Flyweight* — share the heavy, **intrinsic** (identical-for-everyone) part of many objects, and keep
> only the light, **extrinsic** (per-object) part in each one. A map with 10,000 markers has 3 icons in memory, not
> 10,000.

## Problem

A map app shows tens of thousands of pins. Each pin has a heavy icon bitmap (4096 bytes here; megabytes in real
life) that depends only on its **shape and colour**, plus a small position and label. Allocating a bitmap per pin
exhausts memory. Share the icons.

## Requirements

1. `MarkerStyle` — the flyweight: `shape()`, `color()`, `bitmapSize()` (always 4096; allocate a real `byte[4096]` once per style). **No public constructor** (package-private), so only the factory can create one.
2. `StyleFactory.get(shape, color)` returns the **same instance** for the same pair (exact, case-sensitive match); `("pin","red")`, `("pin","Red")` and `("flag","red")` are three different styles. Blank shape/colour → `IllegalArgumentException`. `instancesCreated()` counts styles created.
3. **Thread-safe:** 32 threads asking for the same 5 keys together must still create exactly 5 styles.
4. `Marker(lat, lng, label, style)` keeps position, label and a **reference** to the shared style, and declares **no `byte[]` field**. Latitude `-90..90`, longitude `-180..180`, non-null style, else `IllegalArgumentException`.
5. `MapLayer(factory).addMarker(lat, lng, label, shape, color)` obtains the style from the factory. `markerCount()`, `markers()` (read-only list), `render()` → `"<shape>/<color> @ <lat>,<lng> <label>"` per marker, e.g. `pin/red @ 12.5,-3.25 Cafe`.

## Constraints (what the tests catch)

- 10,000 markers over 3 kinds → `instancesCreated() == 3`, and the markers reference exactly 3 distinct style objects (checked by identity).

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
